//! CLI and config handling.

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, ops::RangeInclusive, path::PathBuf};

pub const DEFAULT_RPC_MAX_REQUEST_BODY_BYTES: u32 = 10 * 1024 * 1024;
pub const DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES: u32 = 100 * 1024 * 1024;
pub const DEFAULT_RPC_MAX_CONNECTIONS: u32 = 100;
pub const DEFAULT_RPC_MAX_BATCH_REQUESTS: u32 = 100;
pub const DEFAULT_RPC_MAX_BLOCKS_PER_FILTER: u64 = 10_000;
pub const DEFAULT_RPC_MAX_LOGS_PER_RESPONSE: u64 = 100_000;
pub const DEFAULT_START_BLOCK: u64 = 10_000_000;
pub const DEFAULT_FAST_SYNC_CHUNK_SIZE: u64 = 32;
pub const DEFAULT_FAST_SYNC_MAX_INFLIGHT: u32 = 32;
pub const DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS: u64 = 8192;
pub const DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_DB_WRITE_BATCH_BLOCKS: u64 = 512;
pub const DEFAULT_SHARD_SIZE: u64 = 10_000;
pub const DEFAULT_LOG_OUTPUT_DIR: &str = "logs";
pub const DEFAULT_LOG_TRACE_FILTER: &str =
    "off,stateless_history_node=trace,reth_eth_wire::p2pstream=trace";
/// Default filter for JSON log output (DEBUG level for node, WARN for externals).
pub const DEFAULT_LOG_JSON_FILTER: &str = "warn,stateless_history_node=debug";

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

/// Top-level CLI commands.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Database/static-file inspection commands.
    #[command(subcommand)]
    Db(DbCommand),
}

#[derive(Subcommand, Debug, Clone)]
pub enum DbCommand {
    /// Print storage statistics.
    Stats(DbStatsArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DbStatsArgs {
    /// Override data directory for stats.
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
    /// Print JSON instead of a table.
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

/// Stateless history node configuration.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "stateless-history-node", about = "Stateless history node v0.2.0")]
#[expect(clippy::struct_excessive_bools, reason = "CLI config needs many boolean flags")]
pub struct NodeConfig {
    /// Chain ID to expose over RPC.
    #[arg(long, default_value_t = 1)]
    pub chain_id: u64,
    /// Base data directory for node storage.
    #[arg(long, default_value = "data")]
    pub data_dir: PathBuf,
    /// Optional directory for persisted peer cache (shared across runs).
    ///
    /// If unset, defaults to the data directory unless overridden by the binary.
    #[arg(long)]
    pub peer_cache_dir: Option<PathBuf>,
    /// JSON-RPC bind address.
    #[arg(long, default_value = "127.0.0.1:8545")]
    pub rpc_bind: SocketAddr,
    /// First block to backfill.
    #[arg(long, default_value_t = DEFAULT_START_BLOCK)]
    pub start_block: u64,
    /// Shard size (blocks per shard) for storage v2.
    #[arg(long, default_value_t = DEFAULT_SHARD_SIZE)]
    pub shard_size: u64,
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
    /// Optional run name (used in log artifact filenames).
    #[arg(long)]
    pub run_name: Option<String>,
    /// Output directory for log artifacts (summary JSON, traces, events).
    #[arg(long, default_value = DEFAULT_LOG_OUTPUT_DIR)]
    pub log_output_dir: PathBuf,
    /// Enable all log outputs (trace, events, json, report).
    #[arg(long, default_value_t = false)]
    pub log: bool,
    /// Emit a Chrome trace file during sync.
    #[arg(long, default_value_t = false)]
    pub log_trace: bool,
    /// EnvFilter-style filter for the trace layer.
    ///
    /// Defaults to a minimal filter that excludes `net::*` targets.
    #[arg(long, default_value = DEFAULT_LOG_TRACE_FILTER)]
    pub log_trace_filter: String,
    /// Include span/event args in trace output.
    #[arg(long, default_value_t = true)]
    pub log_trace_include_args: bool,
    /// Include file+line in trace output.
    #[arg(long, default_value_t = true)]
    pub log_trace_include_locations: bool,
    /// Emit a JSONL event log during sync.
    #[arg(long, default_value_t = false)]
    pub log_events: bool,
    /// Emit a JSONL log file (tracing output) during sync.
    #[arg(long, default_value_t = false)]
    pub log_json: bool,
    /// EnvFilter-style filter for the JSON log layer.
    ///
    /// Defaults to DEBUG for the node, WARN for external crates.
    #[arg(long, default_value = DEFAULT_LOG_JSON_FILTER)]
    pub log_json_filter: String,
    /// Emit a JSON run report after sync completes.
    #[arg(long, default_value_t = false)]
    pub log_report: bool,
    /// Emit a separate JSONL file for resource metrics (CPU, memory, disk).
    #[arg(long, default_value_t = false)]
    pub log_resources: bool,
    /// Minimum peers to wait for before starting sync.
    #[arg(long, default_value_t = 1)]
    pub min_peers: u64,
    /// Run storage repair/recovery without starting sync.
    ///
    /// Scans all shards for interrupted compactions and orphan files,
    /// performs recovery, and exits with a summary.
    #[arg(long, default_value_t = false)]
    pub repair: bool,
    /// Optional command.
    #[command(subcommand)]
    #[serde(skip)]
    pub command: Option<Command>,
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
    /// Initial fast sync chunk size (historical backfill only).
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_CHUNK_SIZE)]
    pub fast_sync_chunk_size: u64,
    /// Optional hard cap for fast sync chunk size (AIMD upper bound).
    ///
    /// If unset, defaults to `4x --fast-sync-chunk-size`.
    #[arg(long)]
    pub fast_sync_chunk_max: Option<u64>,
    /// Max in-flight chunk requests for fast sync.
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_MAX_INFLIGHT)]
    pub fast_sync_max_inflight: u32,
    /// Timeout (ms) for a single peer ingest batch (headers + bodies + receipts).
    ///
    /// This bounds "stuck" peers so work can move to other peers.
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS)]
    pub fast_sync_batch_timeout_ms: u64,
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
        let mut config = Self::parse();
        config.normalize();
        config
    }

    /// Normalize config by propagating convenience flags.
    ///
    /// If `--log` is set, enables all log outputs.
    pub fn normalize(&mut self) {
        if self.log {
            self.log_trace = true;
            self.log_events = true;
            self.log_json = true;
            self.log_report = true;
            self.log_resources = true;
        }
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
        assert_eq!(config.peer_cache_dir, None);
        assert_eq!(config.rpc_bind, "127.0.0.1:8545".parse().unwrap());
        assert_eq!(config.start_block, DEFAULT_START_BLOCK);
        assert_eq!(config.shard_size, DEFAULT_SHARD_SIZE);
        assert_eq!(config.end_block, None);
        assert_eq!(config.rollback_window, 64);
        assert_eq!(config.retention_mode, RetentionMode::Full);
        assert_eq!(config.head_source, HeadSource::P2p);
        assert_eq!(config.reorg_strategy, ReorgStrategy::Delete);
        assert_eq!(config.verbosity, 0);
        assert_eq!(config.run_name, None);
        assert!(!config.log);
        assert_eq!(config.log_output_dir, PathBuf::from(DEFAULT_LOG_OUTPUT_DIR));
        assert!(!config.log_trace);
        assert_eq!(config.log_trace_filter, DEFAULT_LOG_TRACE_FILTER);
        assert!(config.log_trace_include_args);
        assert!(config.log_trace_include_locations);
        assert!(!config.log_events);
        assert!(!config.log_json);
        assert_eq!(config.log_json_filter, DEFAULT_LOG_JSON_FILTER);
        assert!(!config.log_report);
        assert!(!config.log_resources);
        assert_eq!(config.min_peers, 1);
        assert!(config.command.is_none());
        assert_eq!(
            config.rpc_max_request_body_bytes,
            DEFAULT_RPC_MAX_REQUEST_BODY_BYTES
        );
        assert_eq!(
            config.rpc_max_response_body_bytes,
            DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES
        );
        assert_eq!(config.rpc_max_connections, DEFAULT_RPC_MAX_CONNECTIONS);
        assert_eq!(
            config.rpc_max_batch_requests,
            DEFAULT_RPC_MAX_BATCH_REQUESTS
        );
        assert_eq!(
            config.rpc_max_blocks_per_filter,
            DEFAULT_RPC_MAX_BLOCKS_PER_FILTER
        );
        assert_eq!(
            config.rpc_max_logs_per_response,
            DEFAULT_RPC_MAX_LOGS_PER_RESPONSE
        );
        assert_eq!(config.fast_sync_chunk_size, DEFAULT_FAST_SYNC_CHUNK_SIZE);
        assert_eq!(config.fast_sync_chunk_max, None);
        assert_eq!(
            config.fast_sync_max_inflight,
            DEFAULT_FAST_SYNC_MAX_INFLIGHT
        );
        assert_eq!(
            config.fast_sync_batch_timeout_ms,
            DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS
        );
        assert_eq!(
            config.fast_sync_max_buffered_blocks,
            DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS
        );
        assert_eq!(config.db_write_batch_blocks, DEFAULT_DB_WRITE_BATCH_BLOCKS);
        assert_eq!(config.db_write_flush_interval_ms, None);
    }

    #[test]
    fn parse_db_stats_command() {
        let config = NodeConfig::parse_from(["stateless-history-node", "db", "stats"]);
        assert!(matches!(
            config.command,
            Some(Command::Db(DbCommand::Stats(_)))
        ));
    }

    #[test]
    fn log_flag_enables_all_log_outputs() {
        let mut config = NodeConfig::parse_from(["stateless-history-node", "--log"]);
        config.normalize();
        assert!(config.log);
        assert!(config.log_trace);
        assert!(config.log_events);
        assert!(config.log_json);
        assert!(config.log_report);
        assert!(config.log_resources);
    }

    #[test]
    fn repair_flag_parses() {
        let config = NodeConfig::parse_from(["stateless-history-node", "--repair"]);
        assert!(config.repair);
    }
}
