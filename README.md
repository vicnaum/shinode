# Stateless History Node

A minimal, stateless Ethereum history indexer that backfills headers, receipts,
and logs from the EL P2P network and serves a small, indexer-friendly RPC
subset. It does not execute transactions or keep state.

## Status (v0.2)

What works:
- P2P range backfill from `start_block..head` and continuous follow mode (keeps indexing new heads).
- Live reorg handling within `--rollback-window` (delete-on-rollback).
- Sharded static-file persistence (Storage v2): per-`--shard-size` shard directories with
  per-shard presence bitsets + WAL staging + compaction into canonical sorted segments.
- Logs are derived on-demand from receipts; withdrawals are not stored.
- RPC subset: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`,
  `eth_getLogs`, with request limits.
- Graceful shutdown, restart-safe checkpoints, and basic ingest stats (resume skips already-present blocks; finalize recompacts dirty shards).
- Peer health tracking (bans/quality scoring), bounded per-batch timeouts, and partial response handling.
- Verbosity flags and progress bars (TTY only).
- Benchmark warmup gating: `--benchmark-min-peers` (default: 5).
- Fast-sync WAL batching (out-of-order) + benchmark events for compaction/sealing timings.

What does not yet:
- Additional RPC methods (`eth_getBlockByHash`, receipts endpoints, WS).
- Any stateful calls (`eth_call`) or traces.
- Metrics export (Prometheus/OTel).

## Philosophy

- **Stateless by design**: no EVM execution or state trie.
- **Small, auditable surface**: minimal persistence and RPC for indexers.
- **Reuse Reth primitives**: networking, static-file storage, and safety defaults.

## Installation

Requires Rust toolchain and git access for Reth dependencies.

```bash
cargo build --manifest-path node/Cargo.toml
```

Production build:

```bash
cargo build --manifest-path node/Cargo.toml --release
```

## Usage

```bash
cargo run --manifest-path node/Cargo.toml
```

Stop with Ctrl+C. The node persists checkpoints and resumes on restart.

If you omit `--start-block`, it defaults to **10_000_000** (right before Uniswap-V2 ETH mainnet deployment).

## Debugging

On Unix, send `SIGUSR1` to print a sync + peer-health debug dump to logs:

```bash
kill -USR1 <pid>
```

## CLI options

Core:
- `--chain-id <u64>`: chain id for RPC (default: 1 as the only supported is ETH Mainnet).
- `--data-dir <path>`: base data directory (default: `data`).
- `--peer-cache-dir <path>`: directory for persisted peer cache (shared across runs).
- `--rpc-bind <ip:port>`: RPC bind (default: `127.0.0.1:8545`).
- `--start-block <u64>`: first block to backfill (default: 10_000_000 - right before Uniswap V2 was deployed on ETH Mainnet).
- `--shard-size <u64>`: blocks per shard for Storage v2 (default: 10_000).
- `--end-block <u64>`: optional final block to stop at.
- `--rollback-window <u64>`: max rollback depth (default: 64).
- `--retention-mode <full>`: retention policy (default: `full`).
- `--head-source <p2p>`: head source (default: `p2p`).
- `--reorg-strategy <delete>`: rollback strategy (default: `delete`).
- `-v`: info-level node activity (RPC requests, ingest progress).
- `-vv`: debug for node internals.
- `-vvv`: trace for node internals.
- Default without `-v` is errors only. `RUST_LOG` overrides all defaults.

Benchmark/probe:
- `--benchmark probe`: harness-like probe mode (headers + receipts only).
  - No DB writes, no chain tracking, no RPC server.
  - Exits after finishing the configured range.
  - If `--start-block` is above the peer-reported safe head, waits for more peers/head updates.
  - Range selection uses the same flags as normal sync:
    - `--start-block` sets the start (default 0).
    - `--end-block` limits the end.
    - If `--end-block` is not set, the end is the head at startup (with rollback window applied).
- `--benchmark ingest`: full ingest benchmark (fetch + process + DB writes).
  - No RPC server; exits after finishing the configured range.
  - If `--start-block` is above the peer-reported safe head, waits for more peers/head updates.
  - Prints a per-stage timing summary (fetch/process/db) as JSON.
  - Writes a summary JSON file to `--benchmark-output-dir` (default: `benchmarks`).
  - Optional artifacts:
    - `--benchmark-name <string>`: label used in output filenames.
    - `--benchmark-trace`: Chrome trace (`.trace.json`) for timeline inspection.
    - `--benchmark-events`: JSONL event log (`.events.jsonl`) for post-analysis (capped; default: 10,000,000 events).
    - `--benchmark-min-peers <u64>`: wait for at least N connected peers before starting the benchmark (default: 5).

RPC safety limits:
- `--rpc-max-request-body-bytes <u32>` (default: 10_485_760).
- `--rpc-max-response-body-bytes <u32>` (default: 104_857_600).
- `--rpc-max-connections <u32>` (default: 100).
- `--rpc-max-batch-requests <u32>` (default: 100; `0` = unlimited).
- `--rpc-max-blocks-per-filter <u64>` (default: 10000; `0` = unlimited).
- `--rpc-max-logs-per-response <u64>` (default: 100000; `0` = unlimited).

Ingest tuning:
- `--fast-sync-chunk-size <u64>`: initial blocks per peer batch (default: 16).
- `--fast-sync-chunk-max <u64>`: hard cap for per-peer AIMD batch size (defaults to `4x --fast-sync-chunk-size`).
- `--fast-sync-max-inflight <u32>`: max concurrent peer batches (default: 15).
- `--fast-sync-batch-timeout-ms <u64>`: per-batch timeout (default: 5000).
- `--fast-sync-max-buffered-blocks <u64>`: max buffered blocks (default: 2048).
- `--fast-sync-max-lookahead-blocks <u64>`: max blocks ahead of the DB writer low watermark to assign (default: 50_000; `0` = unlimited).
- `--db-write-batch-blocks <u64>`: batch size for fast-sync WAL writes in ingest mode (default: 512). Follow mode remains per-block.
- `--db-write-flush-interval-ms <u64>`: optional time-based flush interval.

DB stats:
- `stateless-history-node db stats --data-dir <path>`: print static-file storage sizes.
- `stateless-history-node db stats --data-dir <path> --json`: JSON output for tooling.

## Profiling

CPU sampling (Linux):

```bash
samply record -- \
  cargo run --manifest-path node/Cargo.toml --release -- \
  --benchmark ingest --start-block 10_000_000 --end-block 10_010_000
```

CPU sampling (macOS):

- Use Instruments → Time Profiler.
- Target the `stateless-history-node` process launched via:

```bash
cargo run --manifest-path node/Cargo.toml --release -- \
  --benchmark ingest --start-block 10_000_000 --end-block 10_010_000
```

Benchmark timeline artifacts:

- Add `--benchmark-trace` to emit a Chrome trace file alongside the summary JSON.
- See `PERFORMANCE.md` for saved benchmark run summaries + bottleneck notes.

Notes:

- Jemalloc is enabled by default (Cargo feature `jemalloc` is in `default`).
  - To build/run *without* jemalloc: add `--no-default-features` to your Cargo command.
  - `tracing_samply` and `tokio-console` integrations are planned but not wired yet.

## Allocator tuning (Linux)

Some benchmark workloads are allocation-heavy (WAL replay/compaction, decompression, etc.). On Linux/glibc you can reduce RSS fragmentation by limiting malloc arenas:

```bash
MALLOC_ARENA_MAX=2 cargo run --manifest-path node/Cargo.toml --release -- --benchmark ingest ...
```

Alternatively, build/run with jemalloc (compile-time):

```bash
cargo run --manifest-path node/Cargo.toml --release -- --benchmark ingest ...
```

## Configuration and storage

Storage is under `data_dir`:
- `meta.json`: schema version, chain id, storage key, shard size, checkpoints.
- `static/shards/<shard_start>/`:
  - `shard.json`, `present.bitset`
  - `state/staging.wal` (during fast-sync / out-of-order ingestion)
  - `sorted/` shard segments (`headers`, `tx_hashes`, `tx_meta`, `receipts`, `block_sizes`)

Peer cache (shared across runs):

- `peers.json`: cached peers (TTL + cap applied on load) stored under `--peer-cache-dir`.
- Default is `~/.stateless-history-node/peers.json` unless `--peer-cache-dir` is set.

The config is validated on startup. If you change storage-affecting settings
(retention, head source, reorg strategy, shard size), use a new `data_dir`.
Runtime-only settings (verbosity, RPC limits) can be changed freely.

## RPC support

Implemented:
- `eth_chainId`
- `eth_blockNumber`
- `eth_getBlockByNumber` (with `includeTransactions=false`, full block shape)
- `eth_getLogs` (filtered by block range, address, and topic0)

Notes:
- `totalDifficulty` is currently mocked to `0x0`.
- `withdrawals` are always `null` (not stored).

All other methods are unimplemented and return `-32601`.

## Project layout

- `node/`: stateless history node implementation.
- `harness/`: receipt availability harness (see `harness/README.md`).
- `SPEC.md`: current system spec (“what exists / doesn’t exist”).
- `PRD.md`: v0.1 product contract (RPC semantics + constraints).
- `ROADMAP.md`: milestones and what’s next.
- `spec/`: supporting docs (worklogs, reth knowledge base, research transcripts).
