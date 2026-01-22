//! Historical sync orchestration (benchmark/probe and future ingest).

mod db_writer;
mod fetch;
mod follow;
mod process;
mod reorg;
mod scheduler;
mod sink;
mod stats;
mod types;

use crate::cli::NodeConfig;
use crate::p2p::PeerPool;
use crate::sync::{BlockPayload, ProgressReporter, SyncProgressStats, SyncStatus};
use alloy_rlp::Encodable;
use eyre::{eyre, Result};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use reth_network_api::PeerId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::RangeInclusive;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout, Duration};
use tracing::Instrument;

use fetch::{fetch_ingest_batch, fetch_probe_batch, FetchIngestOutcome, FetchProbeOutcome};
use db_writer::{DbWriteConfig, DbWriteMode, DbWriterMessage, run_db_writer};
use process::process_ingest;
use scheduler::{PeerHealthConfig, PeerWorkScheduler, SchedulerConfig};
pub(crate) use scheduler::PeerHealthTracker;
use sink::run_probe_sink;
pub use stats::{BenchEventLogger, IngestBenchStats, IngestBenchSummary, ProbeStats};
pub use follow::run_follow_loop;
use stats::{BenchEvent, FetchByteTotals};
use types::{BenchmarkConfig, ProbeRecord, FetchMode};
use crate::storage::Storage;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

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

/// Run benchmark probe mode over a fixed range.
pub async fn run_benchmark_probe(
    config: &NodeConfig,
    pool: Arc<PeerPool>,
    range: RangeInclusive<u64>,
    head_at_startup: u64,
    stats: Arc<ProbeStats>,
) -> Result<()> {
    let mut benchmark = BenchmarkConfig::default();
    benchmark.blocks_per_assignment = config.fast_sync_chunk_size.max(1) as usize;
    let blocks_total = range_len(&range);
    if blocks_total == 0 {
        return Err(eyre!("benchmark range is empty"));
    }

    let blocks: Vec<u64> = range.clone().collect();
    let start_block = *range.start();
    let low_watermark = Arc::new(AtomicU64::new(start_block));
    let max_blocks = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4))
        .max(1) as usize;
    let initial_blocks = benchmark.blocks_per_assignment.max(1).min(max_blocks);
    let scheduler_config = SchedulerConfig {
        blocks_per_assignment: max_blocks,
        initial_blocks_per_assignment: initial_blocks,
        ..SchedulerConfig::default()
    };
    let peer_health = build_peer_health_tracker(config);
    let peer_health_for_select = Arc::clone(&peer_health);
    let scheduler = Arc::new(PeerWorkScheduler::new_with_health(
        scheduler_config,
        blocks,
        peer_health,
        low_watermark,
    ));

    let buffer_cap = config
        .fast_sync_max_buffered_blocks
        .max(1)
        .min(usize::MAX as u64) as usize;
    let (processed_blocks_tx, processed_blocks_rx) = mpsc::channel::<ProbeRecord>(buffer_cap);

    let stats_for_sink = Arc::clone(&stats);
    let sink_handle = tokio::spawn(run_probe_sink(processed_blocks_rx, stats_for_sink));

    let (report_stop_tx, report_stop_rx) = watch::channel(false);
    let peer_feeder_rx = report_stop_rx.clone();
    let reporter_handle = spawn_probe_progress_bar(
        blocks_total,
        Arc::clone(&stats),
        Arc::clone(&scheduler),
        Arc::clone(&pool),
        report_stop_rx,
        benchmark.report_interval,
    );

    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();
    let peer_feeder_handle = spawn_peer_feeder(
        Arc::clone(&pool),
        ready_tx.clone(),
        peer_feeder_rx,
    );

    let fetch_semaphore = Arc::new(tokio::sync::Semaphore::new(
        config.fast_sync_max_inflight.max(1) as usize,
    ));
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
            let Some(peer) = ready_rx.recv().await else {
                break;
            };
            if ready_set.insert(peer.peer_id) {
                ready_peers.push(peer);
            }
            continue;
        }
        while fetch_tasks.try_join_next().is_some() {}
        if scheduler.is_done().await {
            break;
        }
        let active_peers = pool.len();
        let best_idx =
            pick_best_ready_peer_index(&ready_peers, peer_health_for_select.as_ref()).await;
        let peer = ready_peers.swap_remove(best_idx);
        ready_set.remove(&peer.peer_id);
        let head_cap = if peer.head_number == 0 {
            *range.end()
        } else {
            peer.head_number
        };
        let batch = scheduler
            .next_batch_for_peer(peer.peer_id, head_cap, active_peers)
            .await;
        if batch.blocks.is_empty() {
            let ready_tx = ready_tx.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                let _ = ready_tx.send(peer);
            });
            continue;
        }

        let permit = match fetch_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                let ready_tx = ready_tx.clone();
                tokio::spawn(async move {
                    sleep(Duration::from_millis(50)).await;
                    let _ = ready_tx.send(peer);
                });
                continue;
            }
        };

        let scheduler = Arc::clone(&scheduler);
        let records_tx = processed_blocks_tx.clone();
        let ready_tx = ready_tx.clone();
        let stats = Arc::clone(&stats);
        let receipts_per_request = benchmark.receipts_per_request;

        fetch_tasks.spawn(async move {
            let _permit = permit;
            let result: Result<FetchProbeOutcome> =
                fetch_probe_batch(&peer, &batch.blocks, receipts_per_request).await;
            match result {
                Ok(outcome) => {
                    let completed: Vec<u64> =
                        outcome.records.iter().map(|record| record.number).collect();
                    if !completed.is_empty() {
                        let recovered = scheduler.mark_completed(&completed).await;
                        if recovered > 0 {
                            stats.record_block_recovered(recovered);
                        }
                        for record in outcome.records {
                            if records_tx.send(record).await.is_err() {
                                break;
                            }
                        }
                    }
                    if !outcome.missing_blocks.is_empty() {
                        // Record as partial response - peer returned something but not everything
                        scheduler.record_peer_partial(peer.peer_id).await;
                        match batch.mode {
                            FetchMode::Normal => {
                                let newly_failed =
                                    scheduler.requeue_failed(&outcome.missing_blocks).await;
                                for _ in newly_failed {
                                    stats.record_block_failure();
                                }
                            }
                            FetchMode::Escalation => {
                                for block in &outcome.missing_blocks {
                                    scheduler
                                        .requeue_escalation_block(*block, active_peers)
                                        .await;
                                }
                            }
                        }
                        tracing::debug!(
                            peer_id = ?peer.peer_id,
                            missing = outcome.missing_blocks.len(),
                            "probe batch missing receipts or headers"
                        );
                    } else {
                        // Full success - reset failure counters
                        scheduler.record_peer_success(peer.peer_id).await;
                    }
                }
                Err(err) => {
                    scheduler.record_peer_failure(peer.peer_id).await;
                    match batch.mode {
                        FetchMode::Normal => {
                            let newly_failed = scheduler.requeue_failed(&batch.blocks).await;
                            for _ in newly_failed {
                                stats.record_block_failure();
                            }
                        }
                        FetchMode::Escalation => {
                            for block in &batch.blocks {
                                scheduler
                                    .requeue_escalation_block(*block, active_peers)
                                    .await;
                            }
                        }
                    }
                    stats.record_peer_failure();
                    tracing::debug!(
                        peer_id = ?peer.peer_id,
                        error = %err,
                        "probe batch failed"
                    );
                }
            }
            let _ = ready_tx.send(peer);
        });
    }

    while fetch_tasks.join_next().await.is_some() {}
    drop(processed_blocks_tx);
    let _ = sink_handle.await;

    let _ = report_stop_tx.send(true);
    let _ = reporter_handle.await;
    let _ = peer_feeder_handle.await;

    let rollback_applied = config.rollback_window > 0;
    let peers_used = pool.len() as u64;
    let summary = stats.summary(
        *range.start(),
        *range.end(),
        head_at_startup,
        rollback_applied,
        peers_used,
    );
    let summary_json = serde_json::to_string_pretty(&summary)?;
    println!("{summary_json}");

    Ok(())
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

fn spawn_probe_progress_bar(
    total: u64,
    stats: Arc<ProbeStats>,
    scheduler: Arc<PeerWorkScheduler>,
    pool: Arc<PeerPool>,
    mut shutdown_rx: watch::Receiver<bool>,
    update_interval: Duration,
) -> tokio::task::JoinHandle<()> {
    let pb = ProgressBar::with_draw_target(
        Some(total),
        ProgressDrawTarget::stderr_with_hz(10),
    );
    let style = ProgressStyle::with_template(
        "{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}",
    )
    .expect("progress style")
    .progress_chars("█▉░");
    pb.set_style(style);
    pb.set_message(
        "status looking_for_peers | peers 0 | queue 0 | inflight 0 | Failed 0 | speed 0.0/s | eta --",
    );
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(update_interval);
        let mut main_finished = false;
        let mut escalation_total = 0u64;
        let mut escalation_pb: Option<ProgressBar> = None;
        let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
        let mut escalation_window: VecDeque<(Instant, u64)> = VecDeque::new();
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let completed = stats.blocks_succeeded();
                    let failed = stats.blocks_failed();
                    let processed = completed + failed;
                    let queue_main_len = scheduler.pending_main_count().await as u64;
                    let inflight = scheduler.inflight_count().await as u64;
                    let escalation_len = scheduler.escalation_len().await as u64;
                    let peers = pool.len() as u64;
                    let escalation_active = escalation_len > 0;

                    let status = if peers == 0 {
                        "looking_for_peers"
                    } else if escalation_active {
                        "escalating"
                    } else if processed >= total && queue_main_len == 0 && inflight == 0 {
                        "finalizing"
                    } else {
                        "fetching"
                    };

                    if !main_finished {
                        let now = Instant::now();
                        window.push_back((now, processed));
                        while let Some((t, _)) = window.front() {
                            if now.duration_since(*t) > Duration::from_secs(1) && window.len() > 1 {
                                window.pop_front();
                            } else {
                                break;
                            }
                        }
                        let speed = if let (Some((t0, v0)), Some((t1, v1))) =
                            (window.front(), window.back())
                        {
                            let dt = t1.duration_since(*t0).as_secs_f64();
                            if dt > 0.0 && v1 >= v0 {
                                (v1 - v0) as f64 / dt
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };
                        let remaining = total.saturating_sub(processed) as f64;
                        let eta = if speed > 0.0 {
                            crate::sync::format_eta_seconds(remaining / speed)
                        } else {
                            "--".to_string()
                        };

                        let msg = format!(
                            "status {} | peers {} | queue {} | inflight {} | Failed {} | speed {:.1}/s | eta {}",
                            status, peers, queue_main_len, inflight, failed, speed, eta
                        );
                        pb.set_message(msg);
                        pb.set_position(processed.min(total));
                    }

                    if !main_finished && processed >= total && queue_main_len == 0 && inflight == 0 {
                        main_finished = true;
                        pb.finish_and_clear();
                    }

                    if main_finished {
                        if failed > escalation_total {
                            escalation_total = failed;
                            if let Some(ref esc_pb) = escalation_pb {
                                esc_pb.set_length(escalation_total);
                            }
                        }
                        if escalation_total > 0 {
                            if escalation_pb.is_none() {
                                let esc_pb = ProgressBar::with_draw_target(
                                    Some(escalation_total),
                                    ProgressDrawTarget::stderr_with_hz(10),
                                );
                                let esc_style = ProgressStyle::with_template(
                                    "{bar:40.red/black} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}",
                                )
                                .expect("progress style")
                                .progress_chars("▓▒░");
                                esc_pb.set_style(esc_style);
                                esc_pb.set_message(
                                    "status escalating_failed | failed 0/0 | tried 0/0 | peers 0 | inflight 0 | queue 0 | speed 0.0/s | eta --",
                                );
                                escalation_pb = Some(esc_pb);
                            }

                            let remaining = failed;
                            let done = escalation_total.saturating_sub(remaining);
                            let now = Instant::now();
                            escalation_window.push_back((now, done));
                            while let Some((t, _)) = escalation_window.front() {
                                if now.duration_since(*t) > Duration::from_secs(1)
                                    && escalation_window.len() > 1
                                {
                                    escalation_window.pop_front();
                                } else {
                                    break;
                                }
                            }
                            let esc_speed =
                                if let (Some((t0, v0)), Some((t1, v1))) = (
                                    escalation_window.front(),
                                    escalation_window.back(),
                                ) {
                                    let dt = t1.duration_since(*t0).as_secs_f64();
                                    if dt > 0.0 && v1 >= v0 {
                                        (v1 - v0) as f64 / dt
                                    } else {
                                        0.0
                                    }
                                } else {
                                    0.0
                                };
                            let esc_eta = if esc_speed > 0.0 {
                                crate::sync::format_eta_seconds(remaining as f64 / esc_speed)
                            } else {
                                "--".to_string()
                            };
                            let attempted = scheduler.escalation_attempted_count().await as u64;
                            if let Some(ref esc_pb) = escalation_pb {
                                let msg = format!(
                                    "failed {remaining}/{escalation_total} | tried {attempted}/{escalation_total} | peers {peers} | inflight {inflight} | queue {escalation_len} | speed {:.1}/s | eta {esc_eta}",
                                    esc_speed
                                );
                                esc_pb.set_message(msg);
                                esc_pb.set_position(done);
                            }
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    break;
                }
            }
        }
        if !main_finished {
            pb.finish_and_clear();
        }
        if let Some(pb) = escalation_pb {
            pb.finish_and_clear();
        }
    })
}

#[derive(Debug)]
pub enum IngestPipelineOutcome {
    UpToDate { head: u64 },
    RangeApplied { range: RangeInclusive<u64>, logs: u64 },
}

pub async fn run_ingest_pipeline(
    storage: Arc<Storage>,
    pool: Arc<PeerPool>,
    config: &NodeConfig,
    range: RangeInclusive<u64>,
    head_at_startup: u64,
    progress: Option<Arc<dyn ProgressReporter>>,
    stats: Option<Arc<SyncProgressStats>>,
    bench: Option<Arc<IngestBenchStats>>,
    head_cap_override: Option<u64>,
    peer_health: Arc<PeerHealthTracker>,
    events: Option<Arc<BenchEventLogger>>,
) -> Result<IngestPipelineOutcome> {
    let blocks = storage.missing_blocks_in_range(range.clone())?;
    let total = blocks.len() as u64;
    if total == 0 {
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

    let start_block = *range.start();
    let low_watermark = Arc::new(AtomicU64::new(start_block));
    let max_blocks = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4))
        .max(1) as usize;
    let initial_blocks = (config.fast_sync_chunk_size.max(1) as usize).min(max_blocks);
    let mut scheduler_config = SchedulerConfig {
        blocks_per_assignment: max_blocks,
        initial_blocks_per_assignment: initial_blocks,
        max_lookahead_blocks: config.fast_sync_max_lookahead_blocks,
        ..SchedulerConfig::default()
    };
    // In follow mode, missing blocks are typically due to propagation lag near the tip.
    // Keep retrying indefinitely instead of marking them as permanently failed.
    if head_cap_override.is_some() {
        scheduler_config.max_attempts_per_block = u32::MAX;
    }
    let scheduler = Arc::new(PeerWorkScheduler::new_with_health(
        scheduler_config,
        blocks.clone(),
        Arc::clone(&peer_health),
        Arc::clone(&low_watermark),
    ));

    let (fetched_blocks_tx, fetched_blocks_rx) = mpsc::channel::<BlockPayload>(2048);
    let (processed_blocks_tx, processed_blocks_rx) = mpsc::channel(2048);
    let db_flush_tx = processed_blocks_tx.clone();

    let logs_total = Arc::new(AtomicU64::new(0));
    let active_fetch_tasks = Arc::new(AtomicU64::new(0));
    let worker_count = std::thread::available_parallelism()
        .map(|count| count.get())
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

    let db_config = DbWriteConfig::new(
        config.db_write_batch_blocks,
        None,
    );
    let db_mode = if head_cap_override.is_some() {
        DbWriteMode::Follow
    } else {
        DbWriteMode::FastSync
    };
    let remaining_per_shard = if db_mode == DbWriteMode::FastSync {
        let mut remaining: HashMap<u64, usize> = HashMap::new();
        let shard_size = storage.shard_size();
        for block in &blocks {
            let shard_start = (block / shard_size) * shard_size;
            *remaining.entry(shard_start).or_insert(0) += 1;
        }
        Some(Arc::new(tokio::sync::Mutex::new(remaining)))
    } else {
        None
    };
    let db_handle = tokio::spawn(run_db_writer(
        Arc::clone(&storage),
        processed_blocks_rx,
        db_config,
        bench.clone(),
        events.clone(),
        db_mode,
        remaining_per_shard,
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
                let current: HashSet<PeerId> =
                    snapshot.iter().map(|peer| peer.peer_id).collect();
                for added in current.difference(&known_peers) {
                    events.record(BenchEvent::PeerConnected {
                        peer_id: format!("{:?}", added),
                    });
                }
                for removed in known_peers.difference(&current) {
                    events.record(BenchEvent::PeerDisconnected {
                        peer_id: format!("{:?}", removed),
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
                    _ = sleep(Duration::from_secs(1)) => {}
                }
            }
        }))
    } else {
        None
    };
    let peer_feeder_handle = spawn_peer_feeder(
        Arc::clone(&pool),
        ready_tx.clone(),
        shutdown_rx,
    );

    let fetch_semaphore = Arc::new(tokio::sync::Semaphore::new(
        config.fast_sync_max_inflight.max(1) as usize,
    ));
    let fetch_timeout =
        Duration::from_millis(config.fast_sync_batch_timeout_ms.max(1));

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
            let Some(peer) = ready_rx.recv().await else {
                break;
            };
            if ready_set.insert(peer.peer_id) {
                ready_peers.push(peer);
            }
            continue;
        }
        while fetch_tasks.try_join_next().is_some() {}
        if scheduler.is_done().await {
            break;
        }

        let active_peers = pool.len();
        let permit = match fetch_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                sleep(Duration::from_millis(10)).await;
                continue;
            }
        };

        let best_idx = pick_best_ready_peer_index(&ready_peers, peer_health.as_ref()).await;
        let peer = ready_peers.swap_remove(best_idx);
        ready_set.remove(&peer.peer_id);
        let head_cap = match head_cap_override {
            Some(cap) => cap,
            None => {
                if peer.head_number == 0 {
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

        let scheduler = Arc::clone(&scheduler);
        let peer_health = Arc::clone(&peer_health);
        let fetched_tx = fetched_blocks_tx.clone();
        let ready_tx = ready_tx.clone();
        let stats = stats.clone();
        let bench = bench.clone();
        let events = events.clone();
        let assigned_blocks = batch.blocks.len();
        let batch_limit = batch_limit;
        let fetch_timeout = fetch_timeout;
        let active_fetch_tasks = Arc::clone(&active_fetch_tasks);

        let active_now = active_fetch_tasks.fetch_add(1, Ordering::SeqCst).saturating_add(1);
        if let Some(stats) = stats.as_ref() {
            stats.set_peers_active(active_now);
        }

        fetch_tasks.spawn(async move {
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

            let _permit = permit;
            let _active_guard = ActiveFetchTaskGuard {
                counter: Arc::clone(&active_fetch_tasks),
                stats: stats.clone(),
            };
            let fetch_started = Instant::now();
            if let Some(events) = events.as_ref() {
                let range_start = batch.blocks.first().copied().unwrap_or(0);
                let range_end = batch.blocks.last().copied().unwrap_or(range_start);
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
                range_start = batch.blocks.first().copied().unwrap_or(0),
                range_end = batch.blocks.last().copied().unwrap_or(0),
                blocks = assigned_blocks,
                batch_limit = batch_limit
            );
            let fetch_future = fetch_ingest_batch(&peer, &batch.blocks).instrument(span);
            let result = match timeout(fetch_timeout, fetch_future).await {
                    Ok(result) => result,
                    Err(_) => {
                        tracing::debug!(
                            peer_id = ?peer.peer_id,
                            blocks = assigned_blocks,
                            range_start = batch.blocks.first().copied(),
                            range_end = batch.blocks.last().copied(),
                            timeout_ms = fetch_timeout.as_millis() as u64,
                            "ingest batch timed out"
                        );
                        if let Some(events) = events.as_ref() {
                            let range_start = batch.blocks.first().copied().unwrap_or(0);
                            let range_end = batch.blocks.last().copied().unwrap_or(range_start);
                            events.record(BenchEvent::FetchTimeout {
                                peer_id: format!("{:?}", peer.peer_id),
                                range_start,
                                range_end,
                                blocks: assigned_blocks as u64,
                                batch_limit,
                                timeout_ms: fetch_timeout.as_millis() as u64,
                            });
                        }
                        Err(eyre!(
                            "ingest batch to {:?} timed out after {:?}",
                            peer.peer_id,
                            fetch_timeout
                        ))
                    }
                };
            let fetch_elapsed = fetch_started.elapsed();
            match result {
                Ok(outcome) => {
                    let FetchIngestOutcome {
                        payloads,
                        missing_blocks,
                        fetch_stats,
                    } = outcome;
                    let bytes = if bench.is_some() || events.is_some() {
                        let mut bytes = FetchByteTotals::default();
                        for payload in &payloads {
                            bytes.add(fetch_payload_bytes(payload));
                        }
                        Some(bytes)
                    } else {
                        None
                    };
                    if let Some(events) = events.as_ref() {
                        let range_start = batch.blocks.first().copied().unwrap_or(0);
                        let range_end = batch.blocks.last().copied().unwrap_or(range_start);
                        let bytes_headers = bytes.map(|b| b.headers).unwrap_or(0);
                        let bytes_bodies = bytes.map(|b| b.bodies).unwrap_or(0);
                        let bytes_receipts = bytes.map(|b| b.receipts).unwrap_or(0);
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
                        let recovered = scheduler.mark_completed(&completed).await;
                        if recovered > 0 {
                            if let Some(stats) = stats.as_ref() {
                                stats.record_block_recovered(recovered);
                            }
                        }
                    }

                    if let Some(bench) = bench.as_ref() {
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
                        if fetched_tx.send(payload).await.is_err() {
                            break;
                        }
                    }

                    if missing_blocks.is_empty() {
                        scheduler.record_peer_success(peer.peer_id).await;
                    } else {
                        peer_health
                            .note_error(
                                peer.peer_id,
                                format!("missing {} blocks in batch", missing_blocks.len()),
                            )
                            .await;
                        let empty_response = completed.is_empty();
                        if empty_response {
                            scheduler.record_peer_failure(peer.peer_id).await;
                        } else {
                            scheduler.record_peer_partial(peer.peer_id).await;
                        }
                        if let Some(bench) = bench.as_ref() {
                            bench.record_fetch_failure(missing_blocks.len() as u64, fetch_elapsed);
                            if empty_response {
                                bench.record_peer_failure();
                            }
                        }
                        match batch.mode {
                            FetchMode::Normal => {
                                let newly_failed =
                                    scheduler.requeue_failed(&missing_blocks).await;
                                if let Some(stats) = stats.as_ref() {
                                    stats.inc_failed(newly_failed.len() as u64);
                                }
                            }
                            FetchMode::Escalation => {
                                for block in &missing_blocks {
                                    scheduler
                                        .requeue_escalation_block(*block, active_peers)
                                        .await;
                                }
                            }
                        }
                        tracing::debug!(
                            peer_id = ?peer.peer_id,
                            missing = missing_blocks.len(),
                            "ingest batch missing headers or payloads"
                        );
                    }
                }
                Err(err) => {
                    peer_health
                        .note_error(peer.peer_id, format!("ingest error: {err}"))
                        .await;
                    scheduler.record_peer_failure(peer.peer_id).await;
                    if let Some(bench) = bench.as_ref() {
                        bench.record_fetch_failure(batch.blocks.len() as u64, fetch_elapsed);
                        bench.record_peer_failure();
                    }
                    match batch.mode {
                        FetchMode::Normal => {
                            let _ = scheduler.requeue_failed(&batch.blocks).await;
                        }
                        FetchMode::Escalation => {
                            for block in &batch.blocks {
                                scheduler
                                    .requeue_escalation_block(*block, active_peers)
                                    .await;
                            }
                        }
                    }
                    tracing::debug!(peer_id = ?peer.peer_id, error = %err, "ingest batch failed");
                }
            }
            peer_health.finish_assignment(peer.peer_id, assigned_blocks).await;
            if let Some(stats) = stats.as_ref() {
                let pending = scheduler.pending_count().await as u64;
                let inflight = scheduler.inflight_count().await as u64;
                stats.set_queue(pending);
                stats.set_inflight(inflight);
            }
            let _ = ready_tx.send(peer);
        });
    }

    while fetch_tasks.join_next().await.is_some() {}
    let finalize_started = Instant::now();
    if let Some(stats) = stats.as_ref() {
        // Fetching is done; remaining work is local processing/DB flush.
        stats.set_status(SyncStatus::Finalizing);
        let snapshot = stats.snapshot();
        tracing::info!(
            processed = snapshot.processed,
            queue = snapshot.queue,
            inflight = snapshot.inflight,
            failed = snapshot.failed,
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
    tracing::info!(workers = processor_handles.len(), "finalizing: draining processor workers");
    let processors_started = Instant::now();
    for handle in processor_handles {
        let _ = handle.await;
    }
    tracing::info!(
        elapsed_ms = processors_started.elapsed().as_millis() as u64,
        "finalizing: processor workers drained"
    );
    tracing::info!("finalizing: flushing DB writer");
    let db_flush_started = Instant::now();
    let _ = db_flush_tx.send(DbWriterMessage::Finalize).await;
    drop(db_flush_tx);
    let db_result = db_handle.await.map_err(|err| eyre!(err))?;
    db_result?;
    tracing::info!(
        elapsed_ms = db_flush_started.elapsed().as_millis() as u64,
        "finalizing: DB writer flushed"
    );
    tracing::info!(
        elapsed_ms = finalize_started.elapsed().as_millis() as u64,
        "finalizing: complete"
    );

    let logs = logs_total.load(Ordering::SeqCst);
    Ok(IngestPipelineOutcome::RangeApplied { range, logs })
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

fn fetch_payload_bytes(payload: &BlockPayload) -> FetchByteTotals {
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

#[cfg(test)]
mod tests {
    use super::{run_benchmark_probe, ProbeStats};
    use crate::cli::{BenchmarkMode, HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
    use crate::p2p::{peer_pool_for_tests, NetworkPeer};
    use crate::cli::{
        DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS, DEFAULT_FAST_SYNC_CHUNK_SIZE,
        DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS, DEFAULT_FAST_SYNC_MAX_INFLIGHT,
        DEFAULT_RPC_MAX_BATCH_REQUESTS,
        DEFAULT_RPC_MAX_BLOCKS_PER_FILTER, DEFAULT_RPC_MAX_CONNECTIONS,
        DEFAULT_RPC_MAX_LOGS_PER_RESPONSE, DEFAULT_RPC_MAX_REQUEST_BODY_BYTES,
        DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES, DEFAULT_DB_WRITE_BATCH_BLOCKS,
    };
    use alloy_consensus::ReceiptWithBloom;
    use reth_eth_wire::EthVersion;
    use reth_eth_wire_types::{BlockHeaders, GetReceipts, Receipts};
    use reth_network_api::{PeerId, PeerRequest, PeerRequestSender};
    use reth_primitives_traits::Header;
    use reth_ethereum_primitives::{Receipt, TxType};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    fn base_config(data_dir: PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir,
            peer_cache_dir: None,
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            shard_size: crate::cli::DEFAULT_SHARD_SIZE,
            end_block: None,
            rollback_window: 0,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
            verbosity: 0,
            benchmark: BenchmarkMode::Probe,
            benchmark_name: None,
            benchmark_output_dir: PathBuf::from(crate::cli::DEFAULT_BENCHMARK_OUTPUT_DIR),
            benchmark_trace: false,
            benchmark_events: false,
            benchmark_min_peers: None,
            command: None,
            rpc_max_request_body_bytes: DEFAULT_RPC_MAX_REQUEST_BODY_BYTES,
            rpc_max_response_body_bytes: DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES,
            rpc_max_connections: DEFAULT_RPC_MAX_CONNECTIONS,
            rpc_max_batch_requests: DEFAULT_RPC_MAX_BATCH_REQUESTS,
            rpc_max_blocks_per_filter: DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
            rpc_max_logs_per_response: DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
            fast_sync_chunk_size: DEFAULT_FAST_SYNC_CHUNK_SIZE,
            fast_sync_chunk_max: None,
            fast_sync_max_inflight: DEFAULT_FAST_SYNC_MAX_INFLIGHT,
            fast_sync_batch_timeout_ms: DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS,
            fast_sync_max_buffered_blocks: DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS,
            fast_sync_max_lookahead_blocks: crate::cli::DEFAULT_FAST_SYNC_MAX_LOOKAHEAD_BLOCKS,
            db_write_batch_blocks: DEFAULT_DB_WRITE_BATCH_BLOCKS,
            db_write_flush_interval_ms: None,
        }
    }

    fn dummy_receipt() -> Receipt {
        Receipt {
            tx_type: TxType::Legacy,
            success: true,
            cumulative_gas_used: 0,
            logs: Vec::new(),
        }
    }

    async fn run_probe_with_test_peer(config: &NodeConfig) {
        let peer_id = PeerId::random();
        let (tx, mut rx) = mpsc::channel(8);
        let messages = PeerRequestSender::new(peer_id, tx);
        let peer = NetworkPeer {
            peer_id,
            eth_version: EthVersion::Eth68,
            messages,
            head_number: 1,
        };

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                match request {
                    PeerRequest::GetBlockHeaders { request, response } => {
                        let start = match request.start_block {
                            reth_eth_wire_types::BlockHashOrNumber::Number(num) => num,
                            _ => 0,
                        };
                        let count = request.limit as u64;
                        let headers = (0..count)
                            .map(|offset| {
                                let mut header = Header::default();
                                header.number = start + offset;
                                header
                            })
                            .collect::<Vec<_>>();
                        let _ = response.send(Ok(BlockHeaders::from(headers)));
                    }
                    PeerRequest::GetReceipts { request, response } => {
                        let GetReceipts(hashes) = request;
                        let receipts = hashes
                            .iter()
                            .map(|_| vec![ReceiptWithBloom::new(dummy_receipt(), Default::default())])
                            .collect::<Vec<_>>();
                        let _ = response.send(Ok(Receipts(receipts)));
                    }
                    _ => {}
                }
            }
        });

        let pool = Arc::new(peer_pool_for_tests(vec![peer]));
        let range = 0u64..=1u64;
        let head_at_startup = 1;

        let blocks_total = (*range.end())
            .saturating_sub(*range.start())
            .saturating_add(1);
        let stats = Arc::new(ProbeStats::new(blocks_total));
        run_benchmark_probe(config, pool, range, head_at_startup, stats)
            .await
            .expect("probe run");
    }

    #[tokio::test]
    async fn probe_mode_does_not_create_db_files() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let temp_dir = std::env::temp_dir()
            .join(format!("stateless-history-node-probe-test-{now}-{}", std::process::id()));
        let config = base_config(temp_dir.clone());

        run_probe_with_test_peer(&config).await;

        assert!(!config.data_dir.join("meta.json").exists());
        assert!(!config.data_dir.join("static").exists());
    }

    #[tokio::test]
    async fn probe_mode_does_not_update_checkpoints() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let temp_dir = std::env::temp_dir()
            .join(format!("stateless-history-node-probe-checkpoints-{now}-{}", std::process::id()));
        let config = base_config(temp_dir.clone());

        let storage = crate::storage::Storage::open(&config).expect("open storage");
        let before = storage.last_indexed_block().expect("last indexed");
        drop(storage);

        run_probe_with_test_peer(&config).await;

        let storage = crate::storage::Storage::open(&config).expect("open storage");
        let after = storage.last_indexed_block().expect("last indexed");
        assert_eq!(before, after);
    }
}
