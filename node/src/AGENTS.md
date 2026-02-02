# src

## Purpose

Main source tree for the Stateless History Node. Contains all application code organized into subsystems: P2P networking, sync pipeline, sharded storage, JSON-RPC server, terminal UI, logging/observability, CLI configuration, and runtime orchestration.

## Contents (one hop)

### Subdirectories
- [x] `bin/` - Demo/testing binaries for TUI development (ui_mock, color_test, char_test). Standalone executables with no parent crate imports.
- [x] `cli/` - CLI argument parsing and central `NodeConfig` struct (40+ parameters). Single source of truth for all runtime configuration. Includes range helpers (`compute_target_range()`).
- [x] `logging/` - Observability subsystem: async JSON logging with dedup, TUI log capture (`TuiLogBuffer`, `TuiLogLayer`), run reports, platform-specific resource monitoring (Linux proc + macOS sysinfo), SIGUSR1 handler for on-demand state dumps.
- [x] `p2p/` - Ethereum P2P networking via Reth devp2p. `PeerPool` manages active sessions with head tracking and `P2pStats` (atomics). Multi-protocol fetch APIs (eth/68-70), persistent peer cache (`peers.json`), `re_probe_peer_head()` for confirming peer heads.
- [x] `rpc/` - Minimal JSON-RPC server (eth_chainId, eth_blockNumber, eth_getBlockByNumber, eth_getLogs). Derives logs on-demand from stored receipts. Tracks request counters via `SyncProgressStats` for TUI display.
- [x] `run/` - Top-level runtime orchestration. Coordinates startup with TUI splash, sync execution, follow mode tracking, RPC launch, cleanup/finalization, and utility subcommands (`db stats`, `db compact`) plus the `--repair` flag.
- [x] `storage/` - Persistence layer with serializable types and sharded static-file backend. WAL-based writes, atomic compaction with per-shard triggers, LRU-cached segment reads (no file locks).
- [x] `sync/` - Sync state machine, progress tracking (`CoverageTracker`, `FinalizePhase`), and historical sync pipeline. Fast-sync with AIMD batch sizing and quality scoring, follow-mode with reorg detection within configurable rollback window.
- [x] `ui/` - Dual-mode terminal interface: ratatui fullscreen TUI dashboard (default, 10 FPS) with phase-aware layouts, speed chart, coverage map, and panels; or indicatif progress bars (`--no-tui`).

### Files
- `main.rs` - Application entry point. Parses `NodeConfig`, routes to subcommands (`db stats`, `db compact`, `repair`) or `run::run_sync()`. Configures jemalloc allocator (feature-gated). Implements `ProgressReporter` for indicatif `ProgressBar`.
  - **Key items**: `main()`, `ProgressReporter for ProgressBar`, jemalloc feature gate
- `metrics.rs` - Lightweight utility functions for metrics calculations.
  - **Key items**: `range_len()`, `rate_per_sec()`, `percentile_triplet()`, `percentile()`
- `test_utils.rs` - Test-only utilities (compiled under `#[cfg(test)]`).
  - **Key items**: `temp_dir(prefix)` (unique temp dirs), `base_config(data_dir)` (default `NodeConfig` for tests)

## Key APIs (no snippets)

- **Entry point**: `main()` in `main.rs`
- **Configuration**: `cli::NodeConfig` (central config struct)
- **Runtime**: `run::run_sync()`, `run::handle_db_stats()`, `run::handle_db_compact()`, `run::handle_repair()`
- **Storage**: `storage::Storage` (sharded backend)
- **Sync**: `sync::SyncProgressStats`, `sync::historical::run_ingest_pipeline()`, `sync::historical::run_follow_loop()`
- **P2P**: `p2p::connect_mainnet_peers()`, `p2p::fetch_payloads_for_peer()`
- **RPC**: `rpc::start()`
- **Logging**: `logging::init_tracing()`
- **UI**: `ui::TuiController`, `ui::UIController`

## Relationships

- **Depends on**: Reth crates (`reth-network`, `reth-eth-wire`, `reth-nippy-jar`, `reth-ethereum-primitives`), `tokio`, `jsonrpsee`, `ratatui`, `crossterm`, `indicatif`, `tracing`, `clap`, `serde`, `bincode`, `zstd`
- **Used by**: Binary targets (`main.rs`, `bin/*.rs`)
- **Data/control flow**:
  1. `main.rs` parses CLI -> routes to `run::run_sync()` or subcommand handlers (`db stats`, `db compact`), or runs storage-only recovery via `--repair`
  2. `run/` orchestrates: init tracing -> early TUI splash -> open storage -> connect P2P -> compute target range -> run ingest pipeline
  3. `sync/historical/` fetches blocks from P2P peers -> processes (zero-copy Keccak hashing) -> writes to storage WAL
  4. Head tracker persists `head_seen` and feeds tail scheduling to extend the safe-head target
  5. `storage/sharded/` compacts WAL to sorted segments, provides LRU-cached reads
  6. After fast-sync, switches to follow mode; follow reuses ingest + in-order DB writes
  7. `rpc/` starts only after the first "synced" edge (UpToDate/Following) so clients don't hit an empty DB; serves indexed data from storage
  8. `ui/` displays real-time progress from `SyncProgressStats` snapshots (sync progress, coverage map, speed chart, peers, storage, DB, RPC)
  9. `logging/` captures events to JSON files, Chrome traces, and resource logs
