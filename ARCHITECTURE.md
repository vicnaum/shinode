# Architecture

A minimal, stateless Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset. It does not execute transactions or maintain state (no EVM, no state trie).

## Module Structure

```
node/src/
├── main.rs      - CLI dispatch to run::run_sync() or subcommand handlers
├── bin/         - Demo binaries (color-test splash, ui-mock TUI preview)
├── cli/         - Config and argument parsing, NodeConfig defaults
├── run/         - Orchestration: startup (TUI splash), sync execution, follow mode, cleanup
├── p2p/         - devp2p networking (Reth primitives), peer pool, block fetching, P2P stats
├── sync/        - Sync pipeline: scheduling, fetching, processing, DB writing, coverage tracking
├── storage/     - Sharded static-file storage (segments + WAL + bitset + aggregate stats)
├── rpc/         - JSON-RPC server (jsonrpsee) with request stats tracking
├── ui/          - Ratatui TUI dashboard (default) and indicatif progress bars (fallback)
├── logging/     - JSON logs, TUI log capture, run reports, resource metrics, SIGUSR1 handler
├── metrics.rs   - Lightweight helpers for progress and summaries
└── test_utils.rs - Shared test utilities
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
2. **Sync** schedules work, manages batching and retries, and processes blocks. Tracks progress via atomic counters in `SyncProgressStats`.
3. **Storage** persists data in sharded static files with WAL for out-of-order ingestion. Tracks per-shard stats (blocks, transactions, logs, disk bytes).
4. **RPC** serves queries from storage (logs derived on-demand from receipts, segment readers cached via LRU). Reports request counters back to stats.
5. **Run** orchestrates the lifecycle: early TUI splash screen, storage open, P2P connect, fast-sync, follow mode, graceful shutdown.
6. **UI** renders the fullscreen ratatui TUI dashboard (or indicatif progress bars with `--no-tui`). Reads all stats atomically at 10 FPS.

## Key Types

| Type | Location | Description |
|------|----------|-------------|
| `NodeConfig` | `cli/mod.rs` | CLI configuration and defaults |
| `Storage` | `storage/sharded/mod.rs` | Sharded static-file backend |
| `PeerWorkScheduler` | `sync/historical/scheduler.rs` | Work queue and peer assignment |
| `PeerHealthTracker` | `sync/historical/scheduler.rs` | AIMD batch sizing, quality scoring, bans |
| `DbWriter` | `sync/historical/db_writer.rs` | WAL-based batched writes with per-shard compaction |
| `PeerPool` | `p2p/mod.rs` | Active peer sessions with head tracking |
| `SyncProgressStats` | `sync/mod.rs` | Atomic progress counters (peers, blocks, coverage, RPC, DB) |
| `TuiState` | `ui/tui.rs` | Complete TUI display state |
| `TuiController` | `ui/tui.rs` | Terminal I/O and rendering loop |
| `UIController` | `ui/progress.rs` | Legacy indicatif progress bars |

## TUI Dashboard

The node features a fullscreen terminal dashboard with phase-aware layouts:

| Phase | Color | Display |
|-------|-------|---------|
| Startup | Yellow | DOS-style splash screen with ASCII art, config panel |
| Sync | Cyan | Progress bar, blocks coverage map, speed chart, queue/network/storage/DB panels |
| Retry | Red | Same as Sync (escalation queue active) |
| Compact | Magenta | Shard progress map, compaction panel |
| Seal | Green | Shard sealing progress |
| Follow | Light Green | Synced status with big ASCII block number, RPC stats panel |

All phases include a log viewer with timestamped, color-coded entries captured from the tracing system.

## Storage Layout

```
data_dir/
├── meta.json                          # Schema version, chain id, checkpoints
└── static/shards/<shard_start>/       # One directory per shard
    ├── shard.json                     # Shard metadata (stats, compaction phase)
    ├── present.bitset                 # Per-block presence tracking
    ├── state/staging.wal              # Write-ahead log (during ingestion)
    └── sorted/                        # Compacted segments
        ├── headers
        ├── tx_hashes
        ├── tx_meta
        ├── receipts
        └── block_sizes
```

Shards are sized by `--shard-size` (default: 10,000 blocks). During fast-sync, blocks are written out-of-order to the WAL and compacted into sorted segments. Per-shard compaction triggers as soon as all blocks in a shard are written (no waiting for full pipeline completion).

`ShardMeta` tracks: `total_transactions`, `total_receipts`, `total_logs`, `disk_bytes_*` per segment type. `StorageAggregateStats` provides cheap cross-shard rollup without disk I/O.

## Sync Modes

### Fast-Sync (Historical)
For blocks older than `head - rollback_window`:
- Schedule missing blocks across shards (filtered by presence bitsets)
- Fetch concurrently from multiple peers (AIMD batch sizing, quality scoring)
- Write to per-shard WAL (out-of-order OK), accumulate log counts
- Compact shards as they complete; seal completed shards with content hash
- Per-shard compaction triggers during sync (not just at finalization)
- `--defer-compaction`: skip inline compaction, compact all at finalize (better for HDD/slow storage)

### Follow Mode (Live)
For blocks near head:
- Track head from P2P peer probes (not session status)
- Fetch blocks via ingest pipeline with reorder buffer for in-order writes
- Handle reorgs via rollback within `--rollback-window`
- RPC server starts after first "synced" edge

## RPC Surface

Minimal indexer-compatible subset:
- `eth_chainId` - Chain ID
- `eth_blockNumber` - Highest present block
- `eth_getBlockByNumber` - Block by number (tx hashes only)
- `eth_getLogs` - Filtered logs by range, address, topic0

All methods track request counters via `SyncProgressStats` for TUI display.

See [SPEC.md](SPEC.md) for detailed semantics and error codes.

## Configuration

See [docs/configuration.md](docs/configuration.md) for full CLI options, or run:

```bash
cargo run --manifest-path node/Cargo.toml -- --help
```

Key options:
- `--start-block` / `--end-block` - Sync range
- `--shard-size` - Blocks per storage shard (recommend 1000+ for RPC workloads)
- `--rollback-window` - Max reorg depth (default: 64)
- `--rpc-bind` - RPC server address (default: localhost:8545)
- `--defer-compaction` - Skip inline compaction during fast-sync
- `--no-tui` - Disable fullscreen TUI dashboard
- `-v`/`-vv`/`-vvv` - Verbosity levels
- `db compact` - Compact dirty shards and seal completed ones (standalone)

## References

- [SPEC.md](SPEC.md) - Detailed system specification
- [ROADMAP.md](ROADMAP.md) - Current and planned features
- [docs/](docs/) - User-facing documentation
- [docs/UI_DESIGNS.md](docs/UI_DESIGNS.md) - TUI dashboard design reference
