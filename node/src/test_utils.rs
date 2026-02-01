use crate::cli::{
    HeadSource, NodeConfig, ReorgStrategy, RetentionMode, DEFAULT_DB_WRITE_BATCH_BLOCKS,
    DEFAULT_FAST_SYNC_CHUNK_SIZE, DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS,
    DEFAULT_FAST_SYNC_MAX_INFLIGHT, DEFAULT_LOG_JSON_FILTER, DEFAULT_LOG_OUTPUT_DIR,
    DEFAULT_LOG_TRACE_FILTER, DEFAULT_RPC_MAX_BATCH_REQUESTS, DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
    DEFAULT_RPC_MAX_CONNECTIONS, DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
    DEFAULT_RPC_MAX_REQUEST_BODY_BYTES, DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES, DEFAULT_SHARD_SIZE,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn temp_dir(prefix: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time moves forward")
        .as_nanos();
    let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut path = std::env::temp_dir();
    path.push(format!(
        "stateless-history-node-{prefix}-test-{now}-{}-{suffix}",
        std::process::id()
    ));
    path
}

pub fn base_config(data_dir: PathBuf) -> NodeConfig {
    NodeConfig {
        chain_id: 1,
        data_dir,
        peer_cache_dir: None,
        rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
        start_block: 0,
        shard_size: DEFAULT_SHARD_SIZE,
        end_block: None,
        rollback_window: 64,
        retention_mode: RetentionMode::Full,
        head_source: HeadSource::P2p,
        reorg_strategy: ReorgStrategy::Delete,
        verbosity: 0,
        run_name: None,
        log: false,
        log_output_dir: PathBuf::from(DEFAULT_LOG_OUTPUT_DIR),
        log_trace: false,
        log_trace_filter: DEFAULT_LOG_TRACE_FILTER.to_string(),
        log_trace_include_args: false,
        log_trace_include_locations: false,
        log_events: false,
        log_events_verbose: false,
        log_json: false,
        log_json_filter: DEFAULT_LOG_JSON_FILTER.to_string(),
        log_report: false,
        log_resources: false,
        min_peers: 1,
        repair: false,
        no_tui: false,
        command: None,
        rpc_max_request_body_bytes: DEFAULT_RPC_MAX_REQUEST_BODY_BYTES,
        rpc_max_response_body_bytes: DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES,
        rpc_max_connections: DEFAULT_RPC_MAX_CONNECTIONS,
        rpc_max_batch_requests: DEFAULT_RPC_MAX_BATCH_REQUESTS,
        rpc_max_blocks_per_filter: DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
        rpc_max_logs_per_response: DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
        fast_sync_chunk_size: DEFAULT_FAST_SYNC_CHUNK_SIZE,
        fast_sync_chunk_max: None,
        fast_sync_max_inflight: DEFAULT_FAST_SYNC_MAX_INFLIGHT,
        fast_sync_max_buffered_blocks: DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS,
        db_write_batch_blocks: DEFAULT_DB_WRITE_BATCH_BLOCKS,
        db_write_flush_interval_ms: None,
        defer_compaction: false,
    }
}
