mod cli;
mod logging;
mod metrics;
mod p2p;
mod rpc;
mod storage;
mod sync;
mod ui;
#[cfg(test)]
mod test_utils;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use cli::{compute_target_range, NodeConfig};
use eyre::Result;
use indicatif::ProgressBar;
use logging::{
    finalize_log_files, generate_run_report, init_tracing, run_timestamp_utc, spawn_resource_logger,
    RunContext,
};
#[cfg(unix)]
use logging::spawn_usr1_state_logger;
use metrics::range_len;
use p2p::PeerPool;
use reth_network_api::PeerId;
use std::{
    env, fs,
    io::IsTerminal,
    path::PathBuf,
    process,
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, SystemTime},
};
use sync::historical::{BenchEventLogger, IngestBenchStats, PeerHealthTracker};
use sync::{ProgressReporter, SyncProgressStats};
#[cfg(test)]
use sync::SyncStatus;
use tracing::{info, warn};

async fn apply_cached_peer_limits(storage: &storage::Storage, peer_health: &PeerHealthTracker) {
    let peers = storage.peer_cache_snapshot();
    for peer in peers {
        let Some(limit) = peer.aimd_batch_limit else {
            continue;
        };
        let Ok(peer_id) = PeerId::from_str(&peer.peer_id) else {
            continue;
        };
        peer_health.set_batch_limit(peer_id, limit).await;
    }
}

async fn persist_peer_limits(storage: &storage::Storage, peer_health: &PeerHealthTracker) {
    let snapshot = peer_health.snapshot().await;
    let limits: Vec<(String, u64)> = snapshot
        .into_iter()
        .map(|entry| (entry.peer_id.to_string(), entry.batch_limit as u64))
        .collect();
    storage.update_peer_batch_limits(&limits);
}

async fn flush_peer_cache_with_limits(
    session: &p2p::NetworkSession,
    storage: &storage::Storage,
    peer_health: Option<&PeerHealthTracker>,
) {
    if let Err(err) = session.flush_peer_cache() {
        warn!(error = %err, "failed to flush peer cache");
        return;
    }
    if let Some(peer_health) = peer_health {
        persist_peer_limits(storage, peer_health).await;
        if let Err(err) = storage.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache with limits");
        }
    }
}

impl sync::ProgressReporter for ProgressBar {
    fn set_length(&self, len: u64) {
        self.set_length(len);
    }

    fn inc(&self, delta: u64) {
        self.inc(delta);
    }
}

struct IngestProgress {
    bar: ProgressBar,
    total_len: AtomicU64,
}

impl IngestProgress {
    fn new(bar: ProgressBar, total_len: u64) -> Self {
        bar.set_length(total_len);
        Self {
            bar,
            total_len: AtomicU64::new(total_len),
        }
    }

    fn finish(&self) {
        self.bar.finish_and_clear();
    }
}

impl sync::ProgressReporter for IngestProgress {
    fn set_length(&self, len: u64) {
        let current = self.total_len.load(Ordering::SeqCst);
        if len > current {
            self.total_len.store(len, Ordering::SeqCst);
            self.bar.set_length(len);
        }
    }

    fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }
}

fn default_peer_cache_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".stateless-history-node"))
}

async fn wait_for_min_peers(pool: &Arc<PeerPool>, min_peers: usize) {
    if min_peers == 0 {
        return;
    }
    loop {
        let peers = pool.len();
        if peers >= min_peers {
            ui::clear_status_bar();
            return;
        }
        ui::print_status_bar(&format!("Waiting for peers... {}/{}", peers, min_peers));
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn wait_for_peer_head(
    pool: &Arc<PeerPool>,
    start_block: u64,
    end_block: Option<u64>,
    rollback_window: u64,
) -> u64 {
    let mut last_log = Instant::now() - Duration::from_secs(10);
    loop {
        let peers = pool.snapshot();
        let best_head = peers.iter().map(|peer| peer.head_number).max().unwrap_or(0);
        let safe_head = best_head.saturating_sub(rollback_window);
        let requested_end = end_block.unwrap_or(safe_head);
        let end = requested_end.min(safe_head);
        if start_block <= end {
            ui::clear_status_bar();
            return best_head;
        }
        ui::print_status_bar(&format!(
            "Discovering chain head... {} | peers {}",
            best_head,
            peers.len()
        ));
        if last_log.elapsed() >= Duration::from_secs(5) {
            info!(
                peers = peers.len(),
                best_head,
                safe_head,
                start_block,
                requested_end,
                "waiting for peer head to reach start block"
            );
            last_log = Instant::now();
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = NodeConfig::from_args();
    let argv: Vec<String> = env::args().collect();
    // Check if any log artifacts are enabled
    let log_artifacts_enabled = config.log_trace
        || config.log_events
        || config.log_json
        || config.log_report
        || config.log_resources;
    let chunk_max = config
        .fast_sync_chunk_max
        .unwrap_or(config.fast_sync_chunk_size.saturating_mul(4))
        .max(1);
    if chunk_max < config.fast_sync_chunk_size.max(1) {
        warn!(
            initial = config.fast_sync_chunk_size,
            max = chunk_max,
            "fast-sync-chunk-max is below fast-sync-chunk-size; clamping initial to max"
        );
        config.fast_sync_chunk_size = chunk_max;
    }
    config.fast_sync_chunk_max = Some(chunk_max);
    let run_context = if log_artifacts_enabled {
        let timestamp_utc = run_timestamp_utc(SystemTime::now());
        let run_id = format!("{}-{}", timestamp_utc, process::id());
        let run_name = config
            .run_name
            .clone()
            .unwrap_or_else(|| "sync".to_string());
        let output_dir = config.log_output_dir.clone();
        let trace_tmp_path = if config.log_trace {
            Some(output_dir.join(format!("{}.trace.json.part", timestamp_utc)))
        } else {
            None
        };
        let events_tmp_path = if config.log_events {
            Some(output_dir.join(format!("{}.events.jsonl.part", timestamp_utc)))
        } else {
            None
        };
        let logs_tmp_path = if config.log_json {
            Some(output_dir.join(format!("{}.logs.jsonl.part", timestamp_utc)))
        } else {
            None
        };
        let resources_tmp_path = if config.log_resources {
            Some(output_dir.join(format!("{}.resources.jsonl.part", timestamp_utc)))
        } else {
            None
        };
        fs::create_dir_all(&output_dir)?;
        Some(RunContext {
            timestamp_utc,
            run_id,
            run_name,
            output_dir,
            argv,
            trace_tmp_path,
            events_tmp_path,
            logs_tmp_path,
            resources_tmp_path,
        })
    } else {
        None
    };
    if config.peer_cache_dir.is_none() {
        config.peer_cache_dir = default_peer_cache_dir();
    }
    let mut tracing_guards = init_tracing(
        &config,
        run_context
            .as_ref()
            .and_then(|ctx| ctx.trace_tmp_path.clone()),
        run_context
            .as_ref()
            .and_then(|ctx| ctx.logs_tmp_path.clone()),
        run_context
            .as_ref()
            .and_then(|ctx| ctx.resources_tmp_path.clone()),
    );

    if let Some(command) = &config.command {
        match command {
            cli::Command::Db(cli::DbCommand::Stats(args)) => {
                let data_dir = args
                    .data_dir
                    .clone()
                    .unwrap_or_else(|| config.data_dir.clone());
                let stats = storage::Storage::disk_usage_at(&data_dir)?;
                ui::print_db_stats(&data_dir, &stats, args.json)?;
                return Ok(());
            }
        }
    }

    // Handle --repair flag
    if config.repair {
        println!("Checking storage integrity...\n");
        let report = storage::Storage::repair(&config)?;

        if report.shards.is_empty() {
            println!("No shards found in {}", config.data_dir.display());
            return Ok(());
        }

        for info in &report.shards {
            if info.result.needs_recovery() {
                let phase_info = info
                    .original_phase
                    .as_ref()
                    .map(|p| format!(" (phase: {})", p))
                    .unwrap_or_default();
                println!(
                    "Shard {}: {}{}",
                    info.shard_start,
                    info.result.description(),
                    phase_info
                );
            } else {
                println!("Shard {}: OK", info.shard_start);
            }
        }

        println!(
            "\nRepair complete. {}/{} shards repaired, {} already clean.",
            report.repaired_count(),
            report.total_count(),
            report.clean_count()
        );
        return Ok(());
    }

    // Main ingest path
    info!(
        chain_id = config.chain_id,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );
    if !matches!(config.head_source, cli::HeadSource::P2p) {
        return Err(eyre::eyre!("ingest requires --head-source p2p"));
    }

    ui::print_status_bar("Opening storage...");
    let storage = Arc::new(storage::Storage::open(&config)?);

    // Resume behavior: if the previous run exited before compaction finished, ensure we
    // compact any completed shards up-front so we don't keep reprocessing WAL-heavy shards
    // across restarts.
    let dirty_shards = storage.dirty_complete_shards()?;
    if !dirty_shards.is_empty() {
        let shard_count = dirty_shards.len();
        ui::print_status_bar(&format!("Compacting {} shards...", shard_count));
        info!(shard_count, "startup: compacting completed dirty shards");
        let storage_clone = Arc::clone(&storage);
        tokio::task::spawn_blocking(move || -> Result<()> {
            for shard_start in dirty_shards {
                storage_clone.compact_shard(shard_start)?;
            }
            Ok(())
        })
        .await??;
        info!(shard_count, "startup: completed shard compaction done");
    }

    ui::print_status_bar("Connecting to P2P network...");
    info!("starting p2p network");
    let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
    let initial_peers = session.pool.len();
    info!(peers = initial_peers, "p2p peers connected");
    ui::print_status_bar(&format!("P2P connected | {} peers", initial_peers));
    let min_peers = config.min_peers as usize;
    wait_for_min_peers(&session.pool, min_peers).await;
    info!(
        peers = session.pool.len(),
        min_peers, "peer warmup complete"
    );
    let follow_after = config.end_block.is_none();

    let head_at_startup = wait_for_peer_head(
        &session.pool,
        config.start_block,
        config.end_block,
        config.rollback_window,
    )
    .await;
    let range = compute_target_range(
        config.start_block,
        config.end_block,
        head_at_startup,
        config.rollback_window,
    );
    let run_ctx = run_context.as_ref();
    let missing_started = Instant::now();
    let (missing_ranges, missing_total) = storage.missing_ranges_in_range(range.clone())?;
    let missing_elapsed_ms = missing_started.elapsed().as_millis() as u64;
    let requested_total = range_len(&range);
    let already_present = requested_total.saturating_sub(missing_total);
    if already_present > 0 {
        info!(
            range_start = *range.start(),
            range_end = *range.end(),
            requested_total,
            already_present,
            missing_total,
            elapsed_ms = missing_elapsed_ms,
            "resume: found existing blocks in range; skipping already present blocks"
        );
    } else {
        info!(
            range_start = *range.start(),
            range_end = *range.end(),
            requested_total,
            missing_total,
            elapsed_ms = missing_elapsed_ms,
            "resume: no existing blocks in range"
        );
    }
    let total_len = missing_total;
    let blocks = sync::historical::MissingBlocks::Ranges {
        ranges: missing_ranges,
        total: missing_total,
    };

    let progress_stats = if std::io::stderr().is_terminal() {
        Some(Arc::new(SyncProgressStats::default()))
    } else {
        None
    };
    if let Some(stats) = progress_stats.as_ref() {
        stats.set_head_seen(head_at_startup);
    }
    let events = if config.log_events {
        let run_ctx = run_ctx.expect("run context required for log_events");
        let tmp_path = run_ctx.events_tmp_path.clone().expect("events tmp path");
        Some(Arc::new(BenchEventLogger::new(tmp_path)?))
    } else {
        None
    };
    spawn_resource_logger(progress_stats.clone(), events.clone());
    let peer_health_local = sync::historical::build_peer_health_tracker(&config);
    apply_cached_peer_limits(&storage, &peer_health_local).await;
    #[cfg(unix)]
    spawn_usr1_state_logger(
        progress_stats.clone(),
        Some(Arc::clone(&session.pool)),
        Some(Arc::clone(&peer_health_local)),
    );
    // Create UI controller and progress bar
    let is_tty = std::io::stderr().is_terminal();
    let ui_controller = Arc::new(Mutex::new(ui::UIController::new(is_tty)));
    let progress: Option<Arc<IngestProgress>> = if is_tty {
        // Initialize syncing state with progress bar
        {
            let mut ui = ui_controller.lock().expect("ui lock");
            ui.show_syncing(total_len);
        }
        if let Some(stats) = progress_stats.as_ref() {
            stats.set_peers_active(0);
            stats.set_peers_total(session.pool.len() as u64);
            ui::spawn_progress_updater(
                Arc::clone(&ui_controller),
                Arc::clone(stats),
                Arc::clone(&session.pool),
                Some(Arc::clone(&peer_health_local)),
            );
        }
        // Get the bar from the controller for IngestProgress
        let bar = ui_controller
            .lock()
            .expect("ui lock")
            .current_bar()
            .cloned()
            .expect("sync bar should exist");
        Some(Arc::new(IngestProgress::new(bar, total_len)))
    } else {
        None
    };

    let mut head_tracker_handle = None;
    let mut tail_feeder_handle = None;
    let mut head_stop_tx = None;
    let mut tail_stop_tx = None;
    let mut tail_config = None;
    if follow_after {
        let (head_stop, head_stop_rx) = tokio::sync::watch::channel(false);
        let (tail_stop, tail_stop_rx) = tokio::sync::watch::channel(false);
        head_stop_tx = Some(head_stop.clone());
        tail_stop_tx = Some(tail_stop.clone());

        let (head_seen_tx, head_seen_rx) = tokio::sync::watch::channel(head_at_startup);
        let pool = Arc::clone(&session.pool);
        let storage = Arc::clone(&storage);
        let stats = progress_stats.clone();
        let rollback_window = config.rollback_window;
        let mut stop_rx_head = head_stop_rx.clone();
        head_tracker_handle = Some(tokio::spawn(async move {
            let mut last_head = head_at_startup;
            loop {
                if *stop_rx_head.borrow() {
                    break;
                }
                let snapshot = pool.snapshot();
                let best_head = snapshot
                    .iter()
                    .map(|peer| peer.head_number)
                    .max()
                    .unwrap_or(last_head);
                if best_head > last_head {
                    last_head = best_head;
                    let _ = head_seen_tx.send(best_head);
                    if let Err(err) = storage.set_head_seen(best_head) {
                        tracing::debug!(error = %err, "head tracker: failed to persist head_seen");
                    }
                    if let Some(stats) = stats.as_ref() {
                        stats.set_head_seen(best_head);
                    }
                }
                tokio::select! {
                    _ = stop_rx_head.changed() => {
                        if *stop_rx_head.borrow() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {}
                }
            }
        }));

        let (tail_tx, tail_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut stop_rx_tail = tail_stop_rx.clone();
        let mut head_seen_rx_tail = head_seen_rx.clone();
        let mut next_to_schedule = range.end().saturating_add(1);
        tail_feeder_handle = Some(tokio::spawn(async move {
            loop {
                if *stop_rx_tail.borrow() {
                    break;
                }
                let head_seen = *head_seen_rx_tail.borrow();
                let safe_head = head_seen.saturating_sub(rollback_window);
                if safe_head >= next_to_schedule {
                    let start = next_to_schedule;
                    let end = safe_head;
                    let _ = tail_tx.send(start..=end);
                    next_to_schedule = end.saturating_add(1);
                }
                tokio::select! {
                    _ = head_seen_rx_tail.changed() => {}
                    _ = stop_rx_tail.changed() => {
                        if *stop_rx_tail.borrow() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                }
            }
        }));

        tail_config = Some(sync::historical::TailIngestConfig {
            ranges_rx: tail_rx,
            head_seen_rx,
            stop_tx: tail_stop,
            stop_when_caught_up: true,
            head_offset: config.rollback_window,
        });
    }

    let bench = Arc::new(IngestBenchStats::new(total_len));
    let progress_ref = progress
        .as_ref()
        .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let head_stop_tx_for_shutdown = head_stop_tx.clone();
    let tail_stop_tx_for_shutdown = tail_stop_tx.clone();
    let shutdown_requested = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_flag = Arc::clone(&shutdown_requested);
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);
            warn!("shutdown signal received; stopping ingest after draining");
            let _ = stop_tx.send(true);
            if let Some(stop_tx) = head_stop_tx_for_shutdown {
                let _ = stop_tx.send(true);
            }
            if let Some(stop_tx) = tail_stop_tx_for_shutdown {
                let _ = stop_tx.send(true);
            }
            if tokio::signal::ctrl_c().await.is_ok() {
                warn!("second shutdown signal received; forcing exit");
                process::exit(130);
            }
        }
    });
    let outcome = sync::historical::run_ingest_pipeline(
        Arc::clone(&storage),
        Arc::clone(&session.pool),
        &config,
        range.clone(),
        blocks,
        head_at_startup,
        progress_ref,
        progress_stats.clone(),
        Some(Arc::clone(&bench)),
        None,
        None,
        Arc::clone(&peer_health_local),
        events.clone(),
        Some(stop_rx),
        tail_config,
    )
    .await?;

    let finalize_stats = match &outcome {
        sync::historical::IngestPipelineOutcome::RangeApplied { finalize, .. } => Some(*finalize),
        _ => None,
    };
    if follow_after {
        let last_indexed = storage.last_indexed_block().ok().flatten();
        let head_seen = storage.head_seen().ok().flatten();
        let safe_head = head_seen.map(|head| head.saturating_sub(config.rollback_window));
        let dirty = storage.dirty_shards().unwrap_or_default();
        let total_wal_bytes: u64 = dirty.iter().map(|info| info.wal_bytes).sum();
        let total_wal_mib = (total_wal_bytes as f64 / (1024.0 * 1024.0)).round();
        tracing::info!(
            follow_after,
            shutdown_requested = shutdown_requested.load(std::sync::atomic::Ordering::SeqCst),
            switch_tip = ?last_indexed,
            head_seen = ?head_seen,
            safe_head = ?safe_head,
            dirty_shards = dirty.len(),
            total_wal_mib,
            drain_workers_ms = finalize_stats.map(|s| s.drain_workers_ms),
            db_flush_ms = finalize_stats.map(|s| s.db_flush_ms),
            compactions_wait_ms = finalize_stats.map(|s| s.compactions_wait_ms),
            compact_all_dirty_ms = finalize_stats.map(|s| s.compact_all_dirty_ms),
            seal_completed_ms = finalize_stats.map(|s| s.seal_completed_ms),
            total_finalize_ms = finalize_stats.map(|s| s.total_ms),
            "fast-sync -> follow boundary"
        );
    }

    if follow_after && !shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(stop_tx) = tail_stop_tx.take() {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = tail_feeder_handle.take() {
            let _ = handle.await;
        }

        info!("fast-sync complete; switching to follow mode");

        // Generate report at fast-sync completion (saved if --log-report, printed if -v).
        // Log files continue into follow mode - they'll be finalized on exit.
        let logs_total = match &outcome {
            sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => *logs,
            sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
        };
        let storage_stats = match storage.disk_usage() {
            Ok(stats) => Some(stats),
            Err(err) => {
                warn!(error = %err, "failed to collect storage disk stats");
                None
            }
        };
        let summary = bench.summary(
            *range.start(),
            *range.end(),
            head_at_startup,
            config.rollback_window > 0,
            session.pool.len() as u64,
            logs_total,
            storage_stats,
        );
        let report_base_name = if let Some(run_ctx) = run_context.as_ref() {
            match generate_run_report(
                run_ctx,
                &config,
                &range,
                head_at_startup,
                &peer_health_local,
                &summary,
                session.pool.len() as u64,
            )
            .await
            {
                Ok(name) => Some(name),
                Err(err) => {
                    warn!(error = %err, "failed to generate run report");
                    None
                }
            }
        } else {
            None
        };

        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let (synced_tx, synced_rx) = tokio::sync::oneshot::channel();
        let follow_future = sync::historical::run_follow_loop(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            progress_ref,
            progress_stats.clone(),
            Arc::clone(&peer_health_local),
            Some(synced_tx),
        );
        tokio::pin!(follow_future);

        let mut rpc_handle = None;
        let mut synced_rx = Some(synced_rx);
        loop {
            tokio::select! {
                res = &mut follow_future => {
                    if let Err(err) = res {
                        warn!(error = %err, "follow loop exited");
                    }
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    warn!("shutdown signal received");
                    break;
                }
                _ = async {
                    if let Some(rx) = synced_rx.take() {
                        let _ = rx.await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    if rpc_handle.is_none() {
                        let handle = rpc::start(
                            config.rpc_bind,
                            rpc::RpcConfig::from(&config),
                            Arc::clone(&storage),
                        )
                        .await?;
                        info!(rpc_bind = %config.rpc_bind, "rpc server started");
                        rpc_handle = Some(handle);
                    }
                }
            }
        }

        if let Some(tracker) = progress.as_ref() {
            tracker.finish();
        }
        if let Some(handle) = rpc_handle.take() {
            if let Err(err) = handle.stop() {
                warn!(error = %err, "failed to stop rpc server");
            }
            handle.stopped().await;
        }
        if let Some(stop_tx) = head_stop_tx.take() {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = head_tracker_handle.take() {
            let _ = handle.await;
        }
        // Finalize log files on exit from follow mode (report already generated)
        if let Some(run_ctx) = run_context.as_ref() {
            if let Some(base_name) = report_base_name.as_ref() {
                finalize_log_files(
                    run_ctx,
                    base_name,
                    events.as_deref(),
                    tracing_guards.log_writer.as_deref(),
                    tracing_guards.resources_writer.as_deref(),
                    &mut tracing_guards.chrome_guard,
                );
            }
        }
        flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
        return Ok(());
    }

    if shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(stop_tx) = tail_stop_tx.take() {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = tail_feeder_handle.take() {
            let _ = handle.await;
        }
        if let Some(stop_tx) = head_stop_tx.take() {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = head_tracker_handle.take() {
            let _ = handle.await;
        }
        if let Some(tracker) = progress.as_ref() {
            tracker.finish();
        }
        // Generate report and finalize logs on Ctrl-C during fast-sync
        if let Some(run_ctx) = run_context.as_ref() {
            let logs_total = match &outcome {
                sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => *logs,
                sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
            };
            let storage_stats = match storage.disk_usage() {
                Ok(stats) => Some(stats),
                Err(err) => {
                    warn!(error = %err, "failed to collect storage disk stats");
                    None
                }
            };
            let summary = bench.summary(
                *range.start(),
                *range.end(),
                head_at_startup,
                config.rollback_window > 0,
                session.pool.len() as u64,
                logs_total,
                storage_stats,
            );
            let base_name = match generate_run_report(
                run_ctx,
                &config,
                &range,
                head_at_startup,
                &peer_health_local,
                &summary,
                session.pool.len() as u64,
            )
            .await
            {
                Ok(name) => name,
                Err(err) => {
                    warn!(error = %err, "failed to generate run report");
                    // Fallback base name
                    run_ctx.timestamp_utc.clone()
                }
            };
            finalize_log_files(
                run_ctx,
                &base_name,
                events.as_deref(),
                tracing_guards.log_writer.as_deref(),
                tracing_guards.resources_writer.as_deref(),
                &mut tracing_guards.chrome_guard,
            );
        }
        flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
        return Ok(());
    }

    if let Some(tracker) = progress.as_ref() {
        tracker.finish();
    }

    let logs_total = match outcome {
        sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => logs,
        sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
    };
    let storage_stats = match storage.disk_usage() {
        Ok(stats) => Some(stats),
        Err(err) => {
            warn!(error = %err, "failed to collect storage disk stats");
            None
        }
    };
    let summary = bench.summary(
        *range.start(),
        *range.end(),
        head_at_startup,
        config.rollback_window > 0,
        session.pool.len() as u64,
        logs_total,
        storage_stats,
    );
    // Generate report (if --log-report) and finalize log files.
    if let Some(run_ctx) = run_ctx {
        let base_name = match generate_run_report(
            run_ctx,
            &config,
            &range,
            head_at_startup,
            &peer_health_local,
            &summary,
            session.pool.len() as u64,
        )
        .await
        {
            Ok(name) => name,
            Err(err) => {
                warn!(error = %err, "failed to generate run report");
                // Fallback base name
                run_ctx.timestamp_utc.clone()
            }
        };
        finalize_log_files(
            run_ctx,
            &base_name,
            events.as_deref(),
            tracing_guards.log_writer.as_deref(),
            tracing_guards.resources_writer.as_deref(),
            &mut tracing_guards.chrome_guard,
        );
    }
    flush_peer_cache_with_limits(&session, &storage, Some(&peer_health_local)).await;
    Ok(())
}

#[cfg(test)]
fn total_blocks_to_head(start_from: u64, head: u64) -> u64 {
    if head >= start_from {
        head.saturating_sub(start_from).saturating_add(1)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::{total_blocks_to_head, ui, SyncStatus};

    #[test]
    fn total_blocks_handles_empty_range() {
        assert_eq!(total_blocks_to_head(10, 9), 0);
        assert_eq!(total_blocks_to_head(10, 10), 1);
        assert_eq!(total_blocks_to_head(10, 12), 3);
    }

    #[test]
    fn progress_message_formats_status() {
        assert_eq!(
            ui::format_progress_message(SyncStatus::Fetching, 2, 5, 10, 1, 0, 0, 0, 1.5, "12s"),
            "status fetching | peers 2/5 | queue 10 | inflight 1 | retry 0 | speed 1.5/s | eta 12s"
        );
    }
}
