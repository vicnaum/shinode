# cli

## Purpose
Defines the CLI surface and configuration defaults for the node binary. Centralizes tunables for
RPC limits, fast-sync scheduling, storage sharding, logging, and operational modes.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Clap-based CLI parser and config model (`NodeConfig`) plus helper functions for range selection.
  - **Key items**: `NodeConfig`, `DbCommand`, `DbStatsArgs`, `compute_target_range()`, `DEFAULT_SHARD_SIZE`, `DEFAULT_RPC_MAX_BLOCKS_PER_FILTER`

## Key APIs (no snippets)
- **Types / Enums**: `NodeConfig`, `RetentionMode`, `HeadSource`, `ReorgStrategy`, `Command`, `DbCommand`, `DbStatsArgs`
- **Functions**: `NodeConfig::from_args()`, `NodeConfig::normalize()`, `compute_target_range()`

## Relationships
- **Used by**: `node/src/main.rs` (mode selection and startup); `node/src/rpc` (`RpcConfig` derives from RPC limit fields);
  `node/src/storage` (meta validation uses `StorageConfigKey::from(&NodeConfig)`).

## Files (detailed)

### `mod.rs`
- **Role**: Defines the stable "contract" between CLI flags and runtime behavior by mapping flags into `NodeConfig` and providing well-named default constants.
- **Key items**: `NodeConfig`, `DEFAULT_START_BLOCK` (10,000,000), `DEFAULT_SHARD_SIZE` (10,000), `DEFAULT_FAST_SYNC_CHUNK_SIZE` (32), `DEFAULT_FAST_SYNC_MAX_INFLIGHT` (32), `DEFAULT_FAST_SYNC_MAX_BUFFERED_BLOCKS` (8192), `DEFAULT_DB_WRITE_BATCH_BLOCKS` (512), `DEFAULT_RPC_MAX_BLOCKS_PER_FILTER` (10,000), `DEFAULT_RPC_MAX_LOGS_PER_RESPONSE` (100,000), `DEFAULT_LOG_TRACE_FILTER`, `DEFAULT_LOG_JSON_FILTER`
- **Interactions**: `compute_target_range()` is used by `main.rs` to bound historical ingest ranges based on `head_at_startup` and `rollback_window`.
- **Knobs / invariants**: `compute_target_range()` clamps the requested end to `safe_head = head_at_startup - rollback_window` and returns an inclusive range.

## CLI Flags Overview

### Core Node Configuration
- `--chain-id` (default: 1), `--data-dir` (default: "data"), `--peer-cache-dir` (optional)
- `--rpc-bind` (default: "127.0.0.1:8545")
- `--start-block` (default: 10,000,000), `--end-block` (optional)
- `--shard-size` (default: 10,000)

### Operational Modes
- `--min-peers` (default: 1), `--repair` (flag), `--no-tui` (flag)
- `--rollback-window` (default: 64)

### Logging & Output
- `-v`/`-vv`/`-vvv` - Increasing verbosity
- `--log` (convenience: enables all log outputs), `--run-name` (optional)
- `--log-trace`, `--log-events`, `--log-json`, `--log-report`, `--log-resources`
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

## End-to-end flow (high level)
- Parse argv into `NodeConfig` via clap derive.
- Apply defaults for missing flags (RPC limits, fast-sync scheduling, log outputs).
- Call `NodeConfig::normalize()` to propagate convenience flags like `--log`.
- Derive the "safe" historical range via `compute_target_range()`.
- Pass config into storage opening (`data_dir`, `shard_size`), P2P startup, RPC server limits, and sync pipeline knobs.

## Special Modes

### `--repair` Flag
Runs storage repair/recovery without starting sync:
- Scans all shards for interrupted compactions (based on `compaction_phase` in `shard.json`)
- Recovers from interrupted write/swap/cleanup phases
- Cleans up orphan tmp/old files
- Prints a summary and exits

### `--no-tui` Flag
Disables the fullscreen ratatui TUI dashboard. Falls back to legacy indicatif progress bars on stderr.
