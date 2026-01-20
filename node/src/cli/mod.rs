//! CLI and config handling.

use clap::{ArgAction, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, ops::RangeInclusive, path::PathBuf};

pub const DEFAULT_RPC_MAX_REQUEST_BODY_BYTES: u32 = 10 * 1024 * 1024;
pub const DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES: u32 = 100 * 1024 * 1024;
pub const DEFAULT_RPC_MAX_CONNECTIONS: u32 = 100;
pub const DEFAULT_RPC_MAX_BATCH_REQUESTS: u32 = 100;
pub const DEFAULT_RPC_MAX_BLOCKS_PER_FILTER: u64 = 10_000;
pub const DEFAULT_RPC_MAX_LOGS_PER_RESPONSE: u64 = 100_000;
pub const DEFAULT_FAST_SYNC_CHUNK_SIZE: u64 = 32;
pub const DEFAULT_FAST_SYNC_MAX_INFLIGHT: u32 = 15;
pub const DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS: u64 = 2048;
pub const DEFAULT_DB_WRITE_BATCH_BLOCKS: u64 = 512;

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

/// Benchmark mode for probe runs.
#[derive(ValueEnum, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkMode {
    /// Normal operation (no benchmark).
    Disabled,
    /// Harness-like probe mode (headers + receipts only, no DB, no RPC).
    Probe,
    /// Full ingest benchmark (full processing + DB writes, exits after range).
    Ingest,
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
    /// Optional final block to stop at (range-limited sync).
    #[arg(long)]
    pub end_block: Option<u64>,
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
    /// Increase log verbosity (-v, -vv, -vvv).
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbosity: u8,
    /// Benchmark mode (probe only, exits after range).
    #[arg(long, value_enum, default_value_t = BenchmarkMode::Disabled)]
    pub benchmark: BenchmarkMode,
    /// Max JSON-RPC request body size in bytes.
    #[arg(long, default_value_t = DEFAULT_RPC_MAX_REQUEST_BODY_BYTES)]
    pub rpc_max_request_body_bytes: u32,
    /// Max JSON-RPC response body size in bytes.
    #[arg(long, default_value_t = DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES)]
    pub rpc_max_response_body_bytes: u32,
    /// Max concurrent JSON-RPC connections.
    #[arg(long, default_value_t = DEFAULT_RPC_MAX_CONNECTIONS)]
    pub rpc_max_connections: u32,
    /// Max JSON-RPC batch size (0 = unlimited).
    #[arg(long, default_value_t = DEFAULT_RPC_MAX_BATCH_REQUESTS)]
    pub rpc_max_batch_requests: u32,
    /// Max blocks per eth_getLogs filter (0 = unlimited).
    #[arg(long, default_value_t = DEFAULT_RPC_MAX_BLOCKS_PER_FILTER)]
    pub rpc_max_blocks_per_filter: u64,
    /// Max logs per eth_getLogs response (0 = unlimited).
    #[arg(long, default_value_t = DEFAULT_RPC_MAX_LOGS_PER_RESPONSE)]
    pub rpc_max_logs_per_response: u64,
    /// Fast sync chunk size (historical backfill only).
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_CHUNK_SIZE)]
    pub fast_sync_chunk_size: u64,
    /// Max in-flight chunk requests for fast sync.
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_MAX_INFLIGHT)]
    pub fast_sync_max_inflight: u32,
    /// Max buffered blocks (across completed chunks) for fast sync.
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS)]
    pub fast_sync_max_buffered_blocks: u64,
    /// DB writer batch size for ingest mode.
    #[arg(long, default_value_t = DEFAULT_DB_WRITE_BATCH_BLOCKS)]
    pub db_write_batch_blocks: u64,
    /// Optional DB writer flush interval (ms) for ingest mode.
    #[arg(long)]
    pub db_write_flush_interval_ms: Option<u64>,
}

impl NodeConfig {
    /// Parse configuration from CLI args.
    pub fn from_args() -> Self {
        Self::parse()
    }
}

/// Compute the target range for historical sync/benchmark runs.
pub fn compute_target_range(
    start_block: u64,
    end_block: Option<u64>,
    head_at_startup: u64,
    rollback_window: u64,
) -> RangeInclusive<u64> {
    let safe_head = head_at_startup.saturating_sub(rollback_window);
    let requested_end = end_block.unwrap_or(safe_head);
    let end = requested_end.min(safe_head);
    start_block..=end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_contract() {
        let config = NodeConfig::parse_from(["stateless-history-node"]);

        assert_eq!(config.chain_id, 1);
        assert_eq!(config.data_dir, PathBuf::from("data"));
        assert_eq!(config.rpc_bind, "127.0.0.1:8545".parse().unwrap());
        assert_eq!(config.start_block, 0);
        assert_eq!(config.end_block, None);
        assert_eq!(config.rollback_window, 64);
        assert_eq!(config.retention_mode, RetentionMode::Full);
        assert_eq!(config.head_source, HeadSource::P2p);
        assert_eq!(config.reorg_strategy, ReorgStrategy::Delete);
        assert_eq!(config.verbosity, 0);
        assert_eq!(config.benchmark, BenchmarkMode::Disabled);
        assert_eq!(config.rpc_max_request_body_bytes, DEFAULT_RPC_MAX_REQUEST_BODY_BYTES);
        assert_eq!(config.rpc_max_response_body_bytes, DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES);
        assert_eq!(config.rpc_max_connections, DEFAULT_RPC_MAX_CONNECTIONS);
        assert_eq!(config.rpc_max_batch_requests, DEFAULT_RPC_MAX_BATCH_REQUESTS);
        assert_eq!(config.rpc_max_blocks_per_filter, DEFAULT_RPC_MAX_BLOCKS_PER_FILTER);
        assert_eq!(config.rpc_max_logs_per_response, DEFAULT_RPC_MAX_LOGS_PER_RESPONSE);
        assert_eq!(config.fast_sync_chunk_size, DEFAULT_FAST_SYNC_CHUNK_SIZE);
        assert_eq!(config.fast_sync_max_inflight, DEFAULT_FAST_SYNC_MAX_INFLIGHT);
        assert_eq!(
            config.fast_sync_max_buffered_blocks,
            DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS
        );
        assert_eq!(config.db_write_batch_blocks, DEFAULT_DB_WRITE_BATCH_BLOCKS);
        assert_eq!(config.db_write_flush_interval_ms, None);
    }
}
