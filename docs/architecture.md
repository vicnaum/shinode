# Architecture

A minimal, stateless Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset. It does not execute transactions or maintain state (no EVM, no state trie).

## Design Goals

- **Stateless by design**: no EVM execution, no state trie/database, no archive-state growth
- **Small, auditable surface**: minimal persistence and RPC for indexers
- **Reuse Reth primitives**: networking, static-file patterns, safety defaults
- **Future-proof architecture**: clear boundaries, stable storage contract

### Non-Goals

- Full execution/state, archive state, or EVM traces
- Transaction pool or mempool APIs
- "Full node" correctness guarantees (trust model is intentionally pragmatic)

## Module Structure

```
node/src/
├── main.rs       - CLI dispatch to run::run_sync() or subcommand handlers
├── bin/          - Demo binaries (color-test splash, ui-mock TUI preview)
├── cli/          - Config and argument parsing, NodeConfig defaults
├── run/          - Orchestration: startup (TUI splash), sync execution, follow mode, cleanup
├── p2p/          - devp2p networking (Reth primitives), peer pool, block fetching, P2P stats
├── sync/         - Sync pipeline: scheduling, fetching, processing, DB writing, coverage tracking
├── storage/      - Sharded static-file storage (segments + WAL + bitset + aggregate stats)
├── rpc/          - JSON-RPC server (jsonrpsee) with request stats tracking
├── ui/           - Ratatui TUI dashboard (default) and indicatif progress bars (fallback)
├── logging/      - JSON logs, TUI log capture, run reports, resource metrics, SIGUSR1 handler
└── metrics.rs    - Lightweight helpers for progress and summaries
```

## Data Flow

```
                              ┌─────────────────────────────────────────────┐
                              │               run::run_sync()               │
                              │  (orchestrates startup → sync → follow)     │
                              └──────────────────────┬──────────────────────┘
                                                     │
              ┌──────────────────────────────────────┼──────────────────────────────────────┐
              │                                      │                                      │
              ▼                                      ▼                                      ▼
      ┌───────────────┐                     ┌───────────────┐                      ┌───────────────┐
      │     P2P       │                     │    Storage    │                      │     RPC       │
      │  (peer pool,  │────fetch blocks────▶│  (sharded     │◀────read queries─────│  (JSON-RPC    │
      │   fetching)   │                     │   static      │                      │   server)     │
      └───────────────┘                     │   files)      │                      └───────────────┘
              │                             └───────────────┘                             │
              │                                      ▲                                    │
              │                                      │                                    │
              ▼                                      │                                    ▼
      ┌───────────────┐                              │                           ┌───────────────┐
      │     Sync      │                              │                           │   TUI / UI    │
      │  (scheduler,  │──────write blocks────────────┘                           │  (dashboard,  │
      │   pipeline)   │                                                          │   stats)      │
      └───────┬───────┘                                                          └───────┬───────┘
              │                                                                          │
              └──────────── SyncProgressStats (atomics) ─────────────────────────────────┘
```

1. **P2P** connects to Ethereum mainnet peers and fetches headers, bodies, and receipts (eth/68-70).
2. **Sync** schedules work, manages batching and retries, and processes blocks. Tracks progress via atomic counters.
3. **Storage** persists data in sharded static files with WAL for out-of-order ingestion.
4. **RPC** serves queries from storage (logs derived on-demand from receipts, segment readers cached via LRU).
5. **Run** orchestrates the lifecycle: TUI splash, storage open, P2P connect, fast-sync, follow mode, shutdown.
6. **UI** renders the fullscreen ratatui TUI dashboard (or indicatif progress bars with `--no-tui`).

## Key Types

| Type | Location | Description |
|------|----------|-------------|
| `NodeConfig` | `cli/mod.rs` | CLI configuration and defaults |
| `Storage` | `storage/sharded/mod.rs` | Sharded static-file backend |
| `PeerWorkScheduler` | `sync/historical/scheduler.rs` | Work queue and peer assignment |
| `PeerHealthTracker` | `sync/historical/scheduler.rs` | AIMD batch sizing, quality scoring, bans |
| `DbWriter` | `sync/historical/db_writer.rs` | WAL-based batched writes with compaction |
| `PeerPool` | `p2p/mod.rs` | Active peer sessions with head tracking |
| `SyncProgressStats` | `sync/mod.rs` | Atomic progress counters |
| `TuiController` | `ui/tui.rs` | Terminal I/O and rendering loop |

## Data Model

### What We Ingest

To serve `eth_getLogs` correctly (including `transactionHash`/`transactionIndex`/`logIndex`):

- **Headers** (canonical chain): number/hash/parent/timestamp + `logsBloom`
- **Tx hashes per block** (from block bodies)
- **Receipts** per block (from `GetReceipts*`), logs derived at query time

### Retention

- **Current**: headers + tx hashes + tx metadata (no calldata) + full receipts
- Logs derived at query time from receipts
- Withdrawals not stored

### Trust Model

- **Head source**: best-effort tracking from P2P peer views
- **Reorg handling**: rollback within configured window (default: 64 blocks)
- **Future**: integrate beacon API for `safe`/`finalized` head signals

## Storage Layout

```
data_dir/
├── meta.json                          # Schema version, chain id, checkpoints
├── sealed_shards.cache                # Fast-load cache for sealed shards
└── static/shards/<shard_start>/       # One directory per shard
    ├── shard.json                     # Shard metadata (stats, compaction phase)
    ├── present.bitset                 # Per-block presence tracking
    ├── pending.bitset                 # WAL block tracking (for resume)
    ├── state/staging.wal              # Write-ahead log (during ingestion)
    └── sorted/                        # Compacted segments
        ├── headers
        ├── tx_hashes
        ├── tx_meta
        ├── receipts
        └── block_sizes
```

Shards are sized by `--shard-size` (default: 10,000 blocks). Per-shard compaction triggers as shards fill.

`ShardMeta` tracks: `total_transactions`, `total_receipts`, `total_logs`, `disk_bytes_*` per segment.

## Sync Modes

### Fast-Sync (Historical)

For blocks older than `head - rollback_window`:

- Schedule missing blocks across shards (filtered by presence bitsets)
- Fetch concurrently from multiple peers (AIMD batch sizing, quality scoring)
- Write to per-shard WAL (out-of-order OK)
- Compact shards as they complete; seal with content hash
- Optional `--defer-compaction`: skip inline compaction, compact later

### Follow Mode (Live)

For blocks near head:

- Track head from P2P peer probes
- Fetch blocks via ingest pipeline with reorder buffer
- Handle reorgs via rollback within `--rollback-window`
- RPC server starts after first "synced" edge

## RPC Surface

Minimal indexer-compatible subset:

| Method | Description |
|--------|-------------|
| `eth_chainId` | Chain ID |
| `eth_blockNumber` | Highest present block |
| `eth_getBlockByNumber` | Block by number (tx hashes only) |
| `eth_getLogs` | Filtered logs by range, address, topic0 |

Notes:
- `totalDifficulty` mocked to `0x0`
- `withdrawals` always `null` (not stored)
- All other methods return `-32601`

## TUI Dashboard

Fullscreen terminal dashboard (ratatui) with phase-aware layouts:

| Phase | Color | Display |
|-------|-------|---------|
| Startup | Yellow | DOS-style splash screen with ASCII art |
| Sync | Cyan | Progress bar, coverage map, speed chart, panels |
| Retry | Red | Same as Sync (escalation queue active) |
| Compact | Magenta | Shard compaction progress |
| Seal | Green | Shard sealing progress |
| Follow | Light Green | Synced status, RPC stats panel |

Includes: speed chart (1-min history), peer visualization, queue/storage/DB panels, log viewer.

Press `q` to quit. Use `--no-tui` for headless environments.

## References

- [Configuration](configuration.md) - Full CLI options
- [Getting Started](getting-started.md) - Installation and first run
- [UI Designs](UI_DESIGNS.md) - TUI dashboard design reference
