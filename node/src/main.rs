mod chain;
mod cli;
mod metrics;
mod p2p;
mod rpc;
mod storage;
mod sync;

use cli::{compute_target_range, BenchmarkMode, NodeConfig};
use eyre::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use metrics::{lag_to_head, range_len, rate_per_sec};
use std::{
    collections::VecDeque,
    io::IsTerminal,
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
use sync::historical::IngestBenchStats;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

impl sync::ProgressReporter for ProgressBar {
    fn set_length(&self, len: u64) {
        self.set_length(len);
    }

    fn inc(&self, delta: u64) {
        self.inc(delta);
    }

    fn finish(&self) {
        self.finish_and_clear();
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

    fn finish(&self) {}
}
#[tokio::main]
async fn main() -> Result<()> {
    let config = NodeConfig::from_args();
    init_tracing(config.verbosity);

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
        sync::historical::run_benchmark_probe(
            &config,
            Arc::clone(&session.pool),
            range,
            head_at_startup,
        )
            .await?;
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
                stats.set_peers(session.pool.len() as u64, session.pool.len() as u64);
                spawn_progress_updater(
                    bar.clone(),
                    Arc::clone(stats),
                    total_len,
                    Arc::clone(&session.pool),
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
        let outcome = sync::historical::run_ingest_pipeline(
            Arc::clone(&storage),
            Arc::clone(&session.pool),
            &config,
            range.clone(),
            head_at_startup,
            progress_ref,
            progress_stats.clone(),
            Some(Arc::clone(&bench)),
        )
        .await?;

        let logs_total = match outcome {
            sync::historical::IngestPipelineOutcome::RangeApplied { logs, .. } => logs,
            sync::historical::IngestPipelineOutcome::UpToDate { .. } => 0,
        };
        let summary = bench.summary(
            *range.start(),
            *range.end(),
            head_at_startup,
            config.rollback_window > 0,
            session.pool.len() as u64,
            logs_total,
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
                stats.set_peers(session.pool.len() as u64, session.pool.len() as u64);
                spawn_progress_updater(
                    bar.clone(),
                    Arc::clone(stats),
                    total_len,
                    Arc::clone(&session.pool),
                );
            }
            Some(Arc::new(IngestProgress::new(bar, total_len)))
        } else {
            None
        };

        loop {
            let ingest_started = Instant::now();
            let head_at_startup = source.head().await?;
            let last_indexed = storage.last_indexed_block()?;
            let start_from = select_start_from(config.start_block, last_indexed);
            let range = compute_target_range(
                start_from,
                config.end_block,
                head_at_startup,
                config.rollback_window,
            );
            let progress_ref = progress
                .as_ref()
                .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
            let outcome = sync::historical::run_ingest_pipeline(
                Arc::clone(&storage),
                Arc::clone(&session.pool),
                &config,
                range.clone(),
                head_at_startup,
                progress_ref,
                progress_stats.clone(),
                None,
            )
            .await?;
            let elapsed = ingest_started.elapsed();
            let head_seen = storage.head_seen()?;
            if let (Some(tracker), Some(head_seen)) = (progress.as_ref(), head_seen) {
                let last_indexed = storage.last_indexed_block()?;
                let start_from = select_start_from(config.start_block, last_indexed);
                let range = compute_target_range(
                    start_from,
                    config.end_block,
                    head_seen,
                    config.rollback_window,
                );
                tracker.set_length(range_len(&range));
            }
            let last_indexed = storage.last_indexed_block()?;
            let lag = lag_to_head(head_seen, last_indexed);
            let peers = session.pool.len();
            if let Some(stats) = progress_stats.as_ref() {
                stats.set_peers(peers as u64, peers as u64);
                if matches!(outcome, sync::historical::IngestPipelineOutcome::UpToDate { .. }) {
                    stats.set_status(SyncStatus::UpToDate);
                }
            }
            match outcome {
                sync::historical::IngestPipelineOutcome::UpToDate { head } => {
                    info!(
                        head,
                        lag_blocks = ?lag,
                        peers,
                        elapsed_ms = elapsed.as_millis(),
                        "ingest up to date"
                    );
                    break;
                }
                sync::historical::IngestPipelineOutcome::RangeApplied { range, logs } => {
                    let blocks = range_len(&range);
                    let blocks_per_sec = rate_per_sec(blocks, elapsed);
                    let logs_per_sec = rate_per_sec(logs, elapsed);
                    tracing::debug!(
                        range_start = *range.start(),
                        range_end = *range.end(),
                        blocks,
                        logs,
                        blocks_per_sec = ?blocks_per_sec,
                        logs_per_sec = ?logs_per_sec,
                        lag_blocks = ?lag,
                        peers,
                        elapsed_ms = elapsed.as_millis(),
                        "ingested range"
                    );
                }
            }
        }

        if let Some(tracker) = progress.as_ref() {
            tracker.finish();
        }

        _network_session = Some(session);
    }

    tokio::signal::ctrl_c().await?;
    warn!("shutdown signal received");
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

fn spawn_progress_updater(
    bar: ProgressBar,
    stats: Arc<SyncProgressStats>,
    total_len: u64,
    peer_pool: Arc<PeerPool>,
) {
    tokio::spawn(async move {
        let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        loop {
            ticker.tick().await;
            let peers = peer_pool.len() as u64;
            stats.set_peers(peers, peers);
            let snapshot = stats.snapshot();
            let processed = snapshot.processed.min(total_len);
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
            if snapshot.status == SyncStatus::UpToDate
                && snapshot.queue == 0
                && snapshot.inflight == 0
            {
                break;
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
