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
  - Store **headers + tx hashes + tx metadata (no calldata) + full receipts** for retained ranges
  - Logs are derived at query time; withdrawals are not stored
  - Defer “filtered logs only” / “calldata retention” to later versions
- [x] **Canonicality & head source contract** (explicit trust model):
  - MVP default: follow a best-effort head from the P2P view and handle reorgs within a rollback window
  - Later option: integrate a local CL or beacon API for finalized/safe head
- [x] **Reorg semantics**: rollback window, rollback strategy (v0.1 default: delete-on-rollback), and define what `eth_blockNumber` means (e.g., last fully indexed block)
- [x] **Storage choice (early)**: pick a backend that won’t block future work
  - **Static-file storage (NippyJar)** with rollback by tail pruning
- [x] **Crate/module boundaries** (match reth’s separation of concerns): `p2p`, `sync/ingest`, `chain`, `storage`, `rpc`, `cli/config`
- [x] **Test strategy**: unit tests for parsing/mapping + integration tests for reorg rollback + RPC conformance fixtures
- Verified: `cargo test --manifest-path node/Cargo.toml`; `curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}' http://127.0.0.1:8545` → `0x1`
- Verified: `cargo test --manifest-path node/Cargo.toml` (test strategy scaffold + storage metadata + RPC error semantics tests)

### v0.1.1 Sync + ingest loop (backfill → follow)
- [x] Replace anchor-window probing with **range sync** (`start_block..head`)
- Verified: `cargo test --manifest-path node/Cargo.toml` (range sync planner)
- Verified: `cargo test --manifest-path node/Cargo.toml` (checkpoint metadata scaffold)
- Verified: `cargo test --manifest-path node/Cargo.toml` (head tracker + sync controller)
- Verified: `cargo test --manifest-path node/Cargo.toml` (runtime planning log wiring)
- Verified: `cargo test --manifest-path node/Cargo.toml` (canonical chain tracker + reorg detection)
- Verified: `cargo test --manifest-path node/Cargo.toml` (sync runner with checkpoints + reorg rollback)
- [x] Maintain a **canonical header chain** (parent links) and detect reorgs
- [x] Fetch **block bodies** (or at least tx hashes) so logs/receipts can be associated with tx hashes correctly
- [x] Fetch **receipts** (eth/69/70) and derive logs with full metadata
- [x] **Real P2P path**: connect to mainnet peer and fetch headers/bodies/receipts for a range
- [x] **Checkpoint + resume**: idempotent writes, restart safety, and “last indexed block” tracking
- [x] Concurrency and peer selection informed by reth patterns (see `spec/reth_kb` Q015/Q016/Q034)
- [x] **Multi-peer selection + retries/backoff**: rotate peers on failure, limit timeouts
- [x] **Continuous peer discovery**: keep listening for sessions and update peer pool
- Verified: `cargo test --manifest-path node/Cargo.toml` (ingest runner log derivation + concurrency/peer scaffolds)
- Verified: `cargo run --manifest-path node/Cargo.toml -- --start-block 20000000 --rpc-bind 127.0.0.1:0 --data-dir data-p2p-test` (mainnet peer headers/bodies/receipts + ingest range)
- Verified: `cargo run --manifest-path node/Cargo.toml -- --start-block 20000000 --rpc-bind 127.0.0.1:0 --data-dir data-p2p-test2` (multi-peer code path; ingest range)

### v0.1.2 Persistence (queryable, restart-safe)
- [x] Define static-file segments for v0.1 retention (headers, tx hashes, receipts, tx metadata, block sizes)
- [x] Storage refactor: replace MDBX with NippyJar static files
- [x] Write path: append headers + tx hashes + receipts + tx metadata during ingest
- [x] Read path: fetch stored blocks/receipts by number/range (for RPC)
- [ ] Indexes for fast `eth_getLogs` (address/topic0) — deferred to post-sync indexing
- [x] Reorg rollback: tail-prune static segments past common ancestor
- Verified: `cargo test --manifest-path node/Cargo.toml` (storage writes + reads)
- Verified: `cargo test --manifest-path node/Cargo.toml` (range reads for headers/receipts)

### v0.1.3 JSON-RPC server (indexer-compatible subset)
- [x] Target v0.1 indexer: **rindexer** (polling-based; no `eth_subscribe` required)
- [x] **Minimum RPC for rindexer event indexing**:
  - [x] `eth_chainId`
  - [x] `eth_blockNumber`
  - [x] `eth_getBlockByNumber` (must support `"latest"`; must include `timestamp`, `number`, `hash`, `logsBloom`)
  - [x] `eth_getLogs` (must include `blockHash`, `blockNumber`, `transactionHash`, `transactionIndex`, `logIndex`)
- Verified: `cargo test --manifest-path node/Cargo.toml` (RPC block number + block + logs)
- [ ] **Ponder compatibility (later; NOT in v0.1) requires more than rindexer**:
  - `eth_getBlockByHash` (reorg traversal via `parentHash`)
  - `eth_call` (multicall3 + factory/read-contract flows; requires state, so not supported by a fully-stateless node unless proxied or executed via a stateless execution approach like RESS)
  - optional WS: `eth_subscribe` (`newHeads`) with polling fallback
  - optional receipts: `eth_getBlockReceipts` / `eth_getTransactionReceipt`
  - optional call traces: `debug_traceBlockByNumber` / `debug_traceBlockByHash`
- [x] Adopt reth-style query limits (`max_blocks_per_filter`, `max_logs_per_response`) and return a clear error when exceeded
- [x] Security defaults: bind **localhost by default**, configurable host/port, request/response limits, basic rate limiting
- Verified: `cargo test --manifest-path node/Cargo.toml` (RPC limits + server config)

### v0.1.4 Operator experience (minimum to run unattended)
- [x] CLI/config: chain, start block, retention, DB path, RPC bind, resource limits
- [x] Graceful shutdown + safe flushing
- [x] Minimal structured logs + counters (throughput, lag to head, reorg count, peer health)
- [x] Verbosity levels (-v/-vv/-vvv) + progress UI (harness-style)
- [x] Ingest benchmark mode with per-stage timing (fetch/process/static write)
- [x] DB stats CLI for static-file storage sizes
- [x] Defer sender recovery by storing signature + signing hash
- Verified: `cargo test --manifest-path node/Cargo.toml` (graceful shutdown behavior)
- Verified: `cargo test --manifest-path node/Cargo.toml` (ingest metrics logging helpers)
- Verified: `cargo test --manifest-path node/Cargo.toml` (RPC limit CLI config)
- Verified: `cargo test --manifest-path node/Cargo.toml` (verbosity + progress UI)

### v0.1.5 Historical sync speedup
- [ ] Define **historical head** = `head - rollback_window`
- [ ] Chunked range planner for historical blocks (default 32)
- [ ] Concurrent chunk fetch with bounded in-flight requests
- [ ] Buffer out-of-order chunks and **write in block order**
- [ ] Atomic per-chunk static-file appends + checkpoint updates
- [ ] Safe boundary switch to slow path near the reorg window
- [ ] Progress bar: harness-style status line (peers/queue/inflight/speed/eta)
- [ ] Post-sync log index build (address/topic0) to restore fast `eth_getLogs`

### v0.1.6 Live sync + reorg resilience
- [ ] **Follow mode**: loop until head, then keep up with new heads
- [ ] **Live reorg handling**: detect reorgs while following and roll back checkpoints

### Deferred RPC extras (post v0.1)
- [ ] `net_version`, `web3_clientVersion`, tx/receipt endpoints, WS subscriptions

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
- [ ] Peer scoring/backoff beyond simple rotation (timeouts, slow peers, disconnect reasons)
- [ ] Backpressure + memory caps for queues
- [ ] Tests: sync loop + reorg handling + retry/escalation
- [ ] Metrics export (Prometheus/OTel)
- [ ] Optional correctness hardening:
  - receipts root validation (header `receiptsRoot`)
  - multi-peer cross-check for headers/receipts
  - stronger head source (beacon API / CL integration) if P2P-only head proves flaky
  - tombstone / “removed logs” handling for reorgs (eth_getLogs consistency)

---

## 🚀 Later (Optimization): Performance & storage efficiency (v0.3+)
- [ ] Bloom-based short-circuiting (use stored `logsBloom` to skip blocks fast when scanning)
- [ ] Stronger log indexing (topic1-3, composite indexes, partitioning)
- [ ] Compression tuning (zstd) and additional cold storage formats
- [ ] Pipeline improvements: overlap headers/receipts/bodies fetching
- [ ] Optional calldata retention (config/CLI) + full transaction objects

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
