//! CLI and config handling.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

/// Stateless history node configuration.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "stateless-history-node", about = "Stateless history node v0.1")]
pub struct NodeConfig {
    /// Chain ID to expose over RPC.
    #[arg(long, default_value_t = 1)]
    pub chain_id: u64,
    /// Base data directory for node storage.
    #[arg(long, default_value = "data")]
    pub data_dir: PathBuf,
    /// JSON-RPC bind address.
    #[arg(long, default_value = "127.0.0.1:8545")]
    pub rpc_bind: SocketAddr,
}

impl NodeConfig {
    /// Parse configuration from CLI args.
    pub fn from_args() -> Self {
        Self::parse()
    }
}
