//! Startup initialization functions.

use crate::cli::NodeConfig;
use crate::logging::RunContext;
use crate::p2p::{self, PeerPool};
use crate::storage::Storage;
use crate::sync::historical::{BenchEventLogger, PeerHealthTracker};
use crate::sync::SyncProgressStats;
use crate::ui::{self, UIController};
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
pub async fn init_storage(config: &NodeConfig) -> Result<Arc<Storage>> {
    ui::print_status_bar("Opening storage...");
    let storage = Arc::new(Storage::open(config)?);

    // Resume behavior: if the previous run exited before compaction finished, ensure we
    // compact any completed shards up-front so we don't keep reprocessing WAL-heavy shards
    // across restarts.
    let dirty_shards = storage.dirty_complete_shards()?;
    if !dirty_shards.is_empty() {
        let shard_count = dirty_shards.len();
        ui::print_status_bar(&format!("Compacting {shard_count} shards..."));
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
pub async fn connect_p2p(storage: Arc<Storage>) -> Result<p2p::NetworkSession> {
    ui::print_status_bar("Connecting to P2P network...");
    info!("starting p2p network");
    let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
    let initial_peers = session.pool.len();
    info!(peers = initial_peers, "p2p peers connected");
    ui::print_status_bar(&format!("P2P connected | {initial_peers} peers"));
    Ok(session)
}

/// Wait for minimum number of peers.
pub async fn wait_for_min_peers(pool: &Arc<PeerPool>, min_peers: usize) {
    if min_peers == 0 {
        return;
    }
    loop {
        let peers = pool.len();
        if peers >= min_peers {
            ui::clear_status_bar();
            return;
        }
        ui::print_status_bar(&format!("Waiting for peers... {peers}/{min_peers}"));
        sleep(Duration::from_millis(200)).await;
    }
}

/// Wait for peer head to be at or above start block.
pub async fn wait_for_peer_head(
    pool: &Arc<PeerPool>,
    start_block: u64,
    end_block: Option<u64>,
    rollback_window: u64,
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
        sleep(Duration::from_secs(1)).await;
    }
}

/// Setup result containing UI components and progress tracking.
pub struct UiSetup {
    /// UI controller for progress display (currently unused but kept for potential debug use).
    #[expect(dead_code, reason = "kept for potential debug use")]
    pub ui_controller: Arc<Mutex<UIController>>,
    pub progress: Option<Arc<IngestProgress>>,
    pub events: Option<Arc<BenchEventLogger>>,
}

/// Setup UI controller, progress bar, and event logger.
pub fn setup_ui(
    config: &NodeConfig,
    run_context: Option<&RunContext>,
    total_len: u64,
    progress_stats: Option<&Arc<SyncProgressStats>>,
    peer_health: &Arc<PeerHealthTracker>,
    pool: &Arc<PeerPool>,
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

    // Create UI controller and progress bar
    let is_tty = std::io::stderr().is_terminal();
    let ui_controller = Arc::new(Mutex::new(UIController::new(is_tty)));

    let progress: Option<Arc<IngestProgress>> = if is_tty {
        // Initialize syncing state with progress bar
        {
            let mut ui = ui_controller.lock();
            ui.show_syncing(total_len);
        }
        if let Some(stats) = progress_stats {
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
        ui_controller,
        progress,
        events,
    })
}
