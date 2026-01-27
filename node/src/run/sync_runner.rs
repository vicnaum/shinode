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
    IngestBenchStats, IngestPipelineOutcome, MissingBlocks, PeerHealthTracker,
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
    UiSetup,
};
use super::trackers::{build_tail_config, spawn_head_tracker, spawn_tail_feeder};

/// Default peer cache directory.
fn default_peer_cache_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join(".stateless-history-node"))
}

/// Run the main sync process.
pub async fn run_sync(mut config: NodeConfig, argv: Vec<String>) -> Result<()> {
    // Validate and normalize chunk sizes
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

    // Build run context for log artifacts
    let run_context = build_run_context(&config, argv);

    // Set default peer cache dir
    if config.peer_cache_dir.is_none() {
        config.peer_cache_dir = default_peer_cache_dir();
    }

    // Initialize tracing
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

    // Main ingest path
    info!(
        chain_id = config.chain_id,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );
    if !matches!(config.head_source, HeadSource::P2p) {
        return Err(eyre::eyre!("ingest requires --head-source p2p"));
    }

    // Initialize storage
    let storage = init_storage(&config).await?;

    // Connect to P2P network
    let session = connect_p2p(Arc::clone(&storage)).await?;

    // Wait for minimum peers
    let min_peers = config.min_peers as usize;
    wait_for_min_peers(&session.pool, min_peers).await;
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
    }

    // Build peer health tracker
    let peer_health = build_peer_health_tracker(&config);
    apply_cached_peer_limits(&storage, &peer_health).await;

    // Setup resource logger and SIGUSR1 handler
    let run_ctx = run_context.as_ref();

    // Setup UI
    let UiSetup {
        ui_controller: _,
        progress,
        events,
    } = setup_ui(
        &config,
        run_ctx,
        total_len,
        progress_stats.as_ref(),
        &peer_health,
        &session.pool,
    )?;

    spawn_resource_logger(progress_stats.clone(), events.clone());

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
    let (stop_tx, stop_rx) = watch::channel(false);
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
        &config,
        range.clone(),
        blocks,
        head_at_startup,
        progress_ref,
        progress_stats.clone(),
        Some(Arc::clone(&bench)),
        None,
        None,
        Arc::clone(&peer_health),
        events.clone(),
        Some(stop_rx),
        tail_config,
    )
    .await?;

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
            &storage,
            &session,
            &config,
            &range,
            head_at_startup,
            &peer_health,
            &bench,
            &events,
            &progress,
            progress_stats.clone(),
            &mut follow_resources,
            run_context.as_ref(),
            &mut tracing_guards,
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

#[expect(clippy::ref_option, reason = "consistent signature with caller-owned Options")]
async fn run_follow_mode(
    storage: &Arc<Storage>,
    session: &p2p::NetworkSession,
    config: &NodeConfig,
    range: &RangeInclusive<u64>,
    head_at_startup: u64,
    peer_health: &Arc<PeerHealthTracker>,
    bench: &Arc<IngestBenchStats>,
    events: &Option<Arc<BenchEventLogger>>,
    progress: &Option<Arc<IngestProgress>>,
    progress_stats: Option<Arc<SyncProgressStats>>,
    follow_resources: &mut FollowModeResources,
    run_context: Option<&crate::logging::RunContext>,
    tracing_guards: &mut TracingGuards,
    outcome: &IngestPipelineOutcome,
) -> Result<()> {
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

    let summary = bench.summary(
        *range.start(),
        *range.end(),
        head_at_startup,
        config.rollback_window > 0,
        session.pool.len() as u64,
        logs_total,
        storage_stats,
    );

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
