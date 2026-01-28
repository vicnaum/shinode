//! Main sync orchestration.

// SIGINT handler must terminate the process on second signal
#![expect(clippy::exit, reason = "SIGINT handler must terminate process")]

use crate::cli::{compute_target_range, HeadSource, NodeConfig};
use crate::logging::{init_tracing, spawn_resource_logger, TracingGuards};
#[cfg(unix)]
use crate::logging::spawn_usr1_state_logger;
use crate::metrics::range_len;
use crate::p2p;
use crate::rpc;
use crate::storage::Storage;
use crate::sync::historical::{
    build_peer_health_tracker, run_follow_loop, run_ingest_pipeline, BenchEventLogger,
    IngestBenchStats, IngestPipelineConfig, IngestPipelineOutcome, IngestPipelineTrackers,
    MissingBlocks, PeerHealthTracker, SummaryInput,
};
use crate::sync::{ProgressReporter, SyncProgressStats};
use eyre::Result;
use std::env;
use std::io::IsTerminal;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{info, warn};

use super::cleanup::{apply_cached_peer_limits, finalize_session, flush_peer_cache_with_limits, FinalizeContext};
use super::session::{FollowModeResources, IngestProgress};
use super::startup::{
    build_run_context, connect_p2p, init_storage, setup_ui, wait_for_min_peers, wait_for_peer_head,
    EarlyTui, UiSetup,
};
use crate::ui::tui::TuiController;
use parking_lot::Mutex;
use super::trackers::{build_tail_config, spawn_head_tracker, spawn_tail_feeder};

/// Default peer cache directory.
fn default_peer_cache_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".stateless-history-node"))
}

/// Run the main sync process.
#[expect(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    reason = "main sync orchestration has sequential setup phases that are clearer inline"
)]
pub async fn run_sync(mut config: NodeConfig, argv: Vec<String>) -> Result<()> {
    // Number of coverage buckets for TUI blocks map visualization
    const COVERAGE_BUCKETS: usize = 200;

    // Validate and normalize chunk sizes
    let chunk_max = config
        .fast_sync_chunk_max
        .unwrap_or_else(|| config.fast_sync_chunk_size.saturating_mul(4))
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

    // Build run context for log artifacts
    let run_context = build_run_context(&config, argv);

    // Set default peer cache dir
    if config.peer_cache_dir.is_none() {
        config.peer_cache_dir = default_peer_cache_dir();
    }

    // Determine TUI mode early for tracing init
    let is_tty = std::io::stderr().is_terminal();
    let use_tui = !config.no_tui && is_tty;

    // Initialize tracing (suppress stdout fmt_layer in TUI mode)
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
        use_tui,
    );

    // Main ingest path
    info!(
        chain_id = config.chain_id,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );
    if !matches!(config.head_source, HeadSource::P2p) {
        return Err(eyre::eyre!("ingest requires --head-source p2p"));
    }

    // Set TUI mode flag early so startup status bars are suppressed in TUI mode
    crate::ui::set_tui_mode(use_tui);

    // Create early TUI for startup phase display (before storage init)
    // Use a placeholder end_block of 0 - will be updated once we know the actual range
    let early_tui: Option<EarlyTui> = if use_tui {
        match TuiController::new(config.start_block, 0) {
            Ok(tui) => {
                let tui_arc = Arc::new(Mutex::new(tui));
                // Draw initial startup screen with config info
                {
                    let mut guard = tui_arc.lock();
                    guard.state.startup_status = "Initializing...".to_string();
                    // Set config values for display
                    guard.state.set_config(
                        &config.data_dir.display().to_string(),
                        config.shard_size,
                        &config.rpc_bind.to_string(),
                        config.rollback_window,
                    );
                    let _ = guard.draw();
                }
                Some(tui_arc)
            }
            Err(e) => {
                warn!(error = %e, "failed to create early TUI, falling back to status bars");
                None
            }
        }
    } else {
        None
    };

    // Get log buffer reference for startup TUI updates
    let log_buffer = &tracing_guards.tui_log_buffer;

    // Initialize storage
    let storage = init_storage(&config, early_tui.as_ref(), log_buffer).await?;

    // Connect to P2P network
    let session = connect_p2p(Arc::clone(&storage), early_tui.as_ref(), log_buffer).await?;

    // Wait for minimum peers
    let min_peers = config.min_peers as usize;
    wait_for_min_peers(&session.pool, min_peers, early_tui.as_ref(), log_buffer).await;
    info!(
        peers = session.pool.len(),
        min_peers, "peer warmup complete"
    );

    let follow_after = config.end_block.is_none();

    // Wait for peer head
    let head_at_startup = wait_for_peer_head(
        &session.pool,
        config.start_block,
        config.end_block,
        config.rollback_window,
        early_tui.as_ref(),
        log_buffer,
    )
    .await;

    // Compute target range
    let range = compute_target_range(
        config.start_block,
        config.end_block,
        head_at_startup,
        config.rollback_window,
    );

    // Calculate missing blocks
    let (missing_ranges, missing_total) = storage.missing_ranges_in_range(range.clone())?;
    let requested_total = range_len(&range);
    let already_present = requested_total.saturating_sub(missing_total);
    if already_present > 0 {
        info!(
            range_start = *range.start(),
            range_end = *range.end(),
            requested_total,
            already_present,
            missing_total,
            "resume: found existing blocks in range; skipping already present blocks"
        );
    } else {
        info!(
            range_start = *range.start(),
            range_end = *range.end(),
            requested_total,
            missing_total,
            "resume: no existing blocks in range"
        );
    }
    let total_len = missing_total;
    let missing_ranges_for_coverage = missing_ranges.clone();
    let blocks = MissingBlocks::Ranges {
        ranges: missing_ranges,
        total: missing_total,
    };

    // Setup progress stats
    let progress_stats = if std::io::stderr().is_terminal() {
        Some(Arc::new(SyncProgressStats::default()))
    } else {
        None
    };
    if let Some(stats) = progress_stats.as_ref() {
        stats.set_head_seen(head_at_startup);

        // Initialize coverage tracking for TUI blocks map
        stats.init_coverage(*range.start(), *range.end(), COVERAGE_BUCKETS);

        // Pre-fill coverage with blocks already in DB (computed from missing ranges)
        // For each bucket, we count how many blocks are NOT missing (= already present)
        let range_size = range.end() - range.start();
        let blocks_per_bucket = (range_size as f64 / COVERAGE_BUCKETS as f64).max(1.0);

        for bucket_idx in 0..COVERAGE_BUCKETS {
            let bucket_start = *range.start() + (bucket_idx as f64 * blocks_per_bucket) as u64;
            let bucket_end = *range.start() + ((bucket_idx + 1) as f64 * blocks_per_bucket) as u64;
            let bucket_size = bucket_end.saturating_sub(bucket_start);

            // Count missing blocks in this bucket
            let mut missing_in_bucket = 0u64;
            for missing_range in &missing_ranges_for_coverage {
                // Calculate overlap between bucket and missing range
                let overlap_start = bucket_start.max(*missing_range.start());
                let overlap_end = bucket_end.min(missing_range.end().saturating_add(1));
                if overlap_end > overlap_start {
                    missing_in_bucket += overlap_end - overlap_start;
                }
            }

            // Blocks already present = bucket_size - missing_in_bucket
            let already_present_in_bucket = bucket_size.saturating_sub(missing_in_bucket);

            // Record these as "pre-completed" by creating synthetic completed blocks
            // (We don't have the actual block numbers, so we generate representative ones)
            let present_blocks: Vec<u64> = (0..already_present_in_bucket)
                .map(|i| bucket_start + i)
                .collect();
            if !present_blocks.is_empty() {
                stats.record_coverage_completed(&present_blocks);
            }
        }
    }

    // Build peer health tracker
    let peer_health = build_peer_health_tracker(&config);
    apply_cached_peer_limits(&storage, &peer_health).await;

    // Setup resource logger and SIGUSR1 handler
    let run_ctx = run_context.as_ref();

    // Create shutdown channel early so TUI can signal quit
    let (stop_tx, stop_rx) = watch::channel(false);

    // Create completion channel so TUI waits for sync to finish before exiting
    let (completion_tx, completion_rx) = tokio::sync::oneshot::channel();

    // Setup UI (pass shutdown_tx for TUI quit handling, and early_tui to reuse)
    let log_writers = super::startup::LogWriters {
        log_writer: tracing_guards.log_writer.clone(),
        resources_writer: tracing_guards.resources_writer.clone(),
    };
    let UiSetup {
        ui_controller: _,
        tui_controller: _,
        progress,
        events,
    } = setup_ui(
        &config,
        run_ctx,
        *range.start(),
        *range.end(),
        progress_stats.as_ref(),
        &peer_health,
        &session.pool,
        Some(stop_tx.clone()),
        Some(completion_rx),
        early_tui,
        tracing_guards.tui_log_buffer.clone(),
        Some(log_writers),
    )?;

    spawn_resource_logger(
        progress_stats.clone(),
        events.clone(),
        Some(session.handle.clone()),
        Some(Arc::clone(&session.p2p_stats)),
    );

    #[cfg(unix)]
    spawn_usr1_state_logger(
        progress_stats.clone(),
        Some(Arc::clone(&session.pool)),
        Some(Arc::clone(&peer_health)),
    );

    // Setup follow mode resources
    let mut follow_resources = FollowModeResources::empty();
    let mut tail_config = None;

    if follow_after {
        let head_handles = spawn_head_tracker(
            Arc::clone(&session.pool),
            Arc::clone(&storage),
            progress_stats.clone(),
            head_at_startup,
            config.rollback_window,
        );

        let tail_handles = spawn_tail_feeder(
            *range.end(),
            config.rollback_window,
            &head_handles.head_seen_rx,
        );

        tail_config = Some(build_tail_config(
            tail_handles,
            head_handles.head_seen_rx.clone(),
            config.rollback_window,
        ));

        follow_resources = FollowModeResources {
            head_tracker: Some(head_handles.handle),
            tail_feeder: None, // Already consumed by tail_config
            head_stop_tx: Some(head_handles.stop_tx),
            tail_stop_tx: None, // Managed by tail_config
            head_seen_rx: Some(head_handles.head_seen_rx),
        };
    }

    // Setup shutdown handler
    let bench = Arc::new(IngestBenchStats::new(total_len));
    let progress_ref = progress
        .as_ref()
        .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
    let head_stop_tx_for_shutdown = follow_resources.head_stop_tx.clone();
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_flag = Arc::clone(&shutdown_requested);

    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            shutdown_flag.store(true, Ordering::SeqCst);
            warn!("shutdown signal received; stopping ingest after draining");
            let _ = stop_tx.send(true);
            if let Some(stop_tx) = head_stop_tx_for_shutdown {
                let _ = stop_tx.send(true);
            }
            if tokio::signal::ctrl_c().await.is_ok() {
                warn!("second shutdown signal received; forcing exit");
                process::exit(130);
            }
        }
    });

    // Run ingest pipeline
    let outcome = run_ingest_pipeline(
        Arc::clone(&storage),
        Arc::clone(&session.pool),
        IngestPipelineConfig {
            config: &config,
            range: range.clone(),
            head_at_startup,
            db_mode_override: None,
            head_cap_override: None,
            stop_rx: Some(stop_rx),
            tail: tail_config,
        },
        blocks,
        IngestPipelineTrackers {
            progress: progress_ref,
            stats: progress_stats.clone(),
            bench: Some(Arc::clone(&bench)),
            events: events.clone(),
        },
        Arc::clone(&peer_health),
    )
    .await?;

    // Signal TUI that sync is complete (for clean quit handling)
    let _ = completion_tx.send(());

    // Log finalize stats for follow mode transition
    let finalize_stats = match &outcome {
        IngestPipelineOutcome::RangeApplied { finalize, .. } => Some(*finalize),
        IngestPipelineOutcome::UpToDate { .. } => None,
    };

    if follow_after {
        log_follow_transition(&storage, &config, finalize_stats, &shutdown_requested);
    }

    // Handle follow mode or finalization
    if follow_after && !shutdown_requested.load(Ordering::SeqCst) {
        run_follow_mode(
            FollowModeConfig {
                storage: &storage,
                session: &session,
                config: &config,
                range: &range,
                head_at_startup,
                peer_health: &peer_health,
            },
            FollowModeTrackers {
                bench: &bench,
                events: &events,
                progress: &progress,
                progress_stats: progress_stats.clone(),
            },
            FollowModeState {
                follow_resources: &mut follow_resources,
                run_context: run_context.as_ref(),
                tracing_guards: &mut tracing_guards,
            },
            &outcome,
        )
        .await?;
        return Ok(());
    }

    // Handle shutdown during fast-sync or normal completion
    if shutdown_requested.load(Ordering::SeqCst) {
        follow_resources.shutdown().await;
    }

    if let Some(tracker) = progress.as_ref() {
        tracker.finish();
    }

    // Finalize
    let logs_total = match outcome {
        IngestPipelineOutcome::RangeApplied { logs, .. } => logs,
        IngestPipelineOutcome::UpToDate { .. } => 0,
    };

    finalize_session(
        FinalizeContext {
            run_context: run_context.as_ref(),
            config: &config,
            range: &range,
            head_at_startup,
            peer_health: &peer_health,
            network: &session,
            storage: &storage,
            bench: &bench,
            events: events.as_deref(),
            tracing_guards: &mut tracing_guards,
        },
        logs_total,
    )
    .await;

    Ok(())
}

fn log_follow_transition(
    storage: &Storage,
    config: &NodeConfig,
    finalize_stats: Option<crate::sync::historical::IngestFinalizeStats>,
    shutdown_requested: &AtomicBool,
) {
    let last_indexed = storage.last_indexed_block().ok().flatten();
    let head_seen = storage.head_seen().ok().flatten();
    let safe_head = head_seen.map(|head| head.saturating_sub(config.rollback_window));
    let dirty = storage.dirty_shards().unwrap_or_default();
    let total_wal_bytes: u64 = dirty.iter().map(|info| info.wal_bytes).sum();
    let total_wal_mib = (total_wal_bytes as f64 / (1024.0 * 1024.0)).round();
    tracing::info!(
        follow_after = true,
        shutdown_requested = shutdown_requested.load(Ordering::SeqCst),
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

/// Configuration for follow mode execution.
struct FollowModeConfig<'a> {
    storage: &'a Arc<Storage>,
    session: &'a p2p::NetworkSession,
    config: &'a NodeConfig,
    range: &'a RangeInclusive<u64>,
    head_at_startup: u64,
    peer_health: &'a Arc<PeerHealthTracker>,
}

/// Trackers and optional logging for follow mode.
struct FollowModeTrackers<'a> {
    bench: &'a Arc<IngestBenchStats>,
    events: &'a Option<Arc<BenchEventLogger>>,
    progress: &'a Option<Arc<IngestProgress>>,
    progress_stats: Option<Arc<SyncProgressStats>>,
}

/// Mutable state for follow mode.
struct FollowModeState<'a> {
    follow_resources: &'a mut FollowModeResources,
    run_context: Option<&'a crate::logging::RunContext>,
    tracing_guards: &'a mut TracingGuards,
}

#[expect(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    reason = "follow mode orchestration with select loop is clearer inline"
)]
async fn run_follow_mode(
    cfg: FollowModeConfig<'_>,
    trackers: FollowModeTrackers<'_>,
    state: FollowModeState<'_>,
    outcome: &IngestPipelineOutcome,
) -> Result<()> {
    let FollowModeConfig {
        storage,
        session,
        config,
        range,
        head_at_startup,
        peer_health,
    } = cfg;
    let FollowModeTrackers {
        bench,
        events,
        progress,
        progress_stats,
    } = trackers;
    let FollowModeState {
        follow_resources,
        run_context,
        tracing_guards,
    } = state;
    info!("fast-sync complete; switching to follow mode");

    // Generate report at fast-sync completion
    let logs_total = match outcome {
        IngestPipelineOutcome::RangeApplied { logs, .. } => *logs,
        IngestPipelineOutcome::UpToDate { .. } => 0,
    };

    let storage_stats = match storage.disk_usage() {
        Ok(stats) => Some(stats),
        Err(err) => {
            warn!(error = %err, "failed to collect storage disk stats");
            None
        }
    };

    let summary = bench.summary(SummaryInput {
        range_start: *range.start(),
        range_end: *range.end(),
        head_at_startup,
        rollback_window_applied: config.rollback_window > 0,
        peers_used: session.pool.len() as u64,
        logs_total,
        storage_stats,
    });

    let report_base_name = if let Some(run_ctx) = run_context {
        match crate::logging::generate_run_report(
            run_ctx,
            config,
            range,
            head_at_startup,
            peer_health,
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

    // Run follow loop
    let progress_ref = progress
        .as_ref()
        .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
    let (synced_tx, synced_rx) = tokio::sync::oneshot::channel();
    // Clone progress_stats so we can set rpc_active when RPC starts
    let stats_for_rpc = progress_stats.clone();
    let follow_future = run_follow_loop(
        Arc::clone(storage),
        Arc::clone(&session.pool),
        config,
        progress_ref,
        progress_stats,
        Arc::clone(peer_health),
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
            () = async {
                if let Some(rx) = synced_rx.take() {
                    let _ = rx.await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => {
                if rpc_handle.is_none() {
                    let handle = rpc::start(
                        config.rpc_bind,
                        rpc::RpcConfig::from(config),
                        Arc::clone(storage),
                    )
                    .await?;
                    info!(rpc_bind = %config.rpc_bind, "rpc server started");
                    // Signal to UI that RPC is now active
                    if let Some(stats) = stats_for_rpc.as_ref() {
                        stats.set_rpc_active(true);
                    }
                    rpc_handle = Some(handle);
                }
            }
        }
    }

    // Cleanup
    if let Some(tracker) = progress.as_ref() {
        tracker.finish();
    }
    if let Some(handle) = rpc_handle.take() {
        if let Err(err) = handle.stop() {
            warn!(error = %err, "failed to stop rpc server");
        }
        handle.stopped().await;
    }

    // Shutdown head tracker
    if let Some(stop_tx) = follow_resources.head_stop_tx.take() {
        let _ = stop_tx.send(true);
    }
    if let Some(handle) = follow_resources.head_tracker.take() {
        let _ = handle.await;
    }

    // Finalize log files on exit from follow mode
    if let Some(run_ctx) = run_context {
        if let Some(base_name) = report_base_name.as_ref() {
            crate::logging::finalize_log_files(
                run_ctx,
                base_name,
                events.as_deref(),
                tracing_guards.log_writer.as_deref(),
                tracing_guards.resources_writer.as_deref(),
                &mut tracing_guards.chrome_guard,
            );
        }
    }

    flush_peer_cache_with_limits(session, storage, Some(peer_health)).await;
    Ok(())
}
