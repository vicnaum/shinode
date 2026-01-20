mod cli;
mod metrics;
mod p2p;
mod rpc;
mod storage;
mod sync;

use cli::{compute_target_range, BenchmarkMode, NodeConfig};
use eyre::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use metrics::range_len;
use std::{
    collections::VecDeque,
    io::IsTerminal,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use p2p::PeerPool;
use sync::{
    BlockPayloadSource, ProgressReporter, SyncProgressStats, SyncStatus,
};
use sync::historical::{IngestBenchStats, PeerHealthTracker, ProbeStats};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

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
#[tokio::main]
async fn main() -> Result<()> {
    let config = NodeConfig::from_args();
    init_tracing(config.verbosity);

    if let Some(command) = &config.command {
        match command {
            cli::Command::Db(cli::DbCommand::Stats(args)) => {
                let data_dir = args
                    .data_dir
                    .clone()
                    .unwrap_or_else(|| config.data_dir.clone());
                let stats = storage::Storage::disk_usage_at(&data_dir)?;
                print_db_stats(&data_dir, &stats, args.json)?;
                return Ok(());
            }
        }
    }

    if matches!(config.benchmark, BenchmarkMode::Probe) {
        info!(
            chain_id = config.chain_id,
            data_dir = %config.data_dir.display(),
            "starting stateless history node (benchmark probe)"
        );
        if !matches!(config.head_source, cli::HeadSource::P2p) {
            return Err(eyre::eyre!("benchmark probe requires --head-source p2p"));
        }
        let peer_cache = storage::Storage::open_existing(&config)?
            .map(Arc::new);
        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers(peer_cache).await?;
        info!(peers = session.pool.len(), "p2p peers connected");
        let source = p2p::MultiPeerBlockPayloadSource::new(session.pool.clone());
        let head_at_startup = source.head().await?;
        let range = compute_target_range(
            config.start_block,
            config.end_block,
            head_at_startup,
            config.rollback_window,
        );
        let blocks_total = range_len(&range);
        if blocks_total == 0 {
            return Err(eyre::eyre!("benchmark range is empty"));
        }
        let stats = Arc::new(ProbeStats::new(blocks_total));
        let probe_future = sync::historical::run_benchmark_probe(
            &config,
            Arc::clone(&session.pool),
            range.clone(),
            head_at_startup,
            Arc::clone(&stats),
        );
        tokio::pin!(probe_future);
        tokio::select! {
            res = &mut probe_future => {
                res?;
            }
            _ = tokio::signal::ctrl_c() => {
                let summary = stats.summary(
                    *range.start(),
                    *range.end(),
                    head_at_startup,
                    config.rollback_window > 0,
                    session.pool.len() as u64,
                );
                let summary_json = serde_json::to_string_pretty(&summary)?;
                println!("{summary_json}");
                if let Err(err) = session.flush_peer_cache() {
                    warn!(error = %err, "failed to flush peer cache");
                }
                return Ok(());
            }
        }
        if let Err(err) = session.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache");
        }
        return Ok(());
    }

    if matches!(config.benchmark, BenchmarkMode::Ingest) {
        info!(
            chain_id = config.chain_id,
            data_dir = %config.data_dir.display(),
            "starting stateless history node (benchmark ingest)"
        );
        if !matches!(config.head_source, cli::HeadSource::P2p) {
            return Err(eyre::eyre!("benchmark ingest requires --head-source p2p"));
        }

        let storage = Arc::new(storage::Storage::open(&config)?);

        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
        info!(peers = session.pool.len(), "p2p peers connected");
        let source = p2p::MultiPeerBlockPayloadSource::new(session.pool.clone());
        let head_at_startup = source.head().await?;
        let range = compute_target_range(
            config.start_block,
            config.end_block,
            head_at_startup,
            config.rollback_window,
        );
        let total_len = range_len(&range);

        let progress_stats = if std::io::stderr().is_terminal() {
            Some(Arc::new(SyncProgressStats::default()))
        } else {
            None
        };
        let peer_health = sync::historical::build_peer_health_tracker(&config);

        let progress: Option<Arc<IngestProgress>> = if std::io::stderr().is_terminal() {
            let bar = ProgressBar::new(total_len);
            bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
            let style = ProgressStyle::with_template(
                "{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}",
            )
            .expect("progress style");
            bar.set_style(style.progress_chars("█▉░"));
            bar.set_message(format_progress_message(
                SyncStatus::LookingForPeers,
                0,
                0,
                0,
                0,
                0,
                0.0,
                "--",
            ));
            if let Some(stats) = progress_stats.as_ref() {
                stats.set_peers_active(0);
                stats.set_peers_total(session.pool.len() as u64);
                spawn_progress_updater(
                    bar.clone(),
                    Arc::clone(stats),
                    total_len,
                    Arc::clone(&session.pool),
                    Some(Arc::clone(&peer_health)),
                );
            }
            Some(Arc::new(IngestProgress::new(bar, total_len)))
        } else {
            None
        };

        let bench = Arc::new(IngestBenchStats::new(total_len));
        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let ingest_future = sync::historical::run_ingest_pipeline(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            range.clone(),
            head_at_startup,
            progress_ref,
            progress_stats.clone(),
            Some(Arc::clone(&bench)),
            None,
            peer_health,
        );
        tokio::pin!(ingest_future);
        let outcome = tokio::select! {
            res = &mut ingest_future => res?,
            _ = tokio::signal::ctrl_c() => {
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
                    bench.logs_total(),
                    storage_stats,
                );
                let summary_json = serde_json::to_string_pretty(&summary)?;
                println!("{summary_json}");
                if let Err(err) = session.flush_peer_cache() {
                    warn!(error = %err, "failed to flush peer cache");
                }
                return Ok(());
            }
        };

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
        let summary_json = serde_json::to_string_pretty(&summary)?;
        println!("{summary_json}");
        if let Err(err) = session.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache");
        }
        return Ok(());
    }

    info!(
        chain_id = config.chain_id,
        rpc_bind = %config.rpc_bind,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );

    let storage = Arc::new(storage::Storage::open(&config)?);
    let head_seen = storage.head_seen()?;
    let last_indexed = storage.last_indexed_block()?;
    info!(
        head_seen = ?head_seen,
        last_indexed = ?last_indexed,
        "sync checkpoints loaded"
    );

    let rpc_handle =
        rpc::start(config.rpc_bind, rpc::RpcConfig::from(&config), Arc::clone(&storage)).await?;
    info!(rpc_bind = %config.rpc_bind, "rpc server started");

    let mut _network_session = None;
    let mut follow_error: Option<eyre::Report> = None;
    let mut progress: Option<Arc<IngestProgress>> = None;
    if matches!(config.head_source, cli::HeadSource::P2p) {
        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers(Some(Arc::clone(&storage))).await?;
        info!(peers = session.pool.len(), "p2p peers connected");
        let source = p2p::MultiPeerBlockPayloadSource::new(session.pool.clone());
        let initial_head = source.head().await?;
        let last_indexed = storage.last_indexed_block()?;
        let start_from = select_start_from(config.start_block, last_indexed);
        let initial_range = compute_target_range(
            start_from,
            config.end_block,
            initial_head,
            config.rollback_window,
        );
        let total_len = range_len(&initial_range);

        let progress_stats = if std::io::stderr().is_terminal() {
            Some(Arc::new(SyncProgressStats::default()))
        } else {
            None
        };
        let peer_health = sync::historical::build_peer_health_tracker(&config);

        progress = if std::io::stderr().is_terminal() {
            let bar = ProgressBar::new(total_len);
            bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
            let style = ProgressStyle::with_template(
                "{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}",
            )
            .expect("progress style");
            bar.set_style(style.progress_chars("█▉░"));
            bar.set_message(format_progress_message(
                SyncStatus::LookingForPeers,
                0,
                0,
                0,
                0,
                0,
                0.0,
                "--",
            ));
            if let Some(stats) = progress_stats.as_ref() {
                stats.set_peers_active(0);
                stats.set_peers_total(session.pool.len() as u64);
                spawn_progress_updater(
                    bar.clone(),
                    Arc::clone(stats),
                    total_len,
                    Arc::clone(&session.pool),
                    Some(Arc::clone(&peer_health)),
                );
            }
            Some(Arc::new(IngestProgress::new(bar, total_len)))
        } else {
            None
        };

        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let follow_future = sync::historical::run_follow_loop(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            progress_ref,
            progress_stats.clone(),
            peer_health,
        );

        _network_session = Some(session);
        tokio::pin!(follow_future);
        tokio::select! {
            res = &mut follow_future => {
                if let Err(err) = res {
                    follow_error = Some(err);
                }
            }
            _ = tokio::signal::ctrl_c() => {
                warn!("shutdown signal received");
            }
        }
    } else {
        tokio::signal::ctrl_c().await?;
        warn!("shutdown signal received");
    }

    if let Some(tracker) = progress.as_ref() {
        tracker.finish();
    }
    if let Err(err) = rpc_handle.stop() {
        warn!(error = %err, "failed to stop rpc server");
    }
    rpc_handle.stopped().await;
    if let Some(session) = _network_session.as_ref() {
        if let Err(err) = session.flush_peer_cache() {
            warn!(error = %err, "failed to flush peer cache");
        }
    }
    drop(_network_session);
    drop(storage);
    warn!("shutdown complete");

    if let Some(err) = follow_error {
        return Err(err);
    }

    Ok(())
}

fn init_tracing(verbosity: u8) {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => {
            let (global, local) = match verbosity {
                0 => ("error", "error"),
                1 => ("warn", "info"),
                2 => ("warn", "debug"),
                _ => ("warn", "trace"),
            };
            EnvFilter::new(format!("{global},stateless_history_node={local}"))
        }
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();
}

fn select_start_from(start_block: u64, last_indexed: Option<u64>) -> u64 {
    last_indexed
        .map(|block| block.saturating_add(1))
        .unwrap_or(start_block)
        .max(start_block)
}

#[allow(dead_code)]
fn total_blocks_to_head(start_from: u64, head: u64) -> u64 {
    if head >= start_from {
        head.saturating_sub(start_from).saturating_add(1)
    } else {
        0
    }
}

fn format_progress_message(
    status: SyncStatus,
    peers_active: u64,
    peers_total: u64,
    queue: u64,
    inflight: u64,
    failed: u64,
    speed: f64,
    eta: &str,
) -> String {
    format!(
        "status {} | peers {}/{} | queue {} | inflight {} | failed {} | speed {:.1}/s | eta {}",
        status.as_str(),
        peers_active,
        peers_total,
        queue,
        inflight,
        failed,
        speed,
        eta
    )
}

fn print_db_stats(
    data_dir: &Path,
    stats: &storage::StorageDiskStats,
    json: bool,
) -> Result<()> {
    if json {
        let payload = serde_json::to_string_pretty(stats)?;
        println!("{payload}");
        return Ok(());
    }

    println!("DB stats for {}", data_dir.display());
    println!(
        "{:<18} {:>14} {:>12}",
        "Component", "Bytes", "Size"
    );
    println!("{}", "-".repeat(46));
    print_row("total", stats.total_bytes);
    print_row("static", stats.static_total_bytes);
    print_row("meta", stats.meta_bytes);
    print_row("peers", stats.peers_bytes);

    println!();
    println!(
        "{:<18} {:>14} {:>12} {:>8}",
        "Segment", "Bytes", "Size", "Share"
    );
    println!("{}", "-".repeat(54));
    for segment in &stats.segments {
        let share = if stats.static_total_bytes > 0 {
            segment.bytes as f64 / stats.static_total_bytes as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "{:<18} {:>14} {:>12} {:>7.2}%",
            segment.name,
            segment.bytes,
            human_bytes(segment.bytes),
            share
        );
    }
    Ok(())
}

fn print_row(label: &str, bytes: u64) {
    println!(
        "{:<18} {:>14} {:>12}",
        label,
        bytes,
        human_bytes(bytes)
    );
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut idx = 0;
    while value >= 1024.0 && idx < UNITS.len() - 1 {
        value /= 1024.0;
        idx += 1;
    }
    format!("{value:.2} {}", UNITS[idx])
}

fn spawn_progress_updater(
    bar: ProgressBar,
    stats: Arc<SyncProgressStats>,
    total_len: u64,
    peer_pool: Arc<PeerPool>,
    peer_health: Option<Arc<PeerHealthTracker>>,
) {
    tokio::spawn(async move {
        let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        let mut initial_sync_done = false;
        let mut follow_bar: Option<ProgressBar> = None;
        let mut failed_bar: Option<ProgressBar> = None;
        let mut failed_total = 0u64;
        let mut catchup_start_block: Option<u64> = None;
        let mut last_peer_update = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .unwrap_or_else(Instant::now);
        
        loop {
            ticker.tick().await;
            let now = Instant::now();
            if now.duration_since(last_peer_update) >= Duration::from_millis(500) {
                last_peer_update = now;
                let connected = peer_pool.len() as u64;
                let available = if let Some(peer_health) = peer_health.as_ref() {
                    let peers = peer_pool.snapshot();
                    let peer_ids: Vec<_> = peers.iter().map(|peer| peer.peer_id).collect();
                    let banned = peer_health.count_banned_peers(&peer_ids).await;
                    connected.saturating_sub(banned)
                } else {
                    connected
                };
                stats.set_peers_total(available);
            }
            let snapshot = stats.snapshot();
            let failed = snapshot.failed;

            if failed > failed_total {
                failed_total = failed;
                if let Some(ref bar) = failed_bar {
                    bar.set_length(failed_total.max(1));
                }
            }
            if failed_total > 0 && failed_bar.is_none() {
                let fb = ProgressBar::with_draw_target(
                    Some(failed_total.max(1)),
                    ProgressDrawTarget::stderr_with_hz(2),
                );
                let style = ProgressStyle::with_template(
                    "{bar:40.red/black} {pos}/{len} | {msg}",
                )
                .expect("progress style")
                .progress_chars("▓▒░");
                fb.set_style(style);
                failed_bar = Some(fb);
            }
            if let Some(ref fb) = failed_bar {
                let remaining = failed;
                let done = failed_total.saturating_sub(remaining);
                fb.set_length(failed_total.max(1));
                fb.set_position(done);
                fb.set_message(format!(
                    "recovering failed: remaining {remaining}/{failed_total}"
                ));
                if remaining == 0 {
                    fb.finish_and_clear();
                    failed_bar = None;
                    failed_total = 0;
                }
            }
            
            // Check if initial sync is complete and we're now in follow mode
            if !initial_sync_done {
                let processed = snapshot.processed.min(total_len);
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
                let remaining = total_len.saturating_sub(processed) as f64;
                let eta = if speed > 0.0 {
                    format!("{:.0}s", remaining / speed)
                } else {
                    "--".to_string()
                };
                let peers_active = snapshot.peers_active.min(snapshot.peers_total);
                let msg = format_progress_message(
                    snapshot.status,
                    peers_active,
                    snapshot.peers_total,
                    snapshot.queue,
                    snapshot.inflight,
                    snapshot.failed,
                    speed,
                    &eta,
                );
                bar.set_message(msg);
                bar.set_position(processed);
                
                // Transition to follow mode when initial sync completes
                if processed >= total_len {
                    initial_sync_done = true;
                    bar.finish_and_clear();
                    
                    // Create the live follow status bar (message only, we format the bar ourselves)
                    let fb = ProgressBar::with_draw_target(
                        Some(100),
                        ProgressDrawTarget::stderr_with_hz(2),
                    );
                    let style = ProgressStyle::with_template("{msg}")
                        .expect("progress style");
                    fb.set_style(style);
                    follow_bar = Some(fb);
                }
            } else {
                // Live follow mode - show synced status with block number in a colored bar
                if let Some(ref fb) = follow_bar {
                    let head_block = snapshot.head_block;
                    let head_seen = snapshot.head_seen;
                    let peers_active = snapshot.peers_active.min(snapshot.peers_total);
                    let peers_total = snapshot.peers_total;
                    
                    let status_str = if snapshot.status == SyncStatus::Following {
                        "Synced"
                    } else if snapshot.status == SyncStatus::Fetching {
                        "Catching up"
                    } else if snapshot.status == SyncStatus::LookingForPeers {
                        "Waiting for peers"
                    } else {
                        snapshot.status.as_str()
                    };

                    let catching_up =
                        snapshot.status == SyncStatus::Fetching || snapshot.status == SyncStatus::Finalizing;
                    let catchup_suffix = if catching_up {
                        let start = catchup_start_block.get_or_insert(head_block);
                        let done = head_block.saturating_sub(*start);
                        let remaining = head_seen.saturating_sub(head_block);
                        Some(format!(" {done}/{remaining}"))
                    } else {
                        catchup_start_block = None;
                        None
                    };
                    
                    // Format block number centered in a fixed-width field
                    let content = match catchup_suffix {
                        Some(suffix) => format!("[ {} ]{}", head_block, suffix),
                        None => format!("[ {} ]", head_block),
                    };
                    let bar_width: usize = 40; // Match main progress bar width
                    let padding = bar_width.saturating_sub(content.len());
                    let left_pad = padding / 2;
                    let right_pad = padding - left_pad;
                    
                    // ANSI: white text (97) on green background (42), then reset (0)
                    let bar = format!(
                        "\x1b[97;42m{:>width_l$}{}{:<width_r$}\x1b[0m",
                        "", content, "",
                        width_l = left_pad,
                        width_r = right_pad,
                    );
                    
                    let msg = format!(
                        "{} {} | head {} | peers {}/{} | failed {}",
                        bar,
                        status_str,
                        head_seen,
                        peers_active,
                        peers_total,
                        failed,
                    );
                    fb.set_message(msg);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
        use super::{format_progress_message, select_start_from, total_blocks_to_head, SyncStatus};

    #[test]
    fn start_from_respects_start_block() {
        assert_eq!(select_start_from(100, None), 100);
        assert_eq!(select_start_from(100, Some(50)), 100);
        assert_eq!(select_start_from(100, Some(100)), 101);
        assert_eq!(select_start_from(100, Some(150)), 151);
    }

    #[test]
    fn total_blocks_handles_empty_range() {
        assert_eq!(total_blocks_to_head(10, 9), 0);
        assert_eq!(total_blocks_to_head(10, 10), 1);
        assert_eq!(total_blocks_to_head(10, 12), 3);
    }

    #[test]
    fn progress_message_formats_status() {
        assert_eq!(
            format_progress_message(SyncStatus::Fetching, 2, 5, 10, 1, 0, 1.5, "12s"),
            "status fetching | peers 2/5 | queue 10 | inflight 1 | failed 0 | speed 1.5/s | eta 12s"
        );
    }
}
