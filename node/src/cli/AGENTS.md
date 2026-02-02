# cli

## Purpose

Defines the CLI surface and configuration defaults for the node binary. Centralizes tunables for
RPC limits, fast-sync scheduling, storage sharding, logging, and operational modes. The central
`NodeConfig` struct (40+ configurable parameters) drives all node behavior.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Clap-based CLI parser and config model (`NodeConfig`) plus helper functions for range selection.
  - **Key items**: `NodeConfig`, `DbCommand`, `DbStatsArgs`, `DbCompactArgs`, `compute_target_range()`, `DEFAULT_SHARD_SIZE`, `DEFAULT_RPC_MAX_BLOCKS_PER_FILTER`

## Key APIs (no snippets)
- **Types / Enums**: `NodeConfig`, `RetentionMode`, `HeadSource`, `ReorgStrategy`, `Command`, `DbCommand`, `DbStatsArgs`, `DbCompactArgs`
- **Functions**: `NodeConfig::from_args()`, `NodeConfig::normalize()`, `compute_target_range(start_block, end_block, head_at_startup, rollback_window)`
- **Constants**: `DEFAULT_START_BLOCK`, `DEFAULT_SHARD_SIZE`, `DEFAULT_FAST_SYNC_CHUNK_SIZE`, `DEFAULT_RPC_MAX_BLOCKS_PER_FILTER`, and 20+ more

## Relationships
- **Depends on**: `clap` (v4, derive macros), `serde` (serialization for reports)
- **Used by**: `node/src/main.rs` (mode selection and startup); `node/src/rpc` (`RpcConfig` derives from RPC limit fields);
  `node/src/storage` (meta validation uses `StorageConfigKey::from(&NodeConfig)`); `test_utils::base_config()` constructs test fixtures.
- **Data/control flow**:
  1. `main.rs` calls `NodeConfig::from_args()` at startup
  2. Config propagated to storage (`shard_size`, `data_dir`), sync (chunk sizes, rollback), RPC (limits, bind), logging (verbosity, filters), UI (`no_tui`)
  3. Subcommands dispatched to `run::handle_db_stats()` / `run::handle_repair()` / `run::handle_db_compact()`

## Files (detailed)

### `mod.rs`
- **Role**: Complete CLI and configuration implementation (~371 lines, including tests).
- **Key items**: `NodeConfig`, `RetentionMode` (Full), `HeadSource` (P2p), `ReorgStrategy` (Delete), `Command`, `DbCommand`, `DbStatsArgs`, `DbCompactArgs`, `compute_target_range`, `normalize`
- **Interactions**: Consumed by `main.rs` via `NodeConfig::from_args()`. Feeds configuration into all subsystems: storage, sync, rpc, logging, run, ui.
- **Knobs / invariants**:
  - Sync: `start_block` (10M), `end_block` (optional), `shard_size` (10K), `rollback_window` (64)
  - RPC: `rpc_bind` (127.0.0.1:8545), `max_request_body_bytes` (10MB), `max_response_body_bytes` (100MB), `max_connections` (100), `max_batch_requests` (100), `max_blocks_per_filter` (10K), `max_logs_per_response` (100K)
  - Fast-sync: `fast_sync_chunk_size` (32), `fast_sync_chunk_max` (optional, defaults to 4x chunk-size), `fast_sync_max_inflight` (32), `fast_sync_max_buffered_blocks` (8192)
  - DB: `db_write_batch_blocks` (512), `db_write_flush_interval_ms` (optional), `defer_compaction` (false)
  - UI: `no_tui`, `min_peers` (1), `verbosity` (0-3)
  - Logging: `log` (master flag), `log_trace`, `log_events`, `log_events_verbose`, `log_json`, `log_report`, `log_resources`
  - `normalize()`: `--log` enables all log outputs (`log_trace`, `log_events`, `log_json`, `log_report`, `log_resources`)
  - `compute_target_range()`: clamps the requested end to `safe_head = head_at_startup - rollback_window` and returns an inclusive range

## CLI Flags Overview

### Core Node Configuration
- `--chain-id` (default: 1), `--data-dir` (default: "data"), `--peer-cache-dir` (optional)
- `--rpc-bind` (default: "127.0.0.1:8545")
- `--start-block` (default: 10,000,000), `--end-block` (optional)
- `--shard-size` (default: 10,000)

### Operational Modes
- `--min-peers` (default: 1), `--repair` (flag), `--no-tui` (flag)
- `--rollback-window` (default: 64)
- `--defer-compaction` (flag) - Skip inline shard compaction during fast-sync; compact at finalize only or via `db compact`

### Logging & Output
- `-v`/`-vv`/`-vvv` - Increasing verbosity
- `--log` (convenience: enables all log outputs), `--run-name` (optional), `--log-output-dir` (default: "logs")
- `--log-trace`, `--log-events`, `--log-json`, `--log-report`, `--log-resources`
- `--log-events-verbose` - Include high-volume events (ProcessStart/End, FetchStart/End, BatchAssigned) in the event log
- `--log-trace-filter`, `--log-json-filter`, `--log-trace-include-args`, `--log-trace-include-locations`

### RPC Configuration
- `--rpc-max-request-body-bytes` (10 MB), `--rpc-max-response-body-bytes` (100 MB)
- `--rpc-max-connections` (100), `--rpc-max-batch-requests` (100)
- `--rpc-max-blocks-per-filter` (10,000), `--rpc-max-logs-per-response` (100,000)

### Fast Sync
- `--fast-sync-chunk-size` (32), `--fast-sync-chunk-max` (optional, defaults to 4x chunk-size)
- `--fast-sync-max-inflight` (32), `--fast-sync-max-buffered-blocks` (8,192)

### Database Writer
- `--db-write-batch-blocks` (512), `--db-write-flush-interval-ms` (optional)

### Subcommands
- `db stats` - Print storage statistics (`DbStatsArgs`: `--data-dir`, `--json`)
- `db compact` - Compact all dirty shards and seal completed ones (`DbCompactArgs`: `--data-dir`)

## End-to-end flow (high level)
1. Parse argv into `NodeConfig` via clap derive.
2. Apply defaults for missing flags (RPC limits, fast-sync scheduling, log outputs).
3. Call `NodeConfig::normalize()` to propagate convenience flags like `--log`.
4. If subcommand present (`db stats` / `db compact`), dispatch to handler and return.
5. If `--repair` flag, run `Storage::repair()` and return.
6. Otherwise, pass full config to `run::run_sync()` for main operation.
7. Derive the "safe" historical range via `compute_target_range()` after peer head discovery.
8. Pass config into storage opening (`data_dir`, `shard_size`), P2P startup, RPC server limits, and sync pipeline knobs.

## Special Modes

### `--repair` Flag
Runs storage repair/recovery without starting sync:
- Scans all shards for interrupted compactions (based on `compaction_phase` in `shard.json`)
- Recovers from interrupted write/swap/cleanup phases
- Cleans up orphan tmp/old files
- Prints a summary and exits

### `--no-tui` Flag
Disables the fullscreen ratatui TUI dashboard. Falls back to legacy indicatif progress bars on stderr.

### `--defer-compaction` Flag
Skips inline shard compaction during fast-sync. Shards are compacted only at the finalize phase or manually via the `db compact` subcommand. Useful for maximizing ingest throughput during initial sync.

### `db compact` Subcommand
Standalone command to compact dirty shards and seal completed ones. Accepts `--data-dir` to override the data directory. Useful after running with `--defer-compaction` or for maintenance.
