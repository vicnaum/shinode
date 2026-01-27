//! Startup initialization functions.

use crate::cli::NodeConfig;
use crate::logging::RunContext;
use crate::p2p::{self, PeerPool};
use crate::storage::Storage;
use crate::sync::historical::{BenchEventLogger, PeerHealthTracker};
use crate::sync::SyncProgressStats;
use crate::ui::{self, tui::TuiController, UIController};
use eyre::Result;
use std::fs;
use std::io::IsTerminal;
use std::process;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::time::sleep;
use tracing::info;

use super::session::IngestProgress;
use crate::ui::tui::Phase;

/// Type alias for early TUI controller used during startup.
pub type EarlyTui = Arc<Mutex<TuiController>>;

/// Update the TUI startup status message.
/// This is a no-op if tui is None.
/// Returns true if quit was requested.
pub fn update_tui_startup(tui: Option<&EarlyTui>, status: &str) -> bool {
    if let Some(tui) = tui {
        let mut guard = tui.lock();
        guard.state.startup_status = status.to_string();
        guard.state.phase = Phase::Startup;
        // Check for quit
        if guard.poll_quit().unwrap_or(false) || guard.should_quit {
            guard.state.quitting = true;
            let _ = guard.draw();
            return true;
        }
        let _ = guard.draw();
    }
    false
}

/// Update the TUI startup status with best head seen and peer count.
/// Returns true if quit was requested.
pub fn update_tui_startup_head(tui: Option<&EarlyTui>, status: &str, best_head: u64, peers: u64) -> bool {
    if let Some(tui) = tui {
        let mut guard = tui.lock();
        guard.state.startup_status = status.to_string();
        guard.state.phase = Phase::Startup;
        guard.state.best_head_seen = best_head;
        guard.state.peers_connected = peers;
        // Check for quit
        if guard.poll_quit().unwrap_or(false) || guard.should_quit {
            guard.state.quitting = true;
            let _ = guard.draw();
            return true;
        }
        let _ = guard.draw();
    }
    false
}

/// Update the TUI startup status with peer count.
/// Returns true if quit was requested.
pub fn update_tui_startup_peers(tui: Option<&EarlyTui>, status: &str, peers: u64) -> bool {
    if let Some(tui) = tui {
        let mut guard = tui.lock();
        guard.state.startup_status = status.to_string();
        guard.state.phase = Phase::Startup;
        guard.state.peers_connected = peers;
        // Check for quit
        if guard.poll_quit().unwrap_or(false) || guard.should_quit {
            guard.state.quitting = true;
            let _ = guard.draw();
            return true;
        }
        let _ = guard.draw();
    }
    false
}

/// Handle quit request during startup - restores terminal and exits.
pub fn handle_startup_quit(tui: Option<&EarlyTui>) -> ! {
    if let Some(tui) = tui {
        let guard = tui.lock();
        let _ = guard.restore();
    }
    process::exit(0);
}

/// Build the run context for log artifacts.
pub fn build_run_context(config: &NodeConfig, argv: Vec<String>) -> Option<RunContext> {
    let log_artifacts_enabled = config.log_trace
        || config.log_events
        || config.log_json
        || config.log_report
        || config.log_resources;

    if !log_artifacts_enabled {
        return None;
    }

    let timestamp_utc = crate::logging::run_timestamp_utc(SystemTime::now());
    let run_id = format!("{}-{}", timestamp_utc, process::id());
    let run_name = config
        .run_name
        .clone()
        .unwrap_or_else(|| "sync".to_string());
    let output_dir = config.log_output_dir.clone();

    let trace_tmp_path = if config.log_trace {
        Some(output_dir.join(format!("{timestamp_utc}.trace.json.part")))
    } else {
        None
    };
    let events_tmp_path = if config.log_events {
        Some(output_dir.join(format!("{timestamp_utc}.events.jsonl.part")))
    } else {
        None
    };
    let logs_tmp_path = if config.log_json {
        Some(output_dir.join(format!("{timestamp_utc}.logs.jsonl.part")))
    } else {
        None
    };
    let resources_tmp_path = if config.log_resources {
        Some(output_dir.join(format!("{timestamp_utc}.resources.jsonl.part")))
    } else {
        None
    };

    fs::create_dir_all(&output_dir).ok()?;

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
}

/// Initialize storage with startup compaction of dirty shards.
#[expect(clippy::cognitive_complexity, reason = "storage init with optional compaction")]
pub async fn init_storage(config: &NodeConfig, tui: Option<&EarlyTui>) -> Result<Arc<Storage>> {
    ui::print_status_bar("Opening storage...");
    update_tui_startup(tui, "Opening storage...");
    let storage = Arc::new(Storage::open(config)?);

    // Resume behavior: if the previous run exited before compaction finished, ensure we
    // compact any completed shards up-front so we don't keep reprocessing WAL-heavy shards
    // across restarts.
    let dirty_shards = storage.dirty_complete_shards()?;
    if !dirty_shards.is_empty() {
        let shard_count = dirty_shards.len();
        let status_msg = format!("Compacting {shard_count} shards...");
        ui::print_status_bar(&status_msg);
        update_tui_startup(tui, &status_msg);
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

    Ok(storage)
}

/// Connect to the P2P network.
#[expect(clippy::cognitive_complexity, reason = "P2P connection with status updates")]
pub async fn connect_p2p(storage: Arc<Storage>, tui: Option<&EarlyTui>) -> Result<p2p::NetworkSession> {
    ui::print_status_bar("Connecting to P2P network...");
    update_tui_startup(tui, "Connecting to P2P network...");
    info!("starting p2p network");
    let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
    let initial_peers = session.pool.len();
    info!(peers = initial_peers, "p2p peers connected");
    let status_msg = format!("P2P connected | {initial_peers} peers");
    ui::print_status_bar(&status_msg);
    update_tui_startup_peers(tui, &status_msg, initial_peers as u64);
    Ok(session)
}

/// Wait for minimum number of peers.
pub async fn wait_for_min_peers(pool: &Arc<PeerPool>, min_peers: usize, tui: Option<&EarlyTui>) {
    if min_peers == 0 {
        return;
    }
    loop {
        let peers = pool.len();
        if peers >= min_peers {
            ui::clear_status_bar();
            return;
        }
        let status_msg = format!("Waiting for peers... {peers}/{min_peers}");
        ui::print_status_bar(&status_msg);
        if update_tui_startup_peers(tui, &status_msg, peers as u64) {
            handle_startup_quit(tui);
        }
        sleep(Duration::from_millis(200)).await;
    }
}

/// Wait for peer head to be at or above start block.
pub async fn wait_for_peer_head(
    pool: &Arc<PeerPool>,
    start_block: u64,
    end_block: Option<u64>,
    rollback_window: u64,
    tui: Option<&EarlyTui>,
) -> u64 {
    let mut last_log = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
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
        let status_msg = format!(
            "Discovering chain head... {} | peers {}",
            best_head,
            peers.len()
        );
        ui::print_status_bar(&status_msg);
        // TUI shows head/peers in dedicated fields, so use simpler status
        if update_tui_startup_head(tui, "Discovering chain head...", best_head, peers.len() as u64) {
            handle_startup_quit(tui);
        }
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
        sleep(Duration::from_secs(1)).await;
    }
}

/// Setup result containing UI components and progress tracking.
pub struct UiSetup {
    /// UI controller for progress display (indicatif mode).
    #[expect(dead_code, reason = "kept for potential debug use")]
    pub ui_controller: Option<Arc<Mutex<UIController>>>,
    /// TUI controller for fullscreen dashboard mode.
    #[expect(dead_code, reason = "TUI runs in background task")]
    pub tui_controller: Option<Arc<Mutex<TuiController>>>,
    pub progress: Option<Arc<IngestProgress>>,
    pub events: Option<Arc<BenchEventLogger>>,
}

/// Setup UI controller, progress bar, and event logger.
///
/// The `shutdown_tx` is used to signal the sync runner to stop when the user
/// presses 'q' in the TUI.
///
/// The `completion_rx` is used to wait for sync completion before fully exiting
/// the TUI after the user presses 'q'.
///
/// If `early_tui` is provided, it will be reused instead of creating a new TUI.
/// The early TUI is created during startup to show startup phase status.
#[expect(clippy::too_many_arguments, reason = "setup function needs many config params")]
pub fn setup_ui(
    config: &NodeConfig,
    run_context: Option<&RunContext>,
    start_block: u64,
    end_block: u64,
    progress_stats: Option<&Arc<SyncProgressStats>>,
    peer_health: &Arc<PeerHealthTracker>,
    pool: &Arc<PeerPool>,
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    completion_rx: Option<tokio::sync::oneshot::Receiver<()>>,
    early_tui: Option<EarlyTui>,
    tui_log_buffer: Option<Arc<crate::logging::TuiLogBuffer>>,
) -> Result<UiSetup> {
    // Create event logger if enabled
    let events = if config.log_events {
        let run_ctx = run_context
            .ok_or_else(|| eyre::eyre!("run context required for log_events"))?;
        let tmp_path = run_ctx
            .events_tmp_path
            .clone()
            .ok_or_else(|| eyre::eyre!("events tmp path required"))?;
        Some(Arc::new(BenchEventLogger::new(&tmp_path)?))
    } else {
        None
    };

    let is_tty = std::io::stderr().is_terminal();
    let total_len = end_block.saturating_sub(start_block);
    let use_tui = !config.no_tui && is_tty;

    // TUI mode: fullscreen dashboard (default when TTY unless --no-tui)
    // Note: TUI mode flag is set earlier in run_sync before storage init
    if use_tui {
        // Reuse early TUI if provided, otherwise create new one
        let tui_controller = if let Some(tui) = early_tui {
            // Update the early TUI with actual end_block and transition to Sync phase
            {
                let mut guard = tui.lock();
                guard.state.end_block = end_block;
                guard.state.phase = Phase::Sync;
            }
            tui
        } else {
            let tui = TuiController::new(start_block, end_block)
                .map_err(|e| eyre::eyre!("failed to create TUI: {}", e))?;
            Arc::new(Mutex::new(tui))
        };

        // Initialize stats and spawn TUI progress updater
        if let Some(stats) = progress_stats {
            stats.set_start_block(start_block);
            stats.set_peers_active(0);
            stats.set_peers_total(pool.len() as u64);
            ui::spawn_tui_progress_updater(
                Arc::clone(&tui_controller),
                Arc::clone(stats),
                Arc::clone(pool),
                Some(Arc::clone(peer_health)),
                end_block,
                shutdown_tx,
                completion_rx,
                tui_log_buffer,
            );
        }

        return Ok(UiSetup {
            ui_controller: None,
            tui_controller: Some(tui_controller),
            progress: None,
            events,
        });
    }

    // Indicatif mode: progress bars
    let ui_controller = Arc::new(Mutex::new(UIController::new(is_tty)));

    let progress: Option<Arc<IngestProgress>> = if is_tty {
        // Initialize syncing state with progress bar
        {
            let mut ui = ui_controller.lock();
            ui.show_syncing(total_len);
        }
        if let Some(stats) = progress_stats {
            stats.set_start_block(start_block);
            stats.set_peers_active(0);
            stats.set_peers_total(pool.len() as u64);
            ui::spawn_progress_updater(
                Arc::clone(&ui_controller),
                Arc::clone(stats),
                Arc::clone(pool),
                Some(Arc::clone(peer_health)),
            );
        }
        // Get the bar from the controller for IngestProgress
        // SAFETY: sync bar is created in show_syncing above, so it should always exist
        let bar = ui_controller
            .lock()
            .current_bar()
            .cloned();
        let Some(bar) = bar else {
            return Err(eyre::eyre!("sync bar should exist after show_syncing"));
        };
        Some(Arc::new(IngestProgress::new(bar, total_len)))
    } else {
        None
    };

    Ok(UiSetup {
        ui_controller: Some(ui_controller),
        tui_controller: None,
        progress,
        events,
    })
}
