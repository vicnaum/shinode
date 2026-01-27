//! Fetch task execution for the ingest pipeline.

use crate::p2p::NetworkPeer;
use crate::sync::{BlockPayload, SyncProgressStats};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, OwnedSemaphorePermit};
use tokio::time::timeout;
use tracing::Instrument;

use super::db_writer::DbWriteMode;
use super::fetch::{fetch_ingest_batch, FetchIngestOutcome};
use super::scheduler::PeerWorkScheduler;
use super::stats::{BenchEvent, BenchEventLogger, FetchByteTotals, IngestBenchStats};
use super::types::FetchMode;
use super::PeerHealthTracker;

/// Shared context for fetch tasks (references to pipeline state).
pub struct FetchTaskContext {
    pub scheduler: Arc<PeerWorkScheduler>,
    pub peer_health: Arc<PeerHealthTracker>,
    pub fetched_tx: mpsc::Sender<BlockPayload>,
    pub ready_tx: mpsc::UnboundedSender<NetworkPeer>,
    pub stats: Option<Arc<SyncProgressStats>>,
    pub bench: Option<Arc<IngestBenchStats>>,
    pub events: Option<Arc<BenchEventLogger>>,
    pub active_fetch_tasks: Arc<AtomicU64>,
    pub db_mode: DbWriteMode,
    pub fetch_timeout: Duration,
}

/// Parameters for a single fetch task invocation.
pub struct FetchTaskParams {
    pub peer: NetworkPeer,
    pub blocks: Vec<u64>,
    pub mode: FetchMode,
    pub batch_limit: u64,
    pub permit: OwnedSemaphorePermit,
}

struct ActiveFetchTaskGuard {
    counter: Arc<AtomicU64>,
    stats: Option<Arc<SyncProgressStats>>,
}

impl Drop for ActiveFetchTaskGuard {
    fn drop(&mut self) {
        let prev = self.counter.fetch_sub(1, Ordering::SeqCst);
        let next = prev.saturating_sub(1);
        if let Some(stats) = self.stats.as_ref() {
            stats.set_peers_active(next);
        }
    }
}

/// Execute a single fetch task for a batch of blocks from a peer.
#[expect(clippy::cognitive_complexity, reason = "fetch task with timeout handling and stats")]
pub async fn run_fetch_task(ctx: FetchTaskContext, params: FetchTaskParams) {
    let FetchTaskParams {
        peer,
        blocks,
        mode,
        batch_limit,
        permit,
    } = params;
    let assigned_blocks = blocks.len();

    let _permit = permit;
    let _active_guard = ActiveFetchTaskGuard {
        counter: Arc::clone(&ctx.active_fetch_tasks),
        stats: ctx.stats.clone(),
    };

    let fetch_started = Instant::now();
    if let Some(events) = ctx.events.as_ref() {
        let range_start = blocks.first().copied().unwrap_or(0);
        let range_end = blocks.last().copied().unwrap_or(range_start);
        events.record(BenchEvent::FetchStart {
            peer_id: format!("{:?}", peer.peer_id),
            range_start,
            range_end,
            blocks: assigned_blocks as u64,
            batch_limit,
        });
    }

    let span = tracing::trace_span!(
        "fetch_batch",
        peer_id = ?peer.peer_id,
        range_start = blocks.first().copied().unwrap_or(0),
        range_end = blocks.last().copied().unwrap_or(0),
        blocks = assigned_blocks,
        batch_limit = batch_limit
    );
    let fetch_future = fetch_ingest_batch(&peer, &blocks).instrument(span);
    let Ok(result) = timeout(ctx.fetch_timeout, fetch_future).await else {
        handle_fetch_timeout(&ctx, &peer, &blocks, assigned_blocks, batch_limit, mode).await;
        return;
    };

    let fetch_elapsed = fetch_started.elapsed();
    match result {
        Ok(outcome) => {
            handle_fetch_success(&ctx, &peer, &blocks, outcome, fetch_elapsed, batch_limit, mode)
                .await;
        }
        Err(err) => {
            handle_fetch_error(&ctx, &peer, &blocks, err, fetch_elapsed, mode).await;
        }
    }

    ctx.peer_health
        .finish_assignment(peer.peer_id, assigned_blocks)
        .await;

    if let Some(stats) = ctx.stats.as_ref() {
        let pending = ctx.scheduler.pending_count().await as u64;
        let inflight = ctx.scheduler.inflight_count().await as u64;
        let escalation = ctx.scheduler.escalation_len().await as u64;
        stats.set_queue(pending);
        stats.set_inflight(inflight);
        stats.set_escalation(escalation);
    }

    let _ = ctx.ready_tx.send(peer);
}

#[expect(clippy::cognitive_complexity, reason = "timeout handling with logging and stats")]
async fn handle_fetch_timeout(
    ctx: &FetchTaskContext,
    peer: &NetworkPeer,
    blocks: &[u64],
    assigned_blocks: usize,
    batch_limit: u64,
    mode: FetchMode,
) {
    tracing::debug!(
        peer_id = ?peer.peer_id,
        blocks = assigned_blocks,
        range_start = blocks.first().copied(),
        range_end = blocks.last().copied(),
        timeout_ms = ctx.fetch_timeout.as_millis() as u64,
        "ingest batch timed out"
    );

    if let Some(events) = ctx.events.as_ref() {
        let range_start = blocks.first().copied().unwrap_or(0);
        let range_end = blocks.last().copied().unwrap_or(range_start);
        events.record(BenchEvent::FetchTimeout {
            peer_id: format!("{:?}", peer.peer_id),
            range_start,
            range_end,
            blocks: assigned_blocks as u64,
            batch_limit,
            timeout_ms: ctx.fetch_timeout.as_millis() as u64,
        });
    }

    // Treat timeout as a failure
    ctx.peer_health
        .note_error(
            peer.peer_id,
            format!("ingest batch timed out after {:?}", ctx.fetch_timeout),
        )
        .await;
    ctx.scheduler.record_peer_failure(peer.peer_id).await;

    if let Some(bench) = ctx.bench.as_ref() {
        bench.record_fetch_failure(blocks.len() as u64, ctx.fetch_timeout);
        bench.record_peer_failure();
    }

    requeue_blocks(ctx, blocks, mode).await;

    ctx.peer_health
        .finish_assignment(peer.peer_id, assigned_blocks)
        .await;

    if let Some(stats) = ctx.stats.as_ref() {
        let pending = ctx.scheduler.pending_count().await as u64;
        let inflight = ctx.scheduler.inflight_count().await as u64;
        let escalation = ctx.scheduler.escalation_len().await as u64;
        stats.set_queue(pending);
        stats.set_inflight(inflight);
        stats.set_escalation(escalation);
    }

    let _ = ctx.ready_tx.send(peer.clone());
}

#[expect(clippy::too_many_lines, clippy::cognitive_complexity, reason = "handles success path with detailed event recording and error tracking")]
async fn handle_fetch_success(
    ctx: &FetchTaskContext,
    peer: &NetworkPeer,
    blocks: &[u64],
    outcome: FetchIngestOutcome,
    fetch_elapsed: Duration,
    batch_limit: u64,
    mode: FetchMode,
) {
    let FetchIngestOutcome {
        payloads,
        missing_blocks,
        fetch_stats,
    } = outcome;

    let assigned_blocks = blocks.len();
    let bytes = if ctx.bench.is_some() || ctx.events.is_some() {
        let mut bytes = FetchByteTotals::default();
        for payload in &payloads {
            bytes.add(super::fetch_payload_bytes(payload));
        }
        Some(bytes)
    } else {
        None
    };

    if let Some(events) = ctx.events.as_ref() {
        let range_start = blocks.first().copied().unwrap_or(0);
        let range_end = blocks.last().copied().unwrap_or(range_start);
        let bytes_headers = bytes.map_or(0, |b| b.headers);
        let bytes_bodies = bytes.map_or(0, |b| b.bodies);
        let bytes_receipts = bytes.map_or(0, |b| b.receipts);
        events.record(BenchEvent::FetchEnd {
            peer_id: format!("{:?}", peer.peer_id),
            range_start,
            range_end,
            blocks: assigned_blocks as u64,
            batch_limit,
            duration_ms: fetch_elapsed.as_millis() as u64,
            missing_blocks: missing_blocks.len() as u64,
            bytes_headers,
            bytes_bodies,
            bytes_receipts,
            headers_ms: fetch_stats.headers_ms,
            bodies_ms: fetch_stats.bodies_ms,
            receipts_ms: fetch_stats.receipts_ms,
            headers_requests: fetch_stats.headers_requests,
            bodies_requests: fetch_stats.bodies_requests,
            receipts_requests: fetch_stats.receipts_requests,
        });
    }

    let completed: Vec<u64> = payloads
        .iter()
        .map(|payload| payload.header.number)
        .collect();
    if !completed.is_empty() {
        let _ = ctx.scheduler.mark_completed(&completed).await;
    }

    if let Some(bench) = ctx.bench.as_ref() {
        if !payloads.is_empty() {
            bench.record_fetch_success(payloads.len() as u64, fetch_elapsed);
            if let Some(bytes) = bytes {
                bench.record_fetch_bytes(bytes);
            }
        }
        if fetch_stats.headers_requests > 0 {
            bench.record_fetch_stage_stats(
                fetch_stats.headers_ms,
                fetch_stats.bodies_ms,
                fetch_stats.receipts_ms,
                fetch_stats.headers_requests,
                fetch_stats.bodies_requests,
                fetch_stats.receipts_requests,
            );
        }
    }

    for payload in payloads {
        if ctx.fetched_tx.send(payload).await.is_err() {
            break;
        }
    }

    if missing_blocks.is_empty() {
        ctx.scheduler.record_peer_success(peer.peer_id).await;
    } else {
        ctx.peer_health
            .note_error(
                peer.peer_id,
                format!("missing {} blocks in batch", missing_blocks.len()),
            )
            .await;

        let empty_response = completed.is_empty();
        if ctx.db_mode == DbWriteMode::Follow {
            // Near the tip, "missing" is often just propagation lag. Avoid banning
            // peers for empty responses while keeping AIMD/quality feedback.
            ctx.scheduler.record_peer_partial(peer.peer_id).await;
        } else if empty_response {
            ctx.scheduler.record_peer_failure(peer.peer_id).await;
        } else {
            ctx.scheduler.record_peer_partial(peer.peer_id).await;
        }

        if let Some(bench) = ctx.bench.as_ref() {
            bench.record_fetch_failure(missing_blocks.len() as u64, fetch_elapsed);
            if empty_response && ctx.db_mode != DbWriteMode::Follow {
                bench.record_peer_failure();
            }
        }

        requeue_blocks(ctx, &missing_blocks, mode).await;

        tracing::trace!(
            peer_id = ?peer.peer_id,
            missing = missing_blocks.len(),
            "ingest batch missing headers or payloads"
        );
    }
}

async fn handle_fetch_error(
    ctx: &FetchTaskContext,
    peer: &NetworkPeer,
    blocks: &[u64],
    err: eyre::Error,
    fetch_elapsed: Duration,
    mode: FetchMode,
) {
    ctx.peer_health
        .note_error(peer.peer_id, format!("ingest error: {err}"))
        .await;
    ctx.scheduler.record_peer_failure(peer.peer_id).await;

    if let Some(bench) = ctx.bench.as_ref() {
        bench.record_fetch_failure(blocks.len() as u64, fetch_elapsed);
        bench.record_peer_failure();
    }

    requeue_blocks(ctx, blocks, mode).await;

    tracing::debug!(peer_id = ?peer.peer_id, error = %err, "ingest batch failed");
}

async fn requeue_blocks(ctx: &FetchTaskContext, blocks: &[u64], mode: FetchMode) {
    match mode {
        FetchMode::Normal => {
            // Blocks that exceed max attempts are promoted to escalation
            let _ = ctx.scheduler.requeue_failed(blocks).await;
        }
        FetchMode::Escalation => {
            // Escalation blocks are re-added for indefinite retry
            for block in blocks {
                ctx.scheduler.requeue_escalation_block(*block).await;
            }
        }
    }
}
