# node

## Purpose
Rust crate for the `stateless-history-node` binary. Connects to Ethereum mainnet via reth devp2p,
backfills and follows block history into an on-disk sharded store, and serves a minimal JSON-RPC
surface. Also includes benchmark/probe modes that emit trace/event/log artifacts.

## Contents (one hop)
### Subdirectories
- [x] `benchmarks/` - Benchmark/profiling artifacts emitted by the binary (trace JSON, per-run data snapshots).
- [x] `src/` - Implementation: CLI/config, P2P networking, storage, sync pipeline, RPC server, main entrypoint.
- [x] `target/` - (skip: generated Rust build artifacts)
- [x] `tests/` - Integration test scaffolding and TODO probes.

### Files
- `Cargo.toml` - Crate manifest defining features and dependencies (reth git pins, jsonrpsee, tokio).
  - **Key items**: `jemalloc`, `reth-network`, `reth-nippy-jar`, `jsonrpsee`, `tokio`
- `Cargo.lock` - Locked dependency versions for reproducible builds. (generated)
  - **Key items**: n/a

## Key APIs (no snippets)
- **Binaries**: `src/main.rs` - `main()` orchestrates storage + P2P + sync + RPC.
- **Modules**: `src/storage::Storage`, `src/rpc::start()`, `src/sync::historical::run_ingest_pipeline()` - primary entrypoints used by `main()`.

## Relationships
- **Depends on**: `reth-*` crates for devp2p + NippyJar primitives; `jsonrpsee` for JSON-RPC; `tokio` for async.
- **Data/control flow**:
  - Parse `cli::NodeConfig` -> open `storage::Storage`.
  - Start `p2p` network -> build peer pool.
  - Run `sync::historical` pipeline to fetch missing blocks, process bundles, and write to sharded storage.
  - If `--end-block` is set: sync the fixed range, finalize, and exit (no follow).
  - If `--end-block` is not set (default ingest): fast-sync to a moving safe head (tail ranges appended), then switch to follow mode.
  - Follow mode reuses the ingest pipeline with in-order DB writes (reorder buffer in `db_writer`) and reorg rollback.
  - Start `rpc` after follow first reports `SyncStatus::UpToDate` (read-only views over stored headers/receipts/logs).
  - Persist peer cache and (in benchmark modes) write summaries + traces/events/logs under `benchmarks/`.

## Notes
- `target/` and `benchmarks/` are expected to be large/generated and are not intended for manual review.
