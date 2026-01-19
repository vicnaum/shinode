# Stateless History Node

A minimal, stateless Ethereum history indexer that backfills headers, receipts,
and logs from the EL P2P network and serves a small, indexer-friendly RPC
subset. It does not execute transactions or keep state.

## Status (v0.1.4)

What works:
- P2P range backfill from `start_block..head` (single run).
- MDBX persistence of headers, tx hashes, receipts, logs, log indexes, tx metadata
  (no calldata), withdrawals, and block size.
- RPC subset: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`,
  `eth_getLogs`, with request limits.
- Graceful shutdown, restart-safe checkpoints, and basic ingest stats.
- Verbosity flags and a simple progress bar (TTY only).

What does not yet:
- Follow mode (continuous syncing) and live reorg handling.
- Additional RPC methods (`eth_getBlockByHash`, receipts endpoints, WS).
- Any stateful calls (`eth_call`) or traces.
- Metrics export (Prometheus/OTel) or peer scoring/backoff beyond rotation.

## Philosophy

- **Stateless by design**: no EVM execution or state trie.
- **Small, auditable surface**: minimal persistence and RPC for indexers.
- **Reuse Reth primitives**: networking, MDBX, and safety defaults.

## Installation

Requires Rust toolchain and git access for Reth dependencies.

```bash
cargo build --manifest-path node/Cargo.toml
```

## Usage

```bash
cargo run --manifest-path node/Cargo.toml -- \
  --data-dir data \
  --rpc-bind 127.0.0.1:8545 \
  --start-block 19000000 \
  -v
```

Stop with Ctrl+C. The node persists checkpoints and resumes on restart.

## CLI options

Core:
- `--chain-id <u64>`: chain id for RPC (default: 1).
- `--data-dir <path>`: base data directory (default: `data`).
- `--rpc-bind <ip:port>`: RPC bind (default: `127.0.0.1:8545`).
- `--start-block <u64>`: first block to backfill (default: 0).
- `--rollback-window <u64>`: max rollback depth (default: 64).
- `--retention-mode <full>`: retention policy (default: `full`).
- `--head-source <p2p>`: head source (default: `p2p`).
- `--reorg-strategy <delete>`: rollback strategy (default: `delete`).
- `-v`: info-level node activity (RPC requests, ingest progress).
- `-vv`: debug for node internals.
- `-vvv`: trace for node internals.
- Default without `-v` is errors only. `RUST_LOG` overrides all defaults.

RPC safety limits:
- `--rpc-max-request-body-bytes <u32>` (default: 5_242_880).
- `--rpc-max-response-body-bytes <u32>` (default: 5_242_880).
- `--rpc-max-connections <u32>` (default: 100).
- `--rpc-max-batch-requests <u32>` (default: 10).
- `--rpc-max-blocks-per-filter <u64>` (default: 1000).
- `--rpc-max-logs-per-response <u64>` (default: 10000).

## Configuration and storage

The MDBX database lives under `data_dir/db`. A serialized config is persisted
on first run and validated on startup. If you change storage-affecting settings
(retention, head source, reorg strategy), use a new `data_dir`. Runtime-only
settings (verbosity, RPC limits) can be changed freely.

## RPC support

Implemented:
- `eth_chainId`
- `eth_blockNumber`
- `eth_getBlockByNumber` (with `includeTransactions=false`, full block shape)
- `eth_getLogs` (filtered by block range, address, and topic0)

Notes:
- `totalDifficulty` is currently mocked to `0x0`.

All other methods are unimplemented and return `-32601`.

## Project layout

- `node/`: stateless history node implementation.
- `harness/`: receipt availability harness (see `harness/README.md`).
- `spec/`, `SPEC.md`, `ROADMAP.md`: scope, non-goals, and progress.
