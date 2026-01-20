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
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinSet;
use tokio::time::{sleep, timeout, Duration};

use fetch::{fetch_ingest_batch, fetch_probe_batch, FetchIngestOutcome, FetchProbeOutcome};
use db_writer::{DbWriteConfig, DbWriterMessage, run_db_writer};
use process::process_ingest;
use scheduler::{PeerHealthConfig, PeerWorkScheduler, SchedulerConfig};
pub(crate) use scheduler::PeerHealthTracker;
use sink::run_probe_sink;
pub use stats::{IngestBenchStats, ProbeStats};
pub use follow::run_follow_loop;
use stats::FetchByteTotals;
use types::{BenchmarkConfig, ProbeRecord, FetchMode};
use crate::storage::Storage;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

const WARMUP_SECS: u64 = 3;
static PEER_FEEDER_CURSOR: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn build_peer_health_tracker(config: &NodeConfig) -> Arc<PeerHealthTracker> {
    let scheduler_config = SchedulerConfig {
        blocks_per_assignment: config.fast_sync_chunk_size.max(1) as usize,
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
    let scheduler_config = SchedulerConfig {
        blocks_per_assignment: benchmark.blocks_per_assignment,
        ..SchedulerConfig::default()
    };
    let peer_health = build_peer_health_tracker(config);
    let scheduler = Arc::new(PeerWorkScheduler::new_with_health(
        scheduler_config,
        blocks,
        peer_health,
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
    let mut warmed_up = false;
    while let Some(peer) = ready_rx.recv().await {
        if !warmed_up {
            if WARMUP_SECS > 0 {
                sleep(Duration::from_secs(WARMUP_SECS)).await;
            }
            warmed_up = true;
        }
        while fetch_tasks.try_join_next().is_some() {}
        if scheduler.is_done().await {
            break;
        }
        let active_peers = pool.len();
        let batch = scheduler
            .next_batch_for_peer(peer.peer_id, peer.head_number, active_peers)
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
                            format!("{:.0}s", remaining / speed)
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
                                format!("{:.0}s", remaining as f64 / esc_speed)
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
) -> Result<IngestPipelineOutcome> {
    let total = range_len(&range);
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

    let blocks: Vec<u64> = range.clone().collect();
    let mut scheduler_config = SchedulerConfig {
        blocks_per_assignment: config.fast_sync_chunk_size.max(1) as usize,
        ..SchedulerConfig::default()
    };
    // In follow mode, missing blocks are typically due to propagation lag near the tip.
    // Keep retrying indefinitely instead of marking them as permanently failed.
    if head_cap_override.is_some() {
        scheduler_config.max_attempts_per_block = u32::MAX;
    }
    let scheduler = Arc::new(PeerWorkScheduler::new_with_health(
        scheduler_config,
        blocks,
        Arc::clone(&peer_health),
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
        let handle = tokio::spawn(async move {
            loop {
                let next = {
                    let mut guard = rx.lock().await;
                    guard.recv().await
                };
                let Some(payload) = next else {
                    break;
                };
                let bench_ref = bench.as_ref().map(Arc::as_ref);
                let result = process_ingest(payload, bench_ref);
                match result {
                    Ok((bundle, log_count)) => {
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
    let db_handle = tokio::spawn(run_db_writer(
        Arc::clone(&storage),
        processed_blocks_rx,
        db_config,
        bench.clone(),
        *range.start(),
    ));

    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
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
    while let Some(peer) = ready_rx.recv().await {
        while fetch_tasks.try_join_next().is_some() {}
        if scheduler.is_done().await {
            break;
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

        let active_peers = pool.len();
        let head_cap = head_cap_override.unwrap_or(peer.head_number);
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

        peer_health
            .record_assignment(peer.peer_id, batch.blocks.len())
            .await;

        let scheduler = Arc::clone(&scheduler);
        let peer_health = Arc::clone(&peer_health);
        let fetched_tx = fetched_blocks_tx.clone();
        let ready_tx = ready_tx.clone();
        let stats = stats.clone();
        let bench = bench.clone();
        let assigned_blocks = batch.blocks.len();
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
            let result =
                match timeout(fetch_timeout, fetch_ingest_batch(&peer, &batch.blocks)).await {
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
                    } = outcome;
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
                            let mut bytes = FetchByteTotals::default();
                            for payload in &payloads {
                                bytes.add(fetch_payload_bytes(payload));
                            }
                            bench.record_fetch_bytes(bytes);
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
    if let Some(stats) = stats.as_ref() {
        // Fetching is done; remaining work is local processing/DB flush.
        stats.set_status(SyncStatus::Finalizing);
    }
    let _ = shutdown_tx.send(true);
    let _ = peer_feeder_handle.await;
    drop(fetched_blocks_tx);
    for handle in processor_handles {
        let _ = handle.await;
    }
    let _ = db_flush_tx.send(DbWriterMessage::Flush).await;
    drop(db_flush_tx);
    let db_result = db_handle.await.map_err(|err| eyre!(err))?;
    db_result?;

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
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            end_block: None,
            rollback_window: 0,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
            verbosity: 0,
            benchmark: BenchmarkMode::Probe,
            command: None,
            rpc_max_request_body_bytes: DEFAULT_RPC_MAX_REQUEST_BODY_BYTES,
            rpc_max_response_body_bytes: DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES,
            rpc_max_connections: DEFAULT_RPC_MAX_CONNECTIONS,
            rpc_max_batch_requests: DEFAULT_RPC_MAX_BATCH_REQUESTS,
            rpc_max_blocks_per_filter: DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
            rpc_max_logs_per_response: DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
            fast_sync_chunk_size: DEFAULT_FAST_SYNC_CHUNK_SIZE,
            fast_sync_max_inflight: DEFAULT_FAST_SYNC_MAX_INFLIGHT,
            fast_sync_batch_timeout_ms: DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS,
            fast_sync_max_buffered_blocks: DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS,
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
