# Roadmap

## Done: v0.1 MVP - "Synced History RPC for Indexing"

A long-running service that **backfills**, **stays in sync with the head**, **persists history**, and exposes an **indexer-compatible JSON-RPC subset** (target: **rindexer**) so you can index Uniswap without paid RPC.

Intentionally **stateless**: no EVM execution, no state trie, no archive-state growth. Ingests EL history artifacts (headers + bodies/tx hashes + receipts/logs) and serves them back.

### Completed milestones


| Version | Focus            | Key deliverables                                                        |
| ------- | ---------------- | ----------------------------------------------------------------------- |
| v0.1.0  | Product contract | CLI/config, storage schema, retention modes, test strategy              |
| v0.1.1  | Sync + ingest    | Range sync, canonical chain, reorg detection, multi-peer fetching       |
| v0.1.2  | Persistence      | NippyJar static files, idempotent writes, rollback                      |
| v0.1.3  | RPC server       | `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs` |
| v0.1.4  | Operator UX      | CLI config, graceful shutdown, verbosity, progress UI, log artifacts    |
| v0.1.5  | Fast sync        | Chunked concurrent downloads, AIMD batch sizing, bounded buffering      |
| v0.1.6  | Live sync        | Follow mode, live reorg handling within rollback window                 |
| v0.1.7  | Storage v2       | Sharded storage, per-shard WAL, out-of-order ingestion, shard sealing   |


### Release criteria met

- Fresh start: backfills from configured start block to head
- Steady-state: continues indexing new blocks and survives reorgs
- Restart safe: resumes without corrupting data or duplicating logs
- Indexer compatible: rindexer runs against the supported RPC subset

---

## Done: v0.2.0 - Reliability for Long-Running Operation

Focus: stability and performance hardening for production use.

### Completed

- Persist peer cache (`peers.json` with TTL + cap)
- Peer warmup gate (`--min-peers`)
- Peer pool warmup (async head probes)
- Fast-sync WAL batch writes (out-of-order)
- Log events instrumentation (`--log-events`)
- Compaction memory hardening (streaming WAL, serialized compactions)
- Optional jemalloc allocator (default feature)
- Backpressure + memory caps for queues
- Safe boundary switch near reorg window
- Unified ingest pipeline with log artifacts
- Resume without redownload (skip present blocks, recompact dirty shards)
- Follow mode with tail scheduling
- UI improvements (colored stages, compaction/sealing progress)
- Priority escalation queue for difficult blocks
- Atomic compaction with crash recovery
- `--repair` command for storage recovery
- `--log-resources` for CPU/memory/disk metrics
- Modular codebase (run/, ui/, logging/ modules)
- AIMD batch sizing per-peer (adaptive concurrency)
- Peer quality scoring and temporary bans
- LRU segment reader cache (fixes RPC performance regression)

---

## Done: v0.3.0 - TUI Dashboard & Observability

Focus: real-time operator visibility via fullscreen terminal dashboard.

### Completed

- **Fullscreen ratatui TUI dashboard** (default; disable with `--no-tui`)
  - Phase indicator: Startup > Sync > Retry > Compact > Seal > Follow
  - Progress bar with percentage and block/shard counts
  - Blocks coverage map (braille characters with color gradient)
  - Speed chart (braille line graph with 1-minute history, current/avg/peak/ETA)
  - Network panel (peer dots: active/idle/stale/gone, never-shrinking visualization)
  - Queue panel (remaining/inflight/retry counts)
  - Storage panel (per-segment size breakdown, total GiB, write rate MB/s)
  - DB panel (blocks/txns/logs counts, shards compacted/total)
  - RPC panel (follow mode: status, req/s, method counters, errors)
  - Log viewer (real-time tracing events with level coloring)
  - DOS-style splash screen on startup with animated connection status
- **TUI log capture** (circular buffer, suppresses stdout in TUI mode)
- **Per-shard compaction during fast-sync** (shards compact as they fill, not at end)
- **ShardMeta enhancements**: `total_logs`, `total_transactions`, `total_receipts`, `disk_bytes_*`
- **StorageAggregateStats**: cheap cross-shard rollup (no disk I/O)
- **Read-only segment readers** (prevents follow-mode file-lock crash)
- **P2P stats tracking**: discovery count, genesis mismatches, sessions established/closed
- **RPC stats tracking**: total requests, per-method counters, error counts
- **Coverage tracker**: bucket-based visualization for blocks map
- **Peak speed tracking** and follow-mode staleness detection (30s threshold)
- **Stale-peer banning** with 120s cooldown and async re-probe
- **Peer feeder rotation** for fairness
- **Stall detection** with peer health dump (30s threshold)
- Follow-mode fixes: stale-peer spin-loop, head desync, log dedup
- **Sealed shard cache** for fast startup on slow storage (`sealed_shards.cache`)
- **`db compact`** subcommand with progress bars, JSON logging, per-shard timing
- **`db rebuild-cache`** subcommand to rebuild sealed shard cache
- **`--defer-compaction`** flag to skip inline compaction during fast-sync
- **Storage open performance**: 8MB read buffers, in-memory WAL reads, cached disk stats
- **SHiNode branding**: renamed binary (`shinode`), website (shinode.rs), landing page
- **MIT/Apache-2.0 dual licensing**, workspace `Cargo.toml`
- **Release polish**: CHANGELOG, ROADMAP updates, docs refresh

---

## Next: v0.4 - Testing & Refactoring

- [ ] Comprehensive test coverage across all subsystems (P2P, sync, storage, RPC, UI)
- [ ] Integration tests for sync loop + reorg handling
- [ ] Codebase refactor: clean up dead/duplicated code, reorganize modules
- [ ] Separate into library crate (`shinode-core`) and CLI/TUI binary

---

## Future: Performance & Storage

- Bloom-based short-circuiting (skip blocks via `logsBloom`)
- Stronger log indexing (topic1-3, composite indexes)
- Post-sync `eth_getLogs` index build (address/topic0)
- Compression tuning (zstd) and cold storage formats
- Pipeline improvements (overlap headers/receipts/bodies fetching)
- Multi-peer scatter/gather fetching
- Optional calldata retention + full transaction objects

---

## Future: Correctness & Trust

- Deep reorg recovery (auto-rebootstrap policy)
- Receipts root validation (verify against header `receiptsRoot`)
- Multi-peer cross-check (majority header hash / receiptsRoot)
- Stronger head source (beacon API / CL integration)
- Tombstone / "removed logs" handling for reorgs
- Confidence levels (unsafe vs safer head)

---

## Future: Network & Observability

- Metrics export (Prometheus/OTel)
- Serve `GetBlockHeaders` / `GetReceipts` for retained ranges
- Rate limiting + abuse protection
- ETH: era1 / Portal for old ranges

---

## Future: RPC Extras

Ponder compatibility and additional endpoints:

- `eth_getBlockByHash` (reorg traversal)
- `eth_call` (requires state; proxy or RESS approach)
- `eth_subscribe` (WS with polling fallback)
- `eth_getBlockReceipts` / `eth_getTransactionReceipt`
- `debug_traceBlockByNumber` / `debug_traceBlockByHash`
- `net_version`, `web3_clientVersion`

---

## Out of Scope

- Full execution / state / traces
- **OP Stack L2s (Base, Optimism):** These chains use libp2p (not devp2p) with a different protocol. P2P does not serve receipts directly; blocks must be executed to derive them.
