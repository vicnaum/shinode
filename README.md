# Stateless History Node

A minimal, stateless Ethereum history indexer that backfills headers, receipts,
and logs from the EL P2P network and serves a small, indexer-friendly RPC
subset. It does not execute transactions or keep state.

## Status (v0.3.0-dev)

What works:
- P2P range backfill from `start_block..head` and continuous follow mode.
- Live reorg handling within `--rollback-window` (delete-on-rollback).
- Sharded static-file persistence (Storage v2) with WAL staging and compaction.
- Logs derived on-demand from receipts; withdrawals not stored.
- RPC subset: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`.
- Graceful shutdown, restart-safe checkpoints, resume without redownload.
- **Fullscreen ratatui TUI dashboard** with real-time speed chart, coverage map, peer/storage/DB/RPC panels, and log viewer (disable with `--no-tui`).
- DOS-style splash screen during startup with animated connection status.
- Priority escalation queue for difficult blocks.
- Atomic compaction with crash recovery and per-shard compaction during fast-sync.
- `--repair` command for storage recovery.
- `--log-resources` for CPU/memory/disk metrics.
- LRU segment reader cache for RPC performance.
- Peer stats tracking (discovery, sessions, genesis mismatches).
- RPC request metrics displayed in TUI (follow mode).

What does not yet:
- Additional RPC methods (`eth_getBlockByHash`, receipts endpoints, WS).
- Stateful calls (`eth_call`) or traces.
- Metrics export (Prometheus/OTel).

## Philosophy

- **Stateless by design**: no EVM execution or state trie.
- **Small, auditable surface**: minimal persistence and RPC for indexers.
- **Reuse Reth primitives**: networking, static-file storage, and safety defaults.

## Quick Start

```bash
# Build
cargo build --manifest-path node/Cargo.toml --release

# Run (syncs from block 10M to head)
cargo run --release --manifest-path node/Cargo.toml

# Custom range
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 18000000 --end-block 18100000

# Disable TUI (use legacy progress bars)
cargo run --release --manifest-path node/Cargo.toml -- --no-tui
```

Stop with Ctrl+C. The node persists checkpoints and resumes on restart.

For detailed setup, see [docs/getting-started.md](docs/getting-started.md).

## Documentation

- [Getting Started](docs/getting-started.md) - Installation and first run
- [Configuration](docs/configuration.md) - Full CLI options reference
- [Architecture](ARCHITECTURE.md) - System overview and module structure
- [Specification](SPEC.md) - Detailed system specification
- [Roadmap](ROADMAP.md) - Current and planned features
- [UI Designs](docs/UI_DESIGNS.md) - TUI dashboard design reference

## CLI Options (Common)

```
--start-block <u64>     First block to sync (default: 10_000_000)
--end-block <u64>       Optional final block to stop at
--data-dir <path>       Data directory (default: data)
--rpc-bind <ip:port>    RPC bind address (default: 127.0.0.1:8545)
--shard-size <u64>      Blocks per shard (default: 10_000)
--rollback-window <u64> Max reorg depth (default: 64)
--min-peers <u64>       Wait for N peers before sync (default: 1)
--no-tui                Disable fullscreen TUI dashboard
-v/-vv/-vvv             Verbosity levels
--repair                Repair storage (run as subcommand)
--log                   Enable all log artifacts
--log-resources         Include CPU/memory/disk metrics
```

See [docs/configuration.md](docs/configuration.md) for all options.

## TUI Dashboard

The node features a fullscreen terminal dashboard (powered by ratatui) that shows:

- **Phase indicator**: Startup > Sync > Retry > Compact > Seal > Follow
- **Progress bar** with percentage and block counts
- **Blocks coverage map** using braille characters with color gradient
- **Speed chart** with 1-minute history, current/average/peak speeds, and ETA
- **Network panel**: peer visualization (active/idle/stale dots), chain head
- **Queue panel**: remaining/inflight/retry block counts
- **Storage panel**: per-segment size breakdown, total size, write rate
- **DB panel**: block/transaction/log counts, shard compaction status
- **RPC panel** (follow mode): request rates, method counters, errors
- **Log viewer**: real-time log entries with level coloring

Press `q` to quit. Use `--no-tui` for headless environments.

## Debugging

Send `SIGUSR1` to print sync + peer-health debug dump:

```bash
kill -USR1 <pid>
```

## RPC Support

Implemented:
- `eth_chainId`
- `eth_blockNumber`
- `eth_getBlockByNumber` (with `includeTransactions=false`)
- `eth_getLogs` (filtered by block range, address, topic0)

Notes:
- `totalDifficulty` is mocked to `0x0`.
- `withdrawals` are always `null` (not stored).
- All other methods return `-32601`.

## Project Layout

```
node/           Stateless history node implementation
  src/
    bin/        Demo binaries (color-test, ui-mock)
    cli/        CLI and config
    run/        Orchestration (startup, sync, cleanup)
    p2p/        devp2p networking
    sync/       Sync pipeline and scheduling
    storage/    Sharded static-file storage
    rpc/        JSON-RPC server
    ui/         TUI dashboard and progress bars
    logging/    JSON logs, reports, metrics
docs/           User-facing documentation
spec/           Supporting docs (worklogs, research)
```

## Profiling

CPU sampling (macOS: use Instruments; Linux: use samply):

```bash
samply record -- \
  cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 10_000_000 --end-block 10_010_000
```

Timeline artifacts:

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 10_000_000 --end-block 10_010_000 --log-trace
```

## Allocator

Jemalloc is enabled by default. To disable:

```bash
cargo build --manifest-path node/Cargo.toml --no-default-features
```

On Linux/glibc, you can also tune malloc arenas:

```bash
MALLOC_ARENA_MAX=2 cargo run --release --manifest-path node/Cargo.toml
```
