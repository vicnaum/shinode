# Architecture

A minimal, stateless Ethereum history indexer that backfills headers, receipts, and logs from the EL P2P network and serves an indexer-friendly RPC subset. It does not execute transactions or maintain state (no EVM, no state trie).

## Module Structure

```
node/src/
├── main.rs      - CLI dispatch to run::run_sync() or subcommand handlers
├── cli/         - Config and argument parsing, NodeConfig defaults
├── run/         - Orchestration: startup, sync execution, follow mode, cleanup
├── p2p/         - devp2p networking (Reth primitives), peer pool, block fetching
├── sync/        - Sync pipeline: scheduling, fetching, processing, DB writing
├── storage/     - Sharded static-file storage (segments + WAL + bitset)
├── rpc/         - JSON-RPC server (jsonrpsee)
├── ui/          - Progress bars, status display, DB stats output
├── logging/     - JSON logs, run reports, resource metrics, SIGUSR1 handler
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
              │                             └───────────────┘
              │                                      ▲
              │                                      │
              ▼                                      │
      ┌───────────────┐                              │
      │     Sync      │                              │
      │  (scheduler,  │──────write blocks────────────┘
      │   pipeline)   │
      └───────────────┘
```

1. **P2P** connects to Ethereum mainnet peers and fetches headers, bodies, and receipts.
2. **Sync** schedules work, manages batching and retries, and processes blocks.
3. **Storage** persists data in sharded static files with WAL for out-of-order ingestion.
4. **RPC** serves queries from storage (logs derived on-demand from receipts, segment readers cached via LRU).
5. **Run** orchestrates the lifecycle: startup, fast-sync, follow mode, graceful shutdown.

## Key Types

| Type | Location | Description |
|------|----------|-------------|
| `NodeConfig` | `cli/mod.rs` | CLI configuration and defaults |
| `Storage` | `storage/sharded/mod.rs` | Sharded static-file backend |
| `SyncScheduler` | `sync/historical/scheduler.rs` | Work queue and peer assignment |
| `DbWriter` | `sync/historical/db_writer.rs` | WAL-based batched writes |
| `PeerPool` | `p2p/mod.rs` | Active peer sessions with head tracking |
| `UIController` | `ui/mod.rs` | Progress bars and status display |

## Storage Layout

```
data_dir/
├── meta.json                          # Schema version, chain id, checkpoints
└── static/shards/<shard_start>/       # One directory per shard
    ├── shard.json                     # Shard metadata
    ├── present.bitset                 # Per-block presence tracking
    ├── state/staging.wal              # Write-ahead log (during ingestion)
    └── sorted/                        # Compacted segments
        ├── headers
        ├── tx_hashes
        ├── tx_meta
        ├── receipts
        └── block_sizes
```

Shards are sized by `--shard-size` (default: 10,000 blocks). During fast-sync, blocks are written out-of-order to the WAL and compacted into sorted segments on finalization.

## Sync Modes

### Fast-Sync (Historical)
For blocks older than `head - rollback_window`:
- Schedule missing blocks across shards (filtered by presence bitsets)
- Fetch concurrently from multiple peers (AIMD batch sizing)
- Write to per-shard WAL (out-of-order OK)
- Compact into sorted segments on completion

### Follow Mode (Live)
For blocks near head:
- Track head from P2P peer status
- Fetch blocks sequentially
- Write in-order to storage
- Handle reorgs via rollback within `--rollback-window`

## RPC Surface

Minimal indexer-compatible subset:
- `eth_chainId` - Chain ID
- `eth_blockNumber` - Highest present block
- `eth_getBlockByNumber` - Block by number (tx hashes only)
- `eth_getLogs` - Filtered logs by range, address, topic0

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
- `-v`/`-vv`/`-vvv` - Verbosity levels

## References

- [SPEC.md](SPEC.md) - Detailed system specification
- [ROADMAP.md](ROADMAP.md) - Current and planned features
- [docs/](docs/) - User-facing documentation
