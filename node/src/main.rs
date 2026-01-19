mod chain;
mod cli;
mod p2p;
mod rpc;
mod storage;
mod sync;

use chain::MemoryHeadTracker;
use cli::NodeConfig;
use eyre::Result;
use sync::SyncController;
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

    let head_tracker = MemoryHeadTracker::new(head_seen);
    let controller = SyncController::new(head_tracker, DEFAULT_SYNC_BATCH_SIZE);
    match controller.next_range(&storage, config.start_block)? {
        Some(range) => info!(
            range_start = *range.start(),
            range_end = *range.end(),
            "planned sync range"
        ),
        None => info!("no sync range planned (no head or already at head)"),
    }
    let rpc_handle = rpc::start(config.rpc_bind, config.chain_id).await?;
    info!(rpc_bind = %config.rpc_bind, "rpc server started");

    tokio::signal::ctrl_c().await?;
    warn!("shutdown signal received");
    drop(rpc_handle);

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
