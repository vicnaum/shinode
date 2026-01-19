mod chain;
mod cli;
mod p2p;
mod rpc;
mod storage;
mod sync;

use cli::NodeConfig;
use eyre::Result;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

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

    let _storage = storage::Storage::open(&config)?;
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
