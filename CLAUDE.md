# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stateless History Node is a minimal Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset. It does not execute transactions or keep state (no EVM, no state trie).

## Build Commands

```bash
# Development build
cargo build --manifest-path node/Cargo.toml

# Release build (production)
cargo build --manifest-path node/Cargo.toml --release

# Run (debug)
cargo run --manifest-path node/Cargo.toml

# Run (release)
cargo run --release --manifest-path node/Cargo.toml

# Run tests
cargo test --manifest-path node/Cargo.toml

# Run a single test
cargo test --manifest-path node/Cargo.toml <test_name>

# Build without jemalloc (default feature)
cargo build --manifest-path node/Cargo.toml --no-default-features
```

## Architecture

The node is organized into four main subsystems in `node/src/`:

### P2P (`p2p/mod.rs`)
- Uses Reth's devp2p networking (`reth-network`, `reth-eth-wire`)
- `PeerPool` manages active peer sessions with head tracking
- `MultiPeerBlockPayloadSource` provides block data via P2P
- Handles eth/68, eth/69, and eth/70 protocol variants for receipts
- Peer cache persisted to `peers.json` for faster reconnection

### Sync (`sync/`)
- `sync/mod.rs`: Core types (`BlockPayload`, `SyncStatus`, `SyncProgressStats`)
- `sync/historical/`: Fast-sync implementation
  - `scheduler.rs`: Work queue management, peer assignment, AIMD batch sizing
  - `fetch.rs`: Batch fetching from peers
  - `process.rs`: Block processing pipeline
  - `db_writer.rs`: WAL-based batched writes
  - `follow.rs`: Head-following mode after catching up
  - `reorg.rs`: Rollback handling within configured window

### Storage (`storage/`)
- `storage/mod.rs`: Types for stored data (headers, tx hashes, receipts, logs)
- `storage/sharded/`: Sharded static-file backend (Storage v2)
  - `mod.rs`: Main `Storage` struct with read/write operations
  - `wal.rs`: Write-ahead log for out-of-order ingestion
  - `bitset.rs`: Per-shard presence tracking
  - `nippy_raw.rs`: Low-level segment file handling
- Storage layout: `data_dir/static/shards/<shard_start>/`
- Segments: `headers`, `tx_hashes`, `tx_meta`, `receipts`, `block_sizes`

### RPC (`rpc/mod.rs`)
- JSON-RPC server using `jsonrpsee`
- Implements: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`
- Logs are derived on-demand from stored receipts

## Key Dependencies

- **Reth crates**: `reth-network`, `reth-eth-wire`, `reth-ethereum-primitives`, `reth-primitives-traits` (pinned to specific git revision)
- **Serialization**: `bincode` for storage, `serde_json` for config/RPC
- **Compression**: `zstd` for segment compression

## CLI Defaults

- `--start-block`: 10,000,000 (right before Uniswap V2 deployment)
- `--shard-size`: 10,000 blocks per shard
- `--rollback-window`: 64 blocks
- `--rpc-bind`: 127.0.0.1:8545
- `-v`/`-vv`/`-vvv`: Increasing verbosity (default is errors only)

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
  --log-trace --log-events --log-json --log-report --run-name my_run
```

## Debugging

Send `SIGUSR1` to print sync + peer-health debug dump: `kill -USR1 <pid>`
