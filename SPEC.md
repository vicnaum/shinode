# Stateless History Node SPEC

A **stateless history node** that ingests history artifacts from the Ethereum Execution Layer (EL) P2P network and serves an indexer-compatible RPC subset.

> **Note**: The v0.1 MVP PRD is archived at [spec/archive/PRD_v0.1.md](spec/archive/PRD_v0.1.md).
> For architecture overview, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Current status (v0.2.0)

### Core functionality
- Range backfill from `start_block..head` and continuous follow mode.
- Live reorg handling within `--rollback-window` (delete-on-rollback).
- Sharded static-file persistence (Storage v2) with per-shard WAL staging and compaction.
- Logs derived on-demand from receipts; withdrawals not stored.

### RPC
- `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`
- Configurable query limits and safety defaults (localhost bind).
- `totalDifficulty` mocked to `0x0`.

### Operator experience
- UI with colored stage indicators and compaction/sealing progress.
- Priority escalation queue for difficult blocks.
- `--repair` command for storage recovery.
- Resume without redownload (skip present blocks, recompact dirty shards).
- DB stats CLI for on-disk storage sizes.
- Graceful shutdown with checkpoint persistence.

### Reliability
- Peer warmup gating (`--min-peers`).
- Fast-sync WAL batching (out-of-order ingestion).
- Atomic compaction with crash recovery.
- Compaction memory hardening (streaming WAL, serialized compactions).
- Optional jemalloc allocator (default) or glibc arena tuning.
- Backpressure and memory caps for queues.

### Observability
- Verbosity flags (`-v`/`-vv`/`-vvv`) and progress bars (TTY only).
- Optional log artifacts: Chrome trace, JSONL events, JSON logs, run reports.
- `--log-resources` for CPU/memory/disk metrics.
- `SIGUSR1` debug dump for sync + peer health.

### Not yet implemented
- Additional RPC methods (`eth_getBlockByHash`, receipts endpoints, WS).
- Stateful calls (`eth_call`) or traces.
- Metrics export (Prometheus/OTel).
- Stronger head/finalization signals (safe/finalized).

## Goals
- **Ship a usable v0.1 MVP quickly**: backfill → follow head → persist → serve RPC.
- **Stay stateless**: no EVM execution, no state trie/database, no archive-state growth.
- **Future-proof architecture from day 1**: clear boundaries, stable storage contract, minimal rewrites.
- **Reuse Reth components and patterns** wherever feasible:
  - Networking (see `spec/reth_kb/agents-export/reth/crates/net/` for per-crate `AGENTS.md`)
  - Proven download/concurrency patterns (see `spec/reth_kb` Q034/Q035)
  - Storage primitives (static-file patterns) and safety knobs
  - RPC safety defaults and query limits (see `spec/reth_kb` Q021/Q022/Q023)

## Non-goals (for v0.1)
- Full execution/state, archive state, or EVM traces.
- Transaction pool or mempool APIs.
- “Full node” correctness guarantees (the trust model is intentionally pragmatic in early versions).

## MVP v0.1: “Synced History RPC for Indexing”

### What “end-to-end usable” means
- **Backfill** from a configured `start_block` to a moving “head”.
- **Follow** new blocks continuously.
- **Persist** data restart-safely and idempotently (no duplicates, no corruption).
- **Serve RPC** sufficient for the v0.1 target indexer (**rindexer**) to run end-to-end:
  - Required: `eth_chainId`
  - Required: `eth_blockNumber`
  - Required: `eth_getBlockByNumber` (including `"latest"`)
  - Required: `eth_getLogs`
  - Optional (v0.2+): additional endpoints for other indexers (see note below)
- **Handle reorgs** within a configured rollback window (v0.1 default: delete-on-rollback).

### Note: Ponder baseline requires stateful RPC
Ponder’s baseline “normal app” mode includes **`eth_call`** (multicall3 / read-contract / some factory flows) and uses **`eth_getBlockByHash`** for realtime reorg traversal. Since `eth_call` requires EVM state, a fully stateless history node cannot serve it without either:
- running execution/state (out of scope), or
- proxying `eth_call` to an upstream RPC (optional future integration), or
- a stateless execution approach (e.g., Reth’s RESS subsystem) if feasible for the required calls.

### Data we must ingest (minimum set)
To serve `eth_getLogs` correctly (including `transactionHash`/`transactionIndex`/`logIndex`), we need:
- **Headers** (canonical chain): number/hash/parent/timestamp + `logsBloom`
- **Tx hashes per block** (from block bodies or equivalent)
- **Receipts** per block (from `GetReceipts*`), then derive logs with full metadata

### Retention modes (persisted settings)
Retention is a product-contract decision and must be explicit up-front to avoid DB redesign:
- **v0.1 (simple default)**: store headers + tx hashes + tx metadata (no calldata) +
  full receipts; logs derived at query time; withdrawals not stored
- **Later (v0.2+)**:
  - optional “filtered logs only” retention
  - optional tx calldata retention (`input`)

### Canonicality and head source (trust model)
Reth itself relies on an external CL via Engine API for head/finalization (see `spec/reth_kb` Q039).
For our v0.1 stateless history node we can start with a pragmatic model:
- **MVP default**: best-effort head tracking from the P2P view + reorg rollback window
- **Later upgrade**: integrate a local CL (or beacon API) to get `safe`/`finalized` head signals and stronger canonicality

## Architecture outline (inspired by Reth)
High-level components (mirrors reth’s separation of concerns):
1. **P2P**: run devp2p networking using `reth-network`, subscribe to peer events, and issue `GetBlockHeaders` / `GetBlockBodies` / `GetReceipts*` requests.
2. **Sync/ingest orchestrator**: a coordinator loop that decides “what to fetch next” and manages concurrency and retries (informed by reth’s engine download coordination patterns, Q034).
3. **Chain tracker**: maintains the canonical header chain view, detects reorgs, and provides rollback points.
4. **Storage**: a durable store with:
  - schema + versioning
  - append-only static files with rollback via tail pruning
  - logs derived from receipts at query time
5. **RPC**: a minimal JSON-RPC server with safe defaults (localhost bind by default, query limits) and only the namespaces we support.

## Historical fast sync (v0.1.5)
For blocks **older than the reorg window**, we run a fast backfill mode that supports
**out-of-order ingestion** via Storage v2 shards:
- schedule missing blocks across the target range (prefiltered by shard presence bitsets)
- write out-of-order blocks to per-shard `state/staging.wal`
- compact “done for this run” shards into canonical `sorted/` segments

For blocks **inside the reorg window**, the follow loop keeps the DB consistent via rollback.

Storage format details (schema v2): `spec/worklogs/sharding_refactor/SHARDED_STORAGE_REFACTOR_SPEC.md`.

## References
- Reth knowledge base index: `spec/reth_kb/INDEX.md`
- Reth "what lives where" map: `spec/reth_kb/agents-export/reth/crates/**/AGENTS.md`
