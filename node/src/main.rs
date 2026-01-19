mod chain;
mod cli;
mod metrics;
mod p2p;
mod rpc;
mod storage;
mod sync;

use cli::NodeConfig;
use eyre::Result;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use metrics::{lag_to_head, range_len, rate_per_sec};
use std::{io::IsTerminal, sync::Arc, time::Instant};
use sync::IngestOutcome;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const DEFAULT_SYNC_BATCH_SIZE: u64 = 100;

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

#[tokio::main]
async fn main() -> Result<()> {
    let config = NodeConfig::from_args();
    init_tracing(config.verbosity);

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
        let session = p2p::connect_mainnet_peers().await?;
        info!(peers = session.pool.len(), "p2p peers connected");

        let source = p2p::MultiPeerBlockPayloadSource::new(session.pool.clone());
        let mut ingest = sync::IngestRunner::new(source, DEFAULT_SYNC_BATCH_SIZE);
                let progress = if std::io::stderr().is_terminal() {
                    let bar = ProgressBar::new(0);
                    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
                    let style = ProgressStyle::with_template(
                        "{bar:40.cyan/blue} {pos}/{len} | {elapsed_precise} | {msg}",
                    )
                    .expect("progress style");
                    bar.set_style(style);
                    bar.set_message("ingesting range");
                    Some(bar)
                } else {
                    None
                };

                let ingest_started = Instant::now();
                let outcome = match progress.as_ref() {
                    Some(bar) => ingest
                        .run_once_with_progress(storage.as_ref(), config.start_block, Some(bar))
                        .await?,
                    None => ingest.run_once(storage.as_ref(), config.start_block).await?,
                };
                let elapsed = ingest_started.elapsed();
                let head_seen = storage.head_seen()?;
                let last_indexed = storage.last_indexed_block()?;
                let lag = lag_to_head(head_seen, last_indexed);
                let peers = session.pool.len();
                match outcome {
            IngestOutcome::UpToDate { head } => {
                        info!(
                            head,
                            lag_blocks = ?lag,
                            peers,
                            elapsed_ms = elapsed.as_millis(),
                            reorgs = 0u64,
                            "ingest up to date"
                        );
            }
            IngestOutcome::RangeApplied { range, logs } => {
                        let blocks = range_len(&range);
                        let blocks_per_sec = rate_per_sec(blocks, elapsed);
                        let logs_per_sec = rate_per_sec(logs.len() as u64, elapsed);
                info!(
                    range_start = *range.start(),
                    range_end = *range.end(),
                            blocks,
                    logs = logs.len(),
                            blocks_per_sec = ?blocks_per_sec,
                            logs_per_sec = ?logs_per_sec,
                            lag_blocks = ?lag,
                            peers,
                            elapsed_ms = elapsed.as_millis(),
                            reorgs = 0u64,
                    "ingested range"
                );
            }
            IngestOutcome::Reorg { ancestor_number } => {
                        warn!(
                            ancestor_number,
                            lag_blocks = ?lag,
                            peers,
                            elapsed_ms = elapsed.as_millis(),
                            reorgs = 1u64,
                            "reorg detected during ingest"
                        );
            }
        }

        _network_session = Some(session);
    }

    tokio::signal::ctrl_c().await?;
    warn!("shutdown signal received");
    if let Err(err) = rpc_handle.stop() {
        warn!(error = %err, "failed to stop rpc server");
    }
    rpc_handle.stopped().await;
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
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
