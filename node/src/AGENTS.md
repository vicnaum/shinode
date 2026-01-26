# src

## Purpose
Crate source for the `stateless-history-node` binary. Provides the CLI/config model, P2P fetch
stack, storage backend, sync pipeline, and JSON-RPC server used by `main.rs`.

## Contents (one hop)
### Subdirectories
- [x] `chain/` - (skip: empty placeholder directory; no Rust sources)
- [x] `cli/` - CLI parsing + `NodeConfig` defaults and range helpers (`compute_target_range()`).
- [x] `logging/` - Logging, reporting, and resource monitoring (JSON logs, run reports, CPU/mem/disk metrics, SIGUSR1 handler).
- [x] `p2p/` - Mainnet devp2p networking and batched header/body/receipt fetching utilities.
- [x] `rpc/` - jsonrpsee server implementing a minimal Ethereum JSON-RPC surface.
- [x] `run/` - Run orchestration module; startup, sync execution, follow mode tracking, and cleanup/finalization.
- [x] `storage/` - Stored types and sharded on-disk backend (segments + WAL + bitset) for history data.
- [x] `sync/` - Sync primitives (status/progress) and historical backfill/follow pipeline.
- [x] `ui/` - Terminal UI module for progress bars, status display, and db stats output.

### Files
- `main.rs` - Minimal binary entrypoint; CLI dispatch to `run::run_sync()` or subcommand handlers.
  - **Key items**: `main()`, `ProgressBar` impl for `ProgressReporter`
- `metrics.rs` - Lightweight metrics helpers used for progress and summaries.
  - **Key items**: `range_len()`, `rate_per_sec()`, `percentile_triplet()`, `percentile()`
- `test_utils.rs` - Shared test utilities for integration tests.
  - **Key items**: `temp_dir()`, `base_config()`

## Key APIs (no snippets)
- **Modules**: `cli`, `logging`, `p2p`, `rpc`, `run`, `storage`, `sync`, `ui` - primary subsystems wired together by `run::run_sync()`.

## Relationships
- **Used by**: `node/src/main.rs` is the binary root; it dispatches to `run::run_sync()` for sync operations.
- **Data/control flow**:
  - `main.rs` parses CLI and delegates to `run::run_sync()`, `run::handle_db_stats()`, or `run::handle_repair()`.
  - `run_sync()` orchestrates: CLI config -> storage open -> P2P connect -> fast-sync ingest.
  - Head tracker persists `head_seen` and feeds tail scheduling to extend the safe-head target.
  - After fast-sync, switch to follow mode; follow reuses ingest + in-order DB writes. RPC starts only after the first "synced" edge (UpToDate/Following) so clients don't hit an empty DB.
  - RPC reads from `storage` for headers/receipts/logs.
