# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] - 2026-02-04

### Added
- Fullscreen ratatui TUI dashboard (default; disable with `--no-tui`)
  - Phase indicator, progress bar, coverage map, speed chart
  - Network, queue, storage, DB, and RPC panels
  - DOS-style splash screen with animated connection status
  - Real-time log viewer with level coloring
- TUI log capture (circular buffer, suppresses stdout in TUI mode)
- Per-shard compaction during fast-sync (shards compact as they fill)
- P2P stats tracking: discovery count, genesis mismatches, sessions
- RPC stats tracking: total requests, per-method counters, errors
- Coverage tracker with bucket-based braille visualization
- Peak speed tracking and follow-mode staleness detection
- Stale-peer banning with 120s cooldown and async re-probe
- Peer feeder rotation for fairness
- Stall detection with peer health dump (30s threshold)
- Sealed shard cache for fast startup on slow storage
- `db compact` subcommand with progress bars, JSON logging, per-shard timing
- `db rebuild-cache` subcommand to rebuild sealed shard cache
- `--defer-compaction` flag to skip inline compaction during fast-sync
- SHiNode branding: renamed binary to `shinode`, website (shinode.rs)
- MIT/Apache-2.0 dual licensing
- Workspace `Cargo.toml`

### Changed
- Binary renamed from `stateless-history-node` to `shinode`
- Storage open performance: 8MB read buffers, in-memory WAL reads, cached disk stats
- ShardMeta now tracks `total_logs`, `total_transactions`, `total_receipts`, `disk_bytes_*`
- StorageAggregateStats for cheap cross-shard rollup (no disk I/O)
- Read-only segment readers (prevents follow-mode file-lock crash)

### Fixed
- Follow-mode stale-peer spin-loop
- Follow-mode head desync
- Follow-mode log dedup

## [0.2.0] - 2026-01-27

### Added
- Persist peer cache (`peers.json` with TTL + cap)
- Peer warmup gate (`--min-peers`)
- Peer pool warmup with async head probes
- Fast-sync WAL batch writes (out-of-order ingestion)
- Log events instrumentation (`--log-events`)
- Optional jemalloc allocator (default feature)
- Backpressure and memory caps for queues
- Resume without redownload (skip present blocks, recompact dirty shards)
- Follow mode with tail scheduling
- Priority escalation queue for difficult blocks
- Atomic compaction with crash recovery
- `--repair` command for storage recovery
- `--log-resources` for CPU/memory/disk metrics
- AIMD batch sizing per-peer (adaptive concurrency)
- Peer quality scoring and temporary bans
- LRU segment reader cache

### Changed
- Compaction memory hardening (streaming WAL, serialized compactions)
- Safe boundary switch near reorg window
- Unified ingest pipeline with log artifacts
- Modular codebase (run/, ui/, logging/ modules)
- UI improvements (colored stages, compaction/sealing progress)

## [0.1.0] - 2026-01-22

### Added
- CLI configuration and storage schema
- Range sync with canonical chain tracking and reorg detection
- Multi-peer concurrent fetching
- NippyJar static-file persistence with idempotent writes and rollback
- Sharded storage (v2) with per-shard WAL and out-of-order ingestion
- JSON-RPC server: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`
- Graceful shutdown with checkpoint persistence
- Verbosity levels (`-v`/`-vv`/`-vvv`) and progress UI
- Chrome trace and event log artifacts
- Chunked concurrent downloads with AIMD batch sizing
- Follow mode with live reorg handling within rollback window
- Shard sealing for completed ranges
