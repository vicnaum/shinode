//! Historical sync orchestration.

mod db_writer;
mod fetch;
mod fetch_task;
mod follow;
mod process;
mod reorg;
mod scheduler;
mod stats;
mod types;

use crate::cli::NodeConfig;
use crate::p2p::PeerPool;
use crate::sync::{BlockPayload, ProgressReporter, SyncProgressStats, SyncStatus};
use alloy_rlp::Encodable;
use eyre::{eyre, Result};
use reth_network_api::PeerId;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

use crate::storage::Storage;
use db_writer::{run_db_writer, DbWriteConfig, DbWriteMode, DbWriterMessage, DbWriterParams};
pub use follow::run_follow_loop;
use process::process_ingest;
pub(crate) use scheduler::PeerHealthTracker;
use scheduler::{PeerHealthConfig, PeerWorkScheduler, SchedulerConfig};
use stats::FetchByteTotals;
pub use stats::{BenchEvent, BenchEventLogger, IngestBenchStats, IngestBenchSummary, SummaryInput};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use types::FetchMode;

use fetch_task::{run_fetch_task, FetchTaskContext, FetchTaskParams};

static PEER_FEEDER_CURSOR: AtomicUsize = AtomicUsize::new(0);

async fn pick_best_ready_peer_index(
    peers: &[crate::p2p::NetworkPeer],
    peer_health: &PeerHealthTracker,
) -> usize {
    let mut best_idx = 0usize;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_samples = 0u64;
    for (idx, peer) in peers.iter().enumerate() {
        let quality = peer_health.quality(peer.peer_id).await;
        // Use exact equality for tie-breaking: only prefer more samples when scores are identical
        #[expect(clippy::float_cmp, reason = "exact equality needed for tie-breaking")]
        if quality.score > best_score
            || (quality.score == best_score && quality.samples > best_samples)
        {
            best_idx = idx;
            best_score = quality.score;
            best_samples = quality.samples;
        }
    }
    best_idx
}

pub(crate) fn build_peer_health_tracker(config: &NodeConfig) -> Arc<PeerHealthTracker> {
    let max_blocks = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4))
        .max(1) as usize;
    let initial_blocks = (config.fast_sync_chunk_size.max(1) as usize).min(max_blocks);
    let scheduler_config = SchedulerConfig {
        blocks_per_assignment: max_blocks,
        initial_blocks_per_assignment: initial_blocks,
        ..SchedulerConfig::default()
    };
    Arc::new(PeerHealthTracker::new(
        PeerHealthConfig::from_scheduler_config(&scheduler_config),
    ))
}

pub struct TailIngestConfig {
    pub ranges_rx: mpsc::UnboundedReceiver<RangeInclusive<u64>>,
    pub head_seen_rx: watch::Receiver<u64>,
    pub stop_tx: watch::Sender<bool>,
    pub stop_when_caught_up: bool,
    pub head_offset: u64,
}

fn spawn_peer_feeder(
    pool: Arc<PeerPool>,
    ready_tx: mpsc::UnboundedSender<crate::p2p::NetworkPeer>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut known: HashSet<PeerId> = HashSet::new();

        // Seed any already-connected peers immediately.
        let mut peers = pool.snapshot();
        if peers.len() > 1 {
            let start = PEER_FEEDER_CURSOR.fetch_add(1, Ordering::Relaxed) % peers.len();
            peers.rotate_left(start);
        }
        for peer in peers {
            if known.insert(peer.peer_id) {
                let _ = ready_tx.send(peer);
            }
        }

        let mut ticker = tokio::time::interval(Duration::from_millis(200));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let mut peers = pool.snapshot();
                    if peers.len() > 1 {
                        let start = PEER_FEEDER_CURSOR.fetch_add(1, Ordering::Relaxed) % peers.len();
                        peers.rotate_left(start);
                    }
                    for peer in peers {
                        if known.insert(peer.peer_id) {
                            let _ = ready_tx.send(peer);
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
    })
}

#[derive(Debug)]
pub enum IngestPipelineOutcome {
    UpToDate {
        head: u64,
    },
    RangeApplied {
        range: RangeInclusive<u64>,
        logs: u64,
        finalize: IngestFinalizeStats,
    },
}

pub enum MissingBlocks {
    Precomputed(Vec<u64>),
    Ranges {
        ranges: Vec<RangeInclusive<u64>>,
        total: u64,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IngestFinalizeStats {
    pub drain_workers_ms: u64,
    pub db_flush_ms: u64,
    pub compactions_wait_ms: u64,
    pub compact_all_dirty_ms: u64,
    pub seal_completed_ms: u64,
    pub total_ms: u64,
}

/// Configuration for the ingest pipeline.
pub struct IngestPipelineConfig<'a> {
    pub config: &'a NodeConfig,
    pub range: RangeInclusive<u64>,
    pub head_at_startup: u64,
    pub db_mode_override: Option<DbWriteMode>,
    pub head_cap_override: Option<u64>,
    pub stop_rx: Option<watch::Receiver<bool>>,
    pub tail: Option<TailIngestConfig>,
}

/// Optional tracking/stats for the ingest pipeline.
pub struct IngestPipelineTrackers {
    pub progress: Option<Arc<dyn ProgressReporter>>,
    pub stats: Option<Arc<SyncProgressStats>>,
    pub bench: Option<Arc<IngestBenchStats>>,
    pub events: Option<Arc<BenchEventLogger>>,
}

#[expect(
    clippy::too_many_lines,
    reason = "pipeline orchestrates fetching, processing, and DB writes with complex state machine"
)]
pub async fn run_ingest_pipeline(
    storage: Arc<Storage>,
    pool: Arc<PeerPool>,
    pipeline_config: IngestPipelineConfig<'_>,
    missing: MissingBlocks,
    trackers: IngestPipelineTrackers,
    peer_health: Arc<PeerHealthTracker>,
) -> Result<IngestPipelineOutcome> {
    let IngestPipelineConfig {
        config,
        range,
        head_at_startup,
        db_mode_override,
        head_cap_override,
        stop_rx,
        tail,
    } = pipeline_config;
    let IngestPipelineTrackers {
        progress,
        stats,
        bench,
        events,
    } = trackers;
    let (blocks, mut ranges, total) = match missing {
        MissingBlocks::Precomputed(blocks) => {
            let total = blocks.len() as u64;
            (blocks, Vec::new(), total)
        }
        MissingBlocks::Ranges { ranges, total } => (Vec::new(), ranges, total),
    };
    if total == 0 && tail.is_none() {
        if let Some(stats) = stats {
            stats.set_status(SyncStatus::UpToDate);
            stats.set_queue(0);
            stats.set_inflight(0);
        }
        return Ok(IngestPipelineOutcome::UpToDate {
            head: head_at_startup,
        });
    }

    storage.set_head_seen(head_at_startup)?;

    if let Some(progress) = progress.as_ref() {
        progress.set_length(total);
    }
    if let Some(stats) = stats.as_ref() {
        stats.set_status(SyncStatus::Fetching);
        stats.set_queue(total);
        stats.set_inflight(0);
    }

    let max_blocks = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4))
        .max(1) as usize;
    let initial_blocks = (config.fast_sync_chunk_size.max(1) as usize).min(max_blocks);
    let mut scheduler_config = SchedulerConfig {
        blocks_per_assignment: max_blocks,
        initial_blocks_per_assignment: initial_blocks,
        ..SchedulerConfig::default()
    };
    let mut stop_rx = stop_rx;
    let db_mode = db_mode_override.unwrap_or_else(|| {
        if head_cap_override.is_some() {
            DbWriteMode::Follow
        } else {
            DbWriteMode::FastSync
        }
    });
    // In follow mode, missing blocks are typically due to propagation lag near the tip.
    // Keep retrying indefinitely instead of marking them as permanently failed.
    if db_mode == DbWriteMode::Follow {
        scheduler_config.max_attempts_per_block = u32::MAX;
    }
    let remaining_per_shard = if db_mode == DbWriteMode::FastSync {
        let mut remaining: HashMap<u64, usize> = HashMap::new();
        let shard_size = storage.shard_size();
        if ranges.is_empty() {
            for block in &blocks {
                let shard_start = (block / shard_size) * shard_size;
                *remaining.entry(shard_start).or_insert(0) += 1;
            }
        } else {
            for range in &ranges {
                let shard_start = (range.start() / shard_size) * shard_size;
                let count = range_len(range) as usize;
                *remaining.entry(shard_start).or_insert(0) += count;
            }
        }
        if let Some(stats) = stats.as_ref() {
            stats.set_compactions_total(remaining.len() as u64);
        }
        Some(Arc::new(tokio::sync::Mutex::new(remaining)))
    } else {
        None
    };
    let scheduler = Arc::new(PeerWorkScheduler::new_with_health(
        scheduler_config,
        blocks,
        Arc::clone(&peer_health),
    ));
    if !ranges.is_empty() {
        for range in ranges.drain(..) {
            let _ = scheduler.enqueue_range(range).await;
        }
    }

    let total_blocks = Arc::new(AtomicU64::new(total));
    let max_scheduled = Arc::new(AtomicU64::new(*range.end()));
    let tail_head_seen = tail.as_ref().map(|t| t.head_seen_rx.clone());
    let mut tail_head_seen_rx = tail_head_seen.clone();
    let tail_stop_tx = tail.as_ref().map(|t| t.stop_tx.clone());
    let tail_stop_when_caught_up = tail.as_ref().is_none_or(|t| t.stop_when_caught_up);
    let tail_head_offset = tail.as_ref().map_or(0, |t| t.head_offset);
    let mut tail_stop_sent = false;
    let tail_task = if let Some(mut tail) = tail {
        let scheduler = Arc::clone(&scheduler);
        let remaining_per_shard = remaining_per_shard.clone();
        let progress = progress.clone();
        let stats = stats.clone();
        let bench = bench.clone();
        let total_blocks = Arc::clone(&total_blocks);
        let max_scheduled = Arc::clone(&max_scheduled);
        let shard_size = storage.shard_size();
        Some(tokio::spawn(async move {
            while let Some(range) = tail.ranges_rx.recv().await {
                let start = *range.start();
                let end = *range.end();
                if end < start {
                    continue;
                }
                let added = scheduler.enqueue_range(range.clone()).await;
                if added == 0 {
                    continue;
                }
                let new_total = total_blocks
                    .fetch_add(added as u64, Ordering::SeqCst)
                    .saturating_add(added as u64);
                if let Some(progress) = progress.as_ref() {
                    progress.set_length(new_total);
                }
                if let Some(bench) = bench.as_ref() {
                    bench.add_blocks_total(added as u64);
                }
                if let Some(stats) = stats.as_ref() {
                    let pending = scheduler.pending_count().await as u64;
                    stats.set_queue(pending);
                    stats.set_status(SyncStatus::Fetching);
                }
                let prev_max = max_scheduled.load(Ordering::SeqCst);
                if end > prev_max {
                    max_scheduled.store(end, Ordering::SeqCst);
                }
                if let Some(remaining) = remaining_per_shard.as_ref() {
                    let mut guard = remaining.lock().await;
                    let mut new_shards = 0u64;
                    let mut cursor = start;
                    while cursor <= end {
                        let shard_start = (cursor / shard_size) * shard_size;
                        let shard_end = shard_start.saturating_add(shard_size).saturating_sub(1);
                        let span_end = end.min(shard_end);
                        let count = span_end.saturating_sub(cursor).saturating_add(1) as usize;
                        let entry = guard.entry(shard_start).or_insert(0);
                        if *entry == 0 {
                            new_shards = new_shards.saturating_add(1);
                        }
                        *entry = entry.saturating_add(count);
                        cursor = span_end.saturating_add(1);
                    }
                    drop(guard);
                    if let Some(stats) = stats.as_ref() {
                        if new_shards > 0 {
                            let total = remaining.lock().await.len() as u64;
                            stats.set_compactions_total(total);
                        }
                    }
                }
                tracing::debug!(
                    range_start = start,
                    range_end = end,
                    target_end = end,
                    blocks_added = added,
                    total_blocks = new_total,
                    "tail: appended range"
                );
            }
        }))
    } else {
        None
    };

    let (gauge_stop_tx, gauge_stop_rx) = watch::channel(false);
    let scheduler_gauge_handle = if let Some(events) = events.clone() {
        let scheduler = Arc::clone(&scheduler);
        let mut stop_rx = gauge_stop_rx.clone();
        Some(tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(10));
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let pending_total = scheduler.pending_count().await as u64;
                        let pending_main = scheduler.pending_main_count().await as u64;
                        let inflight = scheduler.inflight_count().await as u64;
                        let completed = scheduler.completed_count().await as u64;
                        let attempts = scheduler.attempts_len().await as u64;
                        let escalation_len = scheduler.escalation_len().await as u64;
                        let escalation_attempted = scheduler.escalation_attempted_count().await as u64;
                        events.record(BenchEvent::SchedulerGaugeSample {
                            pending_total,
                            pending_main,
                            inflight,
                            completed,
                            attempts,
                            escalation_len,
                            escalation_attempted,
                        });
                    }
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        }))
    } else {
        None
    };
    let scheduler_gauge_stop = if scheduler_gauge_handle.is_some() {
        Some(gauge_stop_tx)
    } else {
        None
    };

    // Size the pipeline channels from the configured "max buffered blocks" cap.
    //
    // NOTE: `BenchEvent::FetchEnd` is emitted before enqueueing each payload, so plots derived
    // from events may show a small overshoot above this cap.
    let buffer_cap = config
        .fast_sync_max_buffered_blocks
        .max(1)
        .min(usize::MAX as u64) as usize;
    let (fetched_blocks_tx, fetched_blocks_rx) = mpsc::channel::<BlockPayload>(buffer_cap);
    let (processed_blocks_tx, processed_blocks_rx) = mpsc::channel(buffer_cap);
    let db_flush_tx = processed_blocks_tx.clone();

    let logs_total = Arc::new(AtomicU64::new(0));
    let active_fetch_tasks = Arc::new(AtomicU64::new(0));
    let worker_count = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4)
        .max(1);
    let fetched_rx = Arc::new(Mutex::new(fetched_blocks_rx));
    let mut processor_handles = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let rx = Arc::clone(&fetched_rx);
        let tx = processed_blocks_tx.clone();
        let logs_total = Arc::clone(&logs_total);
        let progress = progress.clone();
        let stats = stats.clone();
        let bench = bench.clone();
        let events = events.clone();
        let handle = tokio::spawn(async move {
            loop {
                let next = {
                    let mut guard = rx.lock().await;
                    guard.recv().await
                };
                let Some(payload) = next else {
                    break;
                };
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::ProcessStart {
                        block: payload.header.number,
                    });
                }
                let process_started = Instant::now();
                let bench_ref = bench.as_ref().map(Arc::as_ref);
                let span = tracing::trace_span!("process_block", block = payload.header.number);
                let result = span.in_scope(|| process_ingest(payload, bench_ref));
                match result {
                    Ok((bundle, log_count)) => {
                        if let Some(events) = events.as_ref() {
                            events.record(BenchEvent::ProcessEnd {
                                block: bundle.number,
                                duration_us: process_started.elapsed().as_micros() as u64,
                                logs: log_count,
                            });
                        }
                        logs_total.fetch_add(log_count, Ordering::SeqCst);
                        if let Some(progress) = progress.as_ref() {
                            progress.inc(1);
                        }
                        if let Some(stats) = stats.as_ref() {
                            stats.inc_processed(1);
                            stats.update_head_block_max(bundle.number);
                        }
                        if let Some(bench) = bench.as_ref() {
                            bench.record_logs(log_count);
                        }
                        if tx.send(DbWriterMessage::Block(bundle)).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        if let Some(bench) = bench.as_ref() {
                            bench.record_process_failure();
                        }
                        tracing::warn!(error = %err, "ingest processing failed");
                    }
                }
            }
        });
        processor_handles.push(handle);
    }
    drop(processed_blocks_tx);

    let db_config = DbWriteConfig::new(config.db_write_batch_blocks, None);
    let follow_expected_start = if db_mode == DbWriteMode::Follow {
        Some(*range.start())
    } else {
        None
    };
    let db_handle = tokio::spawn(run_db_writer(
        Arc::clone(&storage),
        processed_blocks_rx,
        DbWriterParams {
            config: db_config,
            mode: db_mode,
            bench: bench.clone(),
            events: events.clone(),
            progress_stats: stats.clone(),
            remaining_per_shard,
            follow_expected_start,
        },
    ));

    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let peer_events_handle = if let Some(events) = events.clone() {
        let pool = Arc::clone(&pool);
        let peer_health = Arc::clone(&peer_health);
        let mut shutdown_rx = shutdown_rx.clone();
        Some(tokio::spawn(async move {
            let mut known_peers: HashSet<PeerId> = HashSet::new();
            let mut banned_state: HashMap<PeerId, bool> = HashMap::new();
            loop {
                if *shutdown_rx.borrow() {
                    break;
                }
                let snapshot = pool.snapshot();
                let current: HashSet<PeerId> = snapshot.iter().map(|peer| peer.peer_id).collect();
                for added in current.difference(&known_peers) {
                    events.record(BenchEvent::PeerConnected {
                        peer_id: format!("{added:?}"),
                    });
                }
                for removed in known_peers.difference(&current) {
                    events.record(BenchEvent::PeerDisconnected {
                        peer_id: format!("{removed:?}"),
                    });
                }
                known_peers = current;

                let health_snapshot = peer_health.snapshot().await;
                for dump in &health_snapshot {
                    let was_banned = banned_state.get(&dump.peer_id).copied().unwrap_or(false);
                    if dump.is_banned && !was_banned {
                        events.record(BenchEvent::PeerBanned {
                            peer_id: format!("{:?}", dump.peer_id),
                            reason: "health",
                        });
                    } else if !dump.is_banned && was_banned {
                        events.record(BenchEvent::PeerUnbanned {
                            peer_id: format!("{:?}", dump.peer_id),
                        });
                    }
                    banned_state.insert(dump.peer_id, dump.is_banned);
                }

                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                    () = sleep(Duration::from_secs(1)) => {}
                }
            }
        }))
    } else {
        None
    };
    let peer_feeder_handle = spawn_peer_feeder(Arc::clone(&pool), ready_tx.clone(), shutdown_rx);

    let fetch_semaphore = Arc::new(tokio::sync::Semaphore::new(
        config.fast_sync_max_inflight.max(1) as usize,
    ));
    let fetch_timeout = Duration::from_millis(config.fast_sync_batch_timeout_ms.max(1));

    let mut fetch_tasks: JoinSet<()> = JoinSet::new();
    let mut ready_peers: Vec<crate::p2p::NetworkPeer> = Vec::new();
    let mut ready_set: HashSet<PeerId> = HashSet::new();
    loop {
        while let Ok(peer) = ready_rx.try_recv() {
            if ready_set.insert(peer.peer_id) {
                ready_peers.push(peer);
            }
        }
        if ready_peers.is_empty() {
            let peer = tokio::select! {
                peer = ready_rx.recv() => peer,
                () = async {
                    if let Some(stop_rx) = stop_rx.as_mut() {
                        let _ = stop_rx.changed().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => None,
            };
            let Some(peer) = peer else {
                break;
            };
            if ready_set.insert(peer.peer_id) {
                ready_peers.push(peer);
            }
            continue;
        }
        while fetch_tasks.try_join_next().is_some() {}
        if stop_rx.as_ref().is_some_and(|rx| *rx.borrow()) {
            break;
        }
        if scheduler.is_done().await {
            if let Some(head_seen_rx) = tail_head_seen_rx.as_mut() {
                let head_seen = *head_seen_rx.borrow();
                let safe_head = head_seen.saturating_sub(tail_head_offset);
                let scheduled = max_scheduled.load(Ordering::SeqCst);
                if scheduled >= safe_head {
                    if let Some(stop_tx) = tail_stop_tx.as_ref() {
                        if !tail_stop_sent && tail_stop_when_caught_up {
                            let _ = stop_tx.send(true);
                            tail_stop_sent = true;
                            continue;
                        }
                    }
                    if tail_stop_when_caught_up {
                        break;
                    }
                    if let Some(stats) = stats.as_ref() {
                        if db_mode == DbWriteMode::Follow {
                            stats.set_status(SyncStatus::Following);
                            stats.set_queue(0);
                            stats.set_inflight(0);
                        }
                    }
                    // Follow-style tailing: wait for head updates and continue.
                    tokio::select! {
                        () = async {
                            if let Some(stop_rx) = stop_rx.as_mut() {
                                let _ = stop_rx.changed().await;
                            } else {
                                std::future::pending::<()>().await;
                            }
                        } => {
                            if stop_rx.as_ref().is_some_and(|rx| *rx.borrow()) {
                                break;
                            }
                        }
                        _ = head_seen_rx.changed() => {}
                        () = sleep(Duration::from_millis(200)) => {}
                    }
                    continue;
                }
            } else {
                break;
            }
        }

        let active_peers = pool.len();
        let Ok(permit) = fetch_semaphore.clone().try_acquire_owned() else {
            sleep(Duration::from_millis(10)).await;
            continue;
        };

        let best_idx = pick_best_ready_peer_index(&ready_peers, peer_health.as_ref()).await;
        let peer = ready_peers.swap_remove(best_idx);
        ready_set.remove(&peer.peer_id);
        let head_cap = match head_cap_override {
            Some(cap) => cap,
            None => {
                if db_mode == DbWriteMode::Follow {
                    // NOTE: `NetworkPeer.head_number` is probed once at connect time and can go
                    // stale. In follow mode we cap scheduling at the global observed head.
                    tail_head_seen
                        .as_ref()
                        .map_or(*range.end(), |rx| *rx.borrow())
                } else if peer.head_number == 0 {
                    *range.end()
                } else {
                    peer.head_number
                }
            }
        };
        let batch = scheduler
            .next_batch_for_peer(peer.peer_id, head_cap, active_peers)
            .await;
        if batch.blocks.is_empty() {
            drop(permit);
            let ready_tx = ready_tx.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                let _ = ready_tx.send(peer);
            });
            continue;
        }

        let batch_limit = peer_health.batch_limit(peer.peer_id).await as u64;
        if let Some(events) = events.as_ref() {
            let range_start = batch.blocks.first().copied().unwrap_or(0);
            let range_end = batch.blocks.last().copied().unwrap_or(range_start);
            let mode = match batch.mode {
                FetchMode::Normal => "normal",
                FetchMode::Escalation => "escalation",
            };
            events.record(BenchEvent::BatchAssigned {
                peer_id: format!("{:?}", peer.peer_id),
                range_start,
                range_end,
                blocks: batch.blocks.len() as u64,
                batch_limit,
                mode,
            });
        }

        peer_health
            .record_assignment(peer.peer_id, batch.blocks.len())
            .await;

        let active_now = active_fetch_tasks
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);
        if let Some(stats) = stats.as_ref() {
            stats.set_peers_active(active_now);
        }

        let ctx = FetchTaskContext {
            scheduler: Arc::clone(&scheduler),
            peer_health: Arc::clone(&peer_health),
            fetched_tx: fetched_blocks_tx.clone(),
            ready_tx: ready_tx.clone(),
            stats: stats.clone(),
            bench: bench.clone(),
            events: events.clone(),
            active_fetch_tasks: Arc::clone(&active_fetch_tasks),
            db_mode,
            fetch_timeout,
        };
        let params = FetchTaskParams {
            peer,
            blocks: batch.blocks,
            mode: batch.mode,
            batch_limit,
            permit,
        };
        fetch_tasks.spawn(run_fetch_task(ctx, params));
    }

    while fetch_tasks.join_next().await.is_some() {}
    let finalize_started = Instant::now();
    if let Some(stats) = stats.as_ref() {
        // Reset compaction counters BEFORE setting status to Finalizing,
        // so UI never sees stale values from the sync phase.
        stats.set_compactions_done(0);
        stats.set_compactions_total(0);
        // Fetching is done; remaining work is local processing/DB flush.
        // Set status BEFORE fetch_complete to avoid race condition where UI sees
        // fetch_complete=true but status is still Fetching.
        stats.set_status(SyncStatus::Finalizing);
        stats.set_fetch_complete(true);
        let snapshot = stats.snapshot();
        tracing::info!(
            processed = snapshot.processed,
            queue = snapshot.queue,
            inflight = snapshot.inflight,
            escalation = snapshot.escalation,
            peers_active = snapshot.peers_active,
            peers_total = snapshot.peers_total,
            "finalizing: fetch complete, draining workers and flushing DB"
        );
    } else {
        tracing::info!("finalizing: fetch complete, draining workers and flushing DB");
    }
    let _ = shutdown_tx.send(true);
    tracing::info!("finalizing: stopping peer feeder");
    let _ = peer_feeder_handle.await;
    if let Some(handle) = peer_events_handle {
        let _ = handle.await;
    }
    drop(fetched_blocks_tx);
    tracing::info!(
        workers = processor_handles.len(),
        "finalizing: draining processor workers"
    );
    let processors_started = Instant::now();
    for handle in processor_handles {
        let _ = handle.await;
    }
    let drain_workers_ms = processors_started.elapsed().as_millis() as u64;
    tracing::info!(
        elapsed_ms = drain_workers_ms,
        "finalizing: processor workers drained"
    );
    tracing::info!("finalizing: flushing DB writer");
    let db_flush_started = Instant::now();
    let _ = db_flush_tx.send(DbWriterMessage::Finalize).await;
    drop(db_flush_tx);
    let db_result = db_handle.await.map_err(|err| eyre!(err))?;
    let db_finalize_stats = db_result?;
    let db_flush_ms = db_flush_started.elapsed().as_millis() as u64;
    tracing::info!(elapsed_ms = db_flush_ms, "finalizing: DB writer flushed");
    let total_ms = finalize_started.elapsed().as_millis() as u64;
    tracing::info!(
        elapsed_ms = total_ms,
        compactions_wait_ms = db_finalize_stats.compactions_wait_ms,
        compact_all_dirty_ms = db_finalize_stats.compact_all_dirty_ms,
        seal_completed_ms = db_finalize_stats.seal_completed_ms,
        "finalizing: complete"
    );

    let logs = logs_total.load(Ordering::SeqCst);
    if let Some(stop_tx) = scheduler_gauge_stop {
        let _ = stop_tx.send(true);
    }
    if let Some(handle) = scheduler_gauge_handle {
        let _ = handle.await;
    }
    if let Some(handle) = tail_task {
        handle.abort();
        let _ = handle.await;
    }

    Ok(IngestPipelineOutcome::RangeApplied {
        range,
        logs,
        finalize: IngestFinalizeStats {
            drain_workers_ms,
            db_flush_ms,
            compactions_wait_ms: db_finalize_stats.compactions_wait_ms,
            compact_all_dirty_ms: db_finalize_stats.compact_all_dirty_ms,
            seal_completed_ms: db_finalize_stats.seal_completed_ms,
            total_ms,
        },
    })
}

fn range_len(range: &RangeInclusive<u64>) -> u64 {
    let start = *range.start();
    let end = *range.end();
    if end >= start {
        end - start + 1
    } else {
        0
    }
}

pub(super) fn fetch_payload_bytes(payload: &BlockPayload) -> FetchByteTotals {
    let headers = payload.header.length() as u64;
    let bodies = payload.body.length() as u64;
    let receipts = payload.receipts.length() as u64;
    let logs = payload
        .receipts
        .iter()
        .map(|receipt| receipt.logs.length() as u64)
        .sum();
    FetchByteTotals {
        headers,
        bodies,
        receipts,
        logs,
    }
}
