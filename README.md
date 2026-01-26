# Stateless History Node

A minimal, stateless Ethereum history indexer that backfills headers, receipts,
and logs from the EL P2P network and serves a small, indexer-friendly RPC
subset. It does not execute transactions or keep state.

## Status (v0.2.0)

What works:
- P2P range backfill from `start_block..head` and continuous follow mode.
- Live reorg handling within `--rollback-window` (delete-on-rollback).
- Sharded static-file persistence (Storage v2) with WAL staging and compaction.
- Logs derived on-demand from receipts; withdrawals not stored.
- RPC subset: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`.
- Graceful shutdown, restart-safe checkpoints, resume without redownload.
- UI with colored stage indicators and compaction/sealing progress.
- Priority escalation queue for difficult blocks.
- Atomic compaction with crash recovery.
- `--repair` command for storage recovery.
- `--log-resources` for CPU/memory/disk metrics.
- Modular codebase (run/, ui/, logging/ modules).

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
```

Stop with Ctrl+C. The node persists checkpoints and resumes on restart.

For detailed setup, see [docs/getting-started.md](docs/getting-started.md).

## Documentation

- [Getting Started](docs/getting-started.md) - Installation and first run
- [Configuration](docs/configuration.md) - Full CLI options reference
- [Architecture](ARCHITECTURE.md) - System overview and module structure
- [Specification](SPEC.md) - Detailed system specification
- [Roadmap](ROADMAP.md) - Current and planned features

## CLI Options (Common)

```
--start-block <u64>     First block to sync (default: 10_000_000)
--end-block <u64>       Optional final block to stop at
--data-dir <path>       Data directory (default: data)
--rpc-bind <ip:port>    RPC bind address (default: 127.0.0.1:8545)
--shard-size <u64>      Blocks per shard (default: 10_000)
--rollback-window <u64> Max reorg depth (default: 64)
--min-peers <u64>       Wait for N peers before sync (default: 1)
-v/-vv/-vvv             Verbosity levels
--repair                Repair storage (run as subcommand)
--log                   Enable all log artifacts
--log-resources         Include CPU/memory/disk metrics
```

See [docs/configuration.md](docs/configuration.md) for all options.

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
    cli/        CLI and config
    run/        Orchestration (startup, sync, cleanup)
    p2p/        devp2p networking
    sync/       Sync pipeline and scheduling
    storage/    Sharded static-file storage
    rpc/        JSON-RPC server
    ui/         Progress bars and status
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
