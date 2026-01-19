# Roadmap

## ✅ Done: Harness (Baseline P2P Fetch Engine)
- [x] Scaffold harness using Reth as a library (no execution/state)
- [x] P2P discovery, dialing, and `eth` handshake
- [x] Header + receipts probing with legacy (eth/68 and below), eth/69, and eth/70 handling
- [x] Batch requests and shared work queue scheduling
- [x] DeFi anchor block targeting with adjustable window
- [x] Known-blocks cache to avoid re-fetching
- [x] Soft ban for peers with repeated failures
- [x] Request/probe/stats JSONL output
- [x] Windowed stats and end-of-run summary
- [x] Real-time progress bar with status and ETA
- [x] Auto-exit when all work is complete
- [x] Two-tier retry strategy (normal retries + escalation pass)

---

## 🎯 v0.1 MVP (Functional): “Synced History RPC for Indexing”

Goal: ship a long-running service that **backfills**, **stays in sync with the head**, **persists history**, and exposes an **indexer-compatible JSON-RPC subset** (v0.1 target: **rindexer**) so you can index Uniswap without paid RPC.

This MVP is intentionally **stateless**: no EVM execution, no state trie, no archive-state growth. It ingests EL history artifacts (headers + bodies/tx hashes + receipts/logs) and serves them back.

### v0.1.0 Product contract + architecture skeleton (no redesign later)
- [x] **Retention (v0.1 simple default)**:
  - Store **headers + tx hashes + full receipts/logs** for retained ranges (no log filtering in v0.1)
  - Defer “filtered logs only” / “tx metadata” / “calldata retention” to later versions
- [x] **Canonicality & head source contract** (explicit trust model):
  - MVP default: follow a best-effort head from the P2P view and handle reorgs within a rollback window
  - Later option: integrate a local CL or beacon API for finalized/safe head
- [x] **Reorg semantics**: rollback window, rollback strategy (v0.1 default: delete-on-rollback), and define what `eth_blockNumber` means (e.g., last fully indexed block)
- [x] **Storage choice (early)**: pick a backend that won’t block future work
  - Recommended baseline: **MDBX** (reth-style), with a schema designed for later “static files” / cold storage
- [x] **Crate/module boundaries** (match reth’s separation of concerns): `p2p`, `sync/ingest`, `chain`, `storage`, `rpc`, `cli/config`
- [x] **Test strategy**: unit tests for parsing/mapping + integration tests for reorg rollback + RPC conformance fixtures
- Verified: `cargo test --manifest-path node/Cargo.toml`; `curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}' http://127.0.0.1:8545` → `0x1`
- Verified: `cargo test --manifest-path node/Cargo.toml` (test strategy scaffold + storage metadata unit test)

### v0.1.1 Sync + ingest loop (backfill → follow)
- [ ] Replace anchor-window probing with **range sync** (`start_block..head`)
- [ ] Maintain a **canonical header chain** (parent links) and detect reorgs
- [ ] Fetch **block bodies** (or at least tx hashes) so logs/receipts can be associated with tx hashes correctly
- [ ] Fetch **receipts**, reconstruct missing blooms (eth/69/70), and derive logs with full metadata
- [ ] **Checkpoint + resume**: idempotent writes, restart safety, and “last indexed block” tracking
- [ ] Concurrency and peer selection informed by reth patterns (see `spec/reth_kb` Q015/Q016/Q034)

### v0.1.2 Persistence (queryable, restart-safe)
- [ ] Schema that supports MVP retention *and* future expansion:
  - blocks: number, hash, parent, timestamp, `logsBloom`
  - txs: per-block tx hashes
  - receipts/logs: enough to answer `eth_getLogs` (and optionally `eth_getTransactionReceipt` later)
- [ ] Indexes for fast `eth_getLogs` (at least by block range + address/topic0)
- [ ] Reorg rollback: delete data past common ancestor (tombstones/“removed logs” support deferred)

### v0.1.3 JSON-RPC server (indexer-compatible subset)
- [ ] Target v0.1 indexer: **rindexer** (polling-based; no `eth_subscribe` required)
- [ ] **Minimum RPC for rindexer event indexing**:
  - `eth_chainId`
  - `eth_blockNumber`
  - `eth_getBlockByNumber` (must support `"latest"`; must include `timestamp`, `number`, `hash`, `logsBloom`)
  - `eth_getLogs` (must include `blockHash`, `blockNumber`, `transactionHash`, `transactionIndex`, `logIndex`)
- [ ] **Ponder compatibility (later; NOT in v0.1) requires more than rindexer**:
  - `eth_getBlockByHash` (reorg traversal via `parentHash`)
  - `eth_call` (multicall3 + factory/read-contract flows; requires state, so not supported by a fully-stateless node unless proxied or executed via a stateless execution approach like RESS)
  - optional WS: `eth_subscribe` (`newHeads`) with polling fallback
  - optional receipts: `eth_getBlockReceipts` / `eth_getTransactionReceipt`
  - optional call traces: `debug_traceBlockByNumber` / `debug_traceBlockByHash`
- [ ] Adopt reth-style query limits (`max_blocks_per_filter`, `max_logs_per_response`) and return a clear error when exceeded
- [ ] Security defaults: bind **localhost by default**, configurable host/port, request/response limits, basic rate limiting
- [ ] Deferred beyond v0.1: `net_version`, `web3_clientVersion`, tx/receipt endpoints, WS subscriptions

### v0.1.4 Operator experience (minimum to run unattended)
- [ ] CLI/config: chain, start block, retention, DB path, RPC bind, resource limits
- [ ] Graceful shutdown + safe flushing
- [ ] Minimal structured logs + counters (throughput, lag to head, reorg count, peer health)

### v0.1 release criteria (definition of “usable MVP”)
- [ ] Fresh start: backfills from configured start block to “head”
- [ ] Steady-state: continues indexing new blocks and survives reorgs within configured window
- [ ] Restart safe: resumes without corrupting data or duplicating logs
- [ ] Indexer can run against it using the supported RPC subset
  - Minimum: Uniswap indexing workload (logs + block timestamps)

---

## ✅ Next (Functional): Reliability for long-running operation (v0.2)
- [ ] Persist known good peers (warm start)
- [ ] **Adaptive concurrency control** (avoid throughput dropping when peers increase)
  - Observed: 10 peers → 1150 b/s, 20 peers → 800 b/s, 50 peers → 500 b/s
- [ ] Better peer scoring/backoff (timeouts, slow peers, disconnect reasons)
- [ ] Backpressure + memory caps for queues
- [ ] Tests: sync loop + reorg handling + retry/escalation
- [ ] Metrics export (Prometheus/OTel)
- [ ] Optional correctness hardening:
  - receipts root validation (header `receiptsRoot`)
  - multi-peer cross-check for headers/receipts
  - stronger head source (beacon API / CL integration) if P2P-only head proves flaky

---

## 🚀 Later (Optimization): Performance & storage efficiency (v0.3+)
- [ ] Bloom-based short-circuiting (use stored `logsBloom` to skip blocks fast when scanning)
- [ ] Stronger log indexing (topic1-3, composite indexes, partitioning)
- [ ] Compression tuning (zstd) and cold storage formats (e.g., reth-style static files / NippyJar segments)
- [ ] Pipeline improvements: overlap headers/receipts/bodies fetching

---

## 🔒 Later (Correctness / trust hardening)
- [ ] Optional receipts root validation (verify against header)
- [ ] Multi-peer cross-check (majority header hash / receiptsRoot)
- [ ] “Confidence levels” (unsafe vs safer head) if needed

---

## 🧰 Later (Fallbacks for pruned history)
- [ ] ETH: era1 / Portal for old ranges

---

## 🤝 Later (Network-serving)
- [ ] Serve `GetBlockHeaders` / `GetReceipts` for the retained ranges
- [ ] Rate limiting + abuse protection

---

## ❌ Out of scope
- Full execution / state / traces
- **OP Stack L2s (Base, Optimism):** These chains use libp2p (not devp2p) with a
  different protocol (`payload_by_number`). P2P does not serve receipts directly;
  blocks must be executed to derive them. A separate harness would be needed.
