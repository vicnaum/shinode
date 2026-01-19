mod chain;
mod cli;
mod p2p;
mod rpc;
mod storage;
mod sync;

use cli::NodeConfig;
use eyre::Result;
use sync::IngestOutcome;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const DEFAULT_SYNC_BATCH_SIZE: u64 = 100;

#[tokio::main]
async fn main() -> Result<()> {
    let config = NodeConfig::from_args();
    init_tracing();

    info!(
        chain_id = config.chain_id,
        rpc_bind = %config.rpc_bind,
        data_dir = %config.data_dir.display(),
        "starting stateless history node"
    );

    let storage = storage::Storage::open(&config)?;
    let head_seen = storage.head_seen()?;
    let last_indexed = storage.last_indexed_block()?;
    info!(
        head_seen = ?head_seen,
        last_indexed = ?last_indexed,
        "sync checkpoints loaded"
    );

    let rpc_handle = rpc::start(config.rpc_bind, config.chain_id).await?;
    info!(rpc_bind = %config.rpc_bind, "rpc server started");

    let mut _network_session = None;
    if matches!(config.head_source, cli::HeadSource::P2p) {
        info!("starting p2p network");
        let session = p2p::connect_mainnet_peers().await?;
        info!(peers = session.pool.len(), "p2p peers connected");

        let source = p2p::MultiPeerBlockPayloadSource::new(session.pool.clone());
        let mut ingest = sync::IngestRunner::new(source, DEFAULT_SYNC_BATCH_SIZE);
        match ingest.run_once(&storage, config.start_block).await? {
            IngestOutcome::UpToDate { head } => {
                info!(head, "ingest up to date");
            }
            IngestOutcome::RangeApplied { range, logs } => {
                info!(
                    range_start = *range.start(),
                    range_end = *range.end(),
                    logs = logs.len(),
                    "ingested range"
                );
            }
            IngestOutcome::Reorg { ancestor_number } => {
                warn!(ancestor_number, "reorg detected during ingest");
            }
        }

        _network_session = Some(session);
    }

    tokio::signal::ctrl_c().await?;
    warn!("shutdown signal received");
    drop(rpc_handle);

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
