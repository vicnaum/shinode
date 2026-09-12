//! CLI and config handling.

use crate::chain::ChainKind;
use alloy_primitives::B256;
use clap::{ArgAction, Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use eyre::{ensure, eyre, Result};
use reth_network_peers::TrustedPeer;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, ops::RangeInclusive, path::PathBuf};

pub const DEFAULT_RPC_MAX_REQUEST_BODY_BYTES: u32 = 10 * 1024 * 1024;
pub const DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES: u32 = 100 * 1024 * 1024;
pub const DEFAULT_RPC_MAX_CONNECTIONS: u32 = 100;
pub const DEFAULT_RPC_MAX_BATCH_REQUESTS: u32 = 100;
pub const DEFAULT_RPC_MAX_BLOCKS_PER_FILTER: u64 = 10_000;
pub const DEFAULT_RPC_MAX_LOGS_PER_RESPONSE: u64 = 100_000;
pub const DEFAULT_START_BLOCK: u64 = 10_000_000;
pub const DEFAULT_P2P_PORT: u16 = 30303;
pub const DEFAULT_FAST_SYNC_CHUNK_SIZE: u64 = 32;
pub const DEFAULT_FAST_SYNC_MAX_INFLIGHT: u32 = 32;
pub const DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS: u64 = 8192;
pub const DEFAULT_DB_WRITE_BATCH_BLOCKS: u64 = 512;
pub const DEFAULT_SHARD_SIZE: u64 = 10_000;
pub const DEFAULT_LOG_OUTPUT_DIR: &str = "logs";
pub const DEFAULT_LOG_TRACE_FILTER: &str = "off,shinode=trace,reth_eth_wire::p2pstream=trace";
/// Default filter for JSON log output (DEBUG level for node, WARN for externals).
pub const DEFAULT_LOG_JSON_FILTER: &str = "warn,shinode=debug";

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
    /// Import verified Base snapshot history into a fresh database.
    SnapshotImport(crate::snapshot::args::SnapshotArgs),
    /// Database/static-file inspection commands.
    #[command(subcommand)]
    Db(DbCommand),
}

#[derive(Subcommand, Debug, Clone)]
pub enum DbCommand {
    /// Print storage statistics.
    Stats(DbStatsArgs),
    /// Compact all dirty shards and seal completed ones.
    Compact(DbCompactArgs),
    /// Rebuild the sealed shard cache for faster startup.
    RebuildCache(DbRebuildCacheArgs),
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

#[derive(Args, Debug, Clone)]
pub struct DbCompactArgs {
    /// Override data directory for compaction.
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
    /// Write JSON logs to file for debugging slow compaction.
    #[arg(long, default_value_t = false)]
    pub log_json: bool,
    /// EnvFilter for JSON log level (e.g. "warn,shinode=trace").
    #[arg(long, default_value = DEFAULT_LOG_JSON_FILTER)]
    pub log_json_filter: String,
    /// Output directory for log artifacts.
    #[arg(long, default_value = DEFAULT_LOG_OUTPUT_DIR)]
    pub log_output_dir: PathBuf,
    /// Verbose logging level (0=info, 1=debug, 2+=trace).
    #[arg(short, long, action = ArgAction::Count, default_value_t = 0)]
    pub verbose: u8,
}

#[derive(Args, Debug, Clone)]
pub struct DbRebuildCacheArgs {
    /// Override data directory.
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
}

/// Stateless history node configuration.
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
#[command(name = "shinode", about = "SHiNode – Stateless History Node v0.3.0")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "CLI config needs many boolean flags"
)]
pub struct NodeConfig {
    /// Network to index; maintenance infers the stored chain when omitted.
    #[arg(long, value_enum, default_value = "mainnet")]
    pub chain: ChainKind,
    /// Optional assertion that must match the selected network's chain ID.
    #[arg(long = "chain-id", value_name = "CHAIN_ID")]
    #[serde(skip)]
    pub chain_id_override: Option<u64>,
    /// Resolved from the selected network, never independently configurable.
    #[arg(skip = 1u64)]
    pub chain_id: u64,
    #[arg(skip)]
    pub genesis_hash: B256,
    /// TCP listening and UDP discovery port.
    #[arg(long, default_value_t = DEFAULT_P2P_PORT, value_parser = clap::value_parser!(u16).range(1..))]
    pub p2p_port: u16,
    /// Serving peer enode URL (repeat for multiple peers; DNS hosts are accepted).
    #[arg(long = "trusted-peer", value_name = "ENODE", value_parser = parse_trusted_peer)]
    pub trusted_peers: Vec<TrustedPeer>,
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
    /// First block to backfill (mainnet default: 10000000; required for a new Base database).
    #[arg(long = "start-block", value_name = "BLOCK")]
    #[serde(skip)]
    pub start_block_override: Option<u64>,
    /// Resolved lower bound; resumed Base runs retain their original range.
    #[arg(skip = DEFAULT_START_BLOCK)]
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
    /// Include high-volume events (ProcessStart/End, FetchStart/End, BatchAssigned) in the event log.
    #[arg(long, default_value_t = false)]
    pub log_events_verbose: bool,
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
    /// Disable fullscreen TUI dashboard, use legacy progress bars.
    #[arg(long, default_value_t = false)]
    pub no_tui: bool,
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
    /// Max buffered blocks (across completed chunks) for fast sync.
    #[arg(long, default_value_t = DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS)]
    pub fast_sync_max_buffered_blocks: u64,
    /// DB writer batch size for ingest mode.
    #[arg(long, default_value_t = DEFAULT_DB_WRITE_BATCH_BLOCKS)]
    pub db_write_batch_blocks: u64,
    /// Optional DB writer flush interval (ms) for ingest mode.
    #[arg(long)]
    pub db_write_flush_interval_ms: Option<u64>,
    /// Defer shard compaction during fast-sync (compact at finalize only).
    #[arg(long, default_value_t = false)]
    #[serde(default)]
    pub defer_compaction: bool,
}

impl NodeConfig {
    /// Parse configuration from CLI args.
    pub fn from_args() -> Result<Self> {
        let matches = Self::command().get_matches();
        let mut config = Self::from_arg_matches(&matches)?;
        config.normalize_selection(matches.value_source("chain") == Some(clap::parser::ValueSource::CommandLine))?;
        Ok(config)
    }

    /// Parse and resolve configuration without exiting the process.
    #[cfg(test)]
    pub fn try_from_args<I, T>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = Self::command().try_get_matches_from(args)?;
        let mut config = Self::from_arg_matches(&matches)?;
        config.normalize_selection(matches.value_source("chain") == Some(clap::parser::ValueSource::CommandLine))?;
        Ok(config)
    }

    /// Normalize config by propagating convenience flags.
    ///
    /// If `--log` is set, enables all log outputs.
    #[cfg(test)]
    pub fn normalize(&mut self) -> Result<()> {
        self.normalize_selection(true)
    }

    fn normalize_selection(&mut self, explicit_chain: bool) -> Result<()> {
        if let Some(Command::SnapshotImport(args)) = &self.command {
            ensure!(!self.repair, "--repair conflicts with snapshot-import");
            ensure!(!explicit_chain || self.chain == args.chain, "conflicting snapshot --chain selections");
            self.chain = args.chain;
            self.chain_id = self.chain.chain_id();
            self.genesis_hash = self.chain.genesis_hash();
            ensure!(self.chain_id_override.is_none_or(|id| id == self.chain_id), "--chain-id conflicts with snapshot chain");
            if let Some(output) = &args.output_dir { self.data_dir = output.clone(); }
            self.start_block = args.start_block;
            self.start_block_override = Some(args.start_block);
            self.end_block = Some(args.end_block);
            return Ok(());
        }
        if let Some(command) = &self.command {
            let override_dir = match command {
                Command::Db(DbCommand::Stats(args)) => args.data_dir.as_ref(),
                Command::Db(DbCommand::Compact(args)) => args.data_dir.as_ref(),
                Command::Db(DbCommand::RebuildCache(args)) => args.data_dir.as_ref(),
                Command::SnapshotImport(_) => None,
            };
            if let Some(dir) = override_dir { self.data_dir = dir.clone(); }
        }
        self.chain_id = self.chain.chain_id();
        self.genesis_hash = self.chain.genesis_hash();
        let sync = self.command.is_none() && !self.repair;
        if !sync {
            crate::storage::Storage::configure_maintenance(self, explicit_chain)?;
        } else if let Some(size) = crate::storage::Storage::read_shard_size(&self.data_dir)? {
            self.shard_size = size;
        }
        if let Some(asserted) = self.chain_id_override {
            ensure!(asserted == self.chain_id,
                "--chain-id {asserted} conflicts with --chain {} (chain ID {}); select the network with --chain",
                self.chain.name(), self.chain_id);
        }
        crate::storage::Storage::validate_existing(self)?;
        self.start_block = match self.start_block_override {
            Some(start) => start,
            None if self.chain == ChainKind::Mainnet => DEFAULT_START_BLOCK,
            None if !sync => 0, // Maintenance does not consume a sync range.
            None => crate::storage::Storage::read_resume_start(
                &self.data_dir, self.chain_id, self.genesis_hash,
            )?.ok_or_else(|| eyre!(
                "--start-block is required for a new Base database; select a recent range or use a serving archive peer"
            ))?,
        };
        if sync {
            ensure!(
                self.end_block.is_none_or(|end| end >= self.start_block),
                "--end-block must be at least --start-block ({})",
                self.start_block
            );
        }
        if self.log {
            self.log_trace = true;
            self.log_events = true;
            self.log_json = true;
            self.log_report = true;
            self.log_resources = true;
        }
        Ok(())
    }


}

#[cfg(test)]
mod chain_tests;

fn parse_trusted_peer(value: &str) -> std::result::Result<TrustedPeer, String> {
    if !value.starts_with("enode://") {
        return Err("trusted peers must use enode:// URLs".to_owned());
    }
    let peer: TrustedPeer = value
        .parse()
        .map_err(|error| format!("invalid trusted peer: {error}"))?;
    if peer.tcp_port == 0 || peer.udp_port == 0 {
        return Err("trusted peer ports must be nonzero".to_owned());
    }
    Ok(peer)
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
        let config = NodeConfig::parse_from(["shinode"]);

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
        assert!(!config.log_events_verbose);
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
            config.fast_sync_max_buffered_blocks,
            DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS
        );
        assert_eq!(config.db_write_batch_blocks, DEFAULT_DB_WRITE_BATCH_BLOCKS);
        assert_eq!(config.db_write_flush_interval_ms, None);
        assert!(!config.defer_compaction);
        assert!(!config.no_tui);
    }

    #[test]
    fn parse_db_stats_command() {
        let config = NodeConfig::parse_from(["shinode", "db", "stats"]);
        assert!(matches!(
            config.command,
            Some(Command::Db(DbCommand::Stats(_)))
        ));
    }

    #[test]
    fn parse_db_compact_command() {
        let config = NodeConfig::parse_from(["shinode", "db", "compact"]);
        assert!(matches!(
            config.command,
            Some(Command::Db(DbCommand::Compact(_)))
        ));
    }

    #[test]
    fn parse_db_rebuild_cache_command() {
        let config = NodeConfig::parse_from(["shinode", "db", "rebuild-cache"]);
        assert!(matches!(
            config.command,
            Some(Command::Db(DbCommand::RebuildCache(_)))
        ));
    }

    #[test]
    fn log_flag_enables_all_log_outputs() -> Result<()> {
        let mut config = NodeConfig::parse_from(["shinode", "--log"]);
        config.normalize()?;
        ensure!(
            config.log
                && config.log_trace
                && config.log_events
                && config.log_json
                && config.log_report
                && config.log_resources,
            "--log must enable every log output"
        );
        Ok(())
    }

    #[test]
    fn repair_flag_parses() {
        let config = NodeConfig::parse_from(["shinode", "--repair"]);
        assert!(config.repair);
    }
}
