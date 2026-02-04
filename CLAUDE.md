# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stateless History Node is a minimal Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset. It does not execute transactions or keep state (no EVM, no state trie).

## Build Commands

```bash
# Development build
cargo build
# Release build (production)
cargo build --manifest-path node/Cargo.toml --release

# Run (debug)
cargo run
# Run (release)
cargo run --release
# Run tests
cargo test
# Run a single test
cargo test --manifest-path node/Cargo.toml <test_name>

# Build without jemalloc (default feature)
cargo build --manifest-path node/Cargo.toml --no-default-features

```

## Architecture

The node is organized into subsystems in `node/src/`:

### P2P (`p2p/mod.rs`)
- Uses Reth's devp2p networking (`reth-network`, `reth-eth-wire`)
- `PeerPool` manages active peer sessions with head tracking
- `P2pStats` tracks discovery, sessions, and genesis mismatches (atomics)
- Handles eth/68, eth/69, and eth/70 protocol variants for receipts
- Peer cache persisted to `peers.json` for faster reconnection
- `re_probe_peer_head()` for confirming peer heads

### Sync (`sync/`)
- `sync/mod.rs`: Core types (`BlockPayload`, `SyncStatus`, `SyncProgressStats`, `CoverageTracker`, `FinalizePhase`)
- `sync/historical/`: Fast-sync implementation
  - `scheduler.rs`: Work queue management, peer assignment, AIMD batch sizing, quality scoring
  - `fetch_task.rs`: Per-batch fetch execution with `ActiveFetchTaskGuard`
  - `fetch.rs`: Batch fetching from peers
  - `process.rs`: Block processing pipeline with zero-copy KeccakBuf
  - `db_writer.rs`: WAL-based batched writes with per-shard compaction triggering
  - `follow.rs`: Head-following mode after catching up
  - `reorg.rs`: Rollback handling within configured window
  - `stats.rs`: Bench event logging and performance aggregation

### Storage (`storage/`)
- `storage/mod.rs`: Types for stored data (headers, tx hashes, receipts, logs)
- `storage/sharded/`: Sharded static-file backend (Storage v2)
  - `mod.rs`: Main `Storage` struct with read/write operations, aggregate stats, repair, sealed cache
  - `wal.rs`: Write-ahead log for out-of-order ingestion
  - `bitset.rs`: Per-shard presence tracking
  - `nippy_raw.rs`: Low-level segment file handling
  - `hash.rs`: Content hash for sealed shards
- Storage layout: `data_dir/static/shards/<shard_start>/`
- Sealed shard cache: `data_dir/sealed_shards.cache` (speeds up startup on slow storage)
- Segments: `headers`, `tx_hashes`, `tx_meta`, `receipts`, `block_sizes`
- `ShardMeta` tracks: `total_transactions`, `total_receipts`, `total_logs`, `disk_bytes_*`
- LRU segment reader cache (read-only, no file locks)

### RPC (`rpc/mod.rs`)
- JSON-RPC server using `jsonrpsee`
- Implements: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`
- Logs are derived on-demand from stored receipts
- Tracks request counters via `SyncProgressStats` for TUI display

### UI (`ui/`)
- Two-mode architecture: ratatui TUI dashboard (default) or indicatif progress bars (`--no-tui`)
- `ui/tui.rs`: Fullscreen dashboard with phase-aware layouts, speed chart, coverage map, panels
- `ui/progress.rs`: `UIController` for legacy progress bars, `spawn_tui_progress_updater()` for TUI
- `ui/mod.rs`: TUI mode detection, status bars, db stats output

### Logging (`logging/`)
- `json.rs`: Async JSON log writer with dedup, plus TUI log capture (`TuiLogBuffer`, `TuiLogLayer`)
- `report.rs`: Run reports for benchmarking
- `resources.rs`: CPU/memory/disk metrics (Linux proc + macOS sysinfo)
- SIGUSR1 handler for on-demand state dumps

## Key Dependencies

- **Reth crates**: `reth-network`, `reth-eth-wire`, `reth-ethereum-primitives`, `reth-primitives-traits` (pinned to specific git revision)
- **Serialization**: `bincode` for storage, `serde_json` for config/RPC
- **Compression**: `zstd` for segment compression
- **TUI**: `ratatui` + `crossterm` for terminal dashboard

## CLI Defaults

- `--start-block`: 10,000,000 (right before Uniswap V2 deployment)
- `--shard-size`: 10,000 blocks per shard
- `--rollback-window`: 64 blocks
- `--rpc-bind`: 127.0.0.1:8545
- `--defer-compaction`: Skip inline shard compaction during fast-sync (compact at finalize or via `db compact`)
- `--no-tui`: Disable fullscreen TUI (use legacy progress bars)
- `-v`/`-vv`/`-vvv`: Increasing verbosity (default is errors only)
- `db compact`: Standalone subcommand to compact dirty shards and seal completed ones
- `db stats`: Print storage statistics
- `db rebuild-cache`: Rebuild sealed shard cache for faster startup

## Log Artifacts

```bash
# Run with Chrome trace for timeline inspection
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 10_000_000 --end-block 10_010_000 \
  --log-trace

# Run with event logging for post-analysis
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 10_000_000 --end-block 10_010_000 \
  --log-events

# All log artifacts (convenience flag)
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 10_000_000 --end-block 10_010_000 \
  --log --run-name my_run

# Equivalent explicit form:
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 10_000_000 --end-block 10_010_000 \
  --log-trace --log-events --log-json --log-report --log-resources --run-name my_run
```

## Debugging

Send `SIGUSR1` to print sync + peer-health debug dump: `kill -USR1 <pid>`
