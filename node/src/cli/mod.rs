//! CLI and config handling.

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

/// Retention mode for stored history.
#[derive(ValueEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RetentionMode {
    /// Store headers, tx hashes, and full receipts/logs.
    Full,
}

/// Source of the canonical head signal.
#[derive(ValueEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HeadSource {
    /// Best-effort head from the P2P view.
    P2p,
}

/// Strategy to apply on reorg rollback.
#[derive(ValueEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReorgStrategy {
    /// Delete data above the common ancestor.
    Delete,
}

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
    /// First block to backfill.
    #[arg(long, default_value_t = 0)]
    pub start_block: u64,
    /// Maximum number of blocks to roll back on reorg.
    #[arg(long, default_value_t = 64)]
    pub rollback_window: u64,
    /// Retention mode for stored history.
    #[arg(long, value_enum, default_value_t = RetentionMode::Full)]
    pub retention_mode: RetentionMode,
    /// Canonical head source.
    #[arg(long, value_enum, default_value_t = HeadSource::P2p)]
    pub head_source: HeadSource,
    /// Reorg rollback strategy.
    #[arg(long, value_enum, default_value_t = ReorgStrategy::Delete)]
    pub reorg_strategy: ReorgStrategy,
}

impl NodeConfig {
    /// Parse configuration from CLI args.
    pub fn from_args() -> Self {
        Self::parse()
    }
}
