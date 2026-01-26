# Roadmap

## Done: v0.1 MVP - "Synced History RPC for Indexing"

A long-running service that **backfills**, **stays in sync with the head**, **persists history**, and exposes an **indexer-compatible JSON-RPC subset** (target: **rindexer**) so you can index Uniswap without paid RPC.

Intentionally **stateless**: no EVM execution, no state trie, no archive-state growth. Ingests EL history artifacts (headers + bodies/tx hashes + receipts/logs) and serves them back.

### Completed milestones

| Version | Focus | Key deliverables |
|---------|-------|------------------|
| v0.1.0 | Product contract | CLI/config, storage schema, retention modes, test strategy |
| v0.1.1 | Sync + ingest | Range sync, canonical chain, reorg detection, multi-peer fetching |
| v0.1.2 | Persistence | NippyJar static files, idempotent writes, rollback |
| v0.1.3 | RPC server | `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs` |
| v0.1.4 | Operator UX | CLI config, graceful shutdown, verbosity, progress UI, log artifacts |
| v0.1.5 | Fast sync | Chunked concurrent downloads, AIMD batch sizing, bounded buffering |
| v0.1.6 | Live sync | Follow mode, live reorg handling within rollback window |
| v0.1.7 | Storage v2 | Sharded storage, per-shard WAL, out-of-order ingestion, shard sealing |

### Release criteria met
- Fresh start: backfills from configured start block to head
- Steady-state: continues indexing new blocks and survives reorgs
- Restart safe: resumes without corrupting data or duplicating logs
- Indexer compatible: rindexer runs against the supported RPC subset

---

## Done: v0.2.0 - Reliability for Long-Running Operation

Focus: stability and performance hardening for production use.

### Completed
- [x] Persist peer cache (`peers.json` with TTL + cap)
- [x] Peer warmup gate (`--min-peers`)
- [x] Peer pool warmup (async head probes)
- [x] Fast-sync WAL batch writes (out-of-order)
- [x] Log events instrumentation (`--log-events`)
- [x] Compaction memory hardening (streaming WAL, serialized compactions)
- [x] Optional jemalloc allocator (default feature)
- [x] Backpressure + memory caps for queues
- [x] Safe boundary switch near reorg window
- [x] Unified ingest pipeline with log artifacts
- [x] Resume without redownload (skip present blocks, recompact dirty shards)
- [x] Follow mode with tail scheduling
- [x] UI improvements (colored stages, compaction/sealing progress)
- [x] Priority escalation queue for difficult blocks
- [x] Atomic compaction with crash recovery
- [x] `--repair` command for storage recovery
- [x] `--log-resources` for CPU/memory/disk metrics
- [x] Modular codebase (run/, ui/, logging/ modules)

---

## Current: v0.2.x - Remaining Reliability Work

### In progress
- [ ] Adaptive concurrency control (throughput drops at high peer counts)
- [ ] Peer scoring/backoff beyond simple rotation
- [ ] Deep reorg recovery (auto-rebootstrap policy)
- [ ] Integration tests for sync loop + reorg handling
- [ ] Metrics export (Prometheus/OTel)

### Optional correctness hardening
- [ ] Receipts root validation (header `receiptsRoot`)
- [ ] Multi-peer cross-check for headers/receipts
- [ ] Stronger head source (beacon API / CL integration)
- [ ] Tombstone / "removed logs" handling for reorgs

---

## Later: v0.3+ - Performance & Storage Efficiency

- [ ] Bloom-based short-circuiting (skip blocks via `logsBloom`)
- [ ] Stronger log indexing (topic1-3, composite indexes)
- [ ] Post-sync `eth_getLogs` index build (address/topic0)
- [ ] Compression tuning (zstd) and cold storage formats
- [ ] Pipeline improvements (overlap headers/receipts/bodies fetching)
- [ ] Multi-peer scatter/gather fetching
- [ ] Optional calldata retention + full transaction objects

---

## Later: Correctness / Trust Hardening

- [ ] Optional receipts root validation
- [ ] Multi-peer cross-check (majority header hash / receiptsRoot)
- [ ] Confidence levels (unsafe vs safer head)

---

## Later: Fallbacks for Pruned History

- [ ] ETH: era1 / Portal for old ranges

---

## Later: Network Serving

- [ ] Serve `GetBlockHeaders` / `GetReceipts` for retained ranges
- [ ] Rate limiting + abuse protection

---

## Deferred: RPC Extras

Ponder compatibility and additional endpoints:
- [ ] `eth_getBlockByHash` (reorg traversal)
- [ ] `eth_call` (requires state; proxy or RESS approach)
- [ ] `eth_subscribe` (WS with polling fallback)
- [ ] `eth_getBlockReceipts` / `eth_getTransactionReceipt`
- [ ] `debug_traceBlockByNumber` / `debug_traceBlockByHash`
- [ ] `net_version`, `web3_clientVersion`

---

## Out of Scope

- Full execution / state / traces
- **OP Stack L2s (Base, Optimism):** These chains use libp2p (not devp2p) with a different protocol. P2P does not serve receipts directly; blocks must be executed to derive them.
