# cli

## Purpose
Defines the CLI surface and configuration defaults for the node binary. Centralizes tunables for
RPC limits, fast-sync scheduling, storage sharding, and log output.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Clap-based CLI parser and config model (`NodeConfig`) plus helper functions for range selection.
  - **Key items**: `NodeConfig`, `compute_target_range()`, `DEFAULT_SHARD_SIZE`, `DEFAULT_RPC_MAX_BLOCKS_PER_FILTER`

## Key APIs (no snippets)
- **Types / Enums**: `NodeConfig`, `RetentionMode`, `HeadSource`, `ReorgStrategy`, `Command`, `DbCommand`, `DbStatsArgs`
- **Functions**: `NodeConfig::from_args()`, `NodeConfig::normalize()`, `compute_target_range()`

## Relationships
- **Used by**: `node/src/main.rs` (mode selection and startup); `node/src/rpc` (`RpcConfig` derives from RPC limit fields);
  `node/src/storage` (meta validation uses `StorageConfigKey::from(&NodeConfig)`).

## Files (detailed)

### `mod.rs`
- **Role**: Defines the stable "contract" between CLI flags and runtime behavior by mapping flags into `NodeConfig` and providing well-named default constants.
- **Key items**: `NodeConfig`, `DEFAULT_START_BLOCK`, `DEFAULT_SHARD_SIZE`, `DEFAULT_FAST_SYNC_MAX_INFLIGHT`, `DEFAULT_DB_WRITE_BATCH_BLOCKS`, `DEFAULT_LOG_TRACE_FILTER`, `DEFAULT_LOG_JSON_FILTER`
- **Interactions**: `compute_target_range()` is used by `main.rs` to bound historical ingest ranges based on `head_at_startup` and `rollback_window`.
- **Knobs / invariants**: `compute_target_range()` clamps the requested end to `safe_head = head_at_startup - rollback_window` and returns an inclusive range.

## End-to-end flow (high level)
- Parse argv into `NodeConfig` via clap derive.
- Apply defaults for missing flags (RPC limits, fast-sync scheduling, log outputs).
- Derive the "safe" historical range via `compute_target_range()`.
- Pass config into storage opening (`data_dir`, `shard_size`), P2P startup, RPC server limits, and sync pipeline knobs.

## Special Modes

### `--repair` Flag
Runs storage repair/recovery without starting sync:
- Scans all shards for interrupted compactions (based on `compaction_phase` in `shard.json`)
- Recovers from interrupted write/swap/cleanup phases
- Cleans up orphan tmp/old files
- Prints a summary and exits

Example output:
```
Checking storage integrity...

Shard 24280000: recovered from interrupted swap phase (phase: swapping)
Shard 24290000: OK
Shard 24300000: cleaned orphan files

Repair complete. 2 shards repaired, 1 shards OK.
```
