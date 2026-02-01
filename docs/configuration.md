# Configuration

Full reference for CLI options and environment variables.

## Core Options

| Option | Default | Description |
|--------|---------|-------------|
| `--chain-id <u64>` | `1` | Chain ID for RPC (Ethereum mainnet only) |
| `--data-dir <path>` | `data` | Base data directory for storage |
| `--peer-cache-dir <path>` | (data dir) | Directory for persisted peer cache; defaults to data directory if unset |
| `--start-block <u64>` | `10000000` | First block to backfill (pre-Uniswap V2) |
| `--end-block <u64>` | - | Optional final block to stop at |
| `--shard-size <u64>` | `10000` | Blocks per storage shard |
| `--rollback-window <u64>` | `64` | Max rollback depth for reorgs |
| `--retention-mode <mode>` | `full` | Retention policy (`full` only) |
| `--head-source <source>` | `p2p` | Head source (`p2p` only) |
| `--reorg-strategy <strategy>` | `delete` | Rollback strategy (`delete` only) |

## Display Options

| Option | Default | Description |
|--------|---------|-------------|
| `--no-tui` | `false` | Disable fullscreen TUI dashboard; use legacy progress bars |

## RPC Options

| Option | Default | Description |
|--------|---------|-------------|
| `--rpc-bind <ip:port>` | `127.0.0.1:8545` | RPC server bind address |
| `--rpc-max-request-body-bytes <u32>` | `10485760` | Max request body size (10MB) |
| `--rpc-max-response-body-bytes <u32>` | `104857600` | Max response body size (100MB) |
| `--rpc-max-connections <u32>` | `100` | Max concurrent connections |
| `--rpc-max-batch-requests <u32>` | `100` | Max batch requests (`0` = unlimited) |
| `--rpc-max-blocks-per-filter <u64>` | `10000` | Max blocks per `eth_getLogs` (`0` = unlimited) |
| `--rpc-max-logs-per-response <u64>` | `100000` | Max logs per response (`0` = unlimited) |

## Peer Options

| Option | Default | Description |
|--------|---------|-------------|
| `--min-peers <u64>` | `1` | Min peers before starting sync |

## Ingest Tuning

| Option | Default | Description |
|--------|---------|-------------|
| `--fast-sync-chunk-size <u64>` | `32` | Initial blocks per peer batch |
| `--fast-sync-chunk-max <u64>` | (4x chunk-size) | Hard cap for AIMD upper bound |
| `--fast-sync-max-inflight <u32>` | `32` | Max concurrent peer batches |
| `--fast-sync-max-buffered-blocks <u64>` | `8192` | Max buffered blocks |
| `--db-write-batch-blocks <u64>` | `512` | Batch size for fast-sync WAL writes |
| `--db-write-flush-interval-ms <u64>` | - | Optional time-based flush interval |

## Verbosity

| Option | Effect |
|--------|--------|
| (none) | Errors only |
| `-v` | Info level (RPC requests, ingest progress) |
| `-vv` | Debug level (node internals) |
| `-vvv` | Trace level (detailed internals) |

`RUST_LOG` environment variable overrides all verbosity defaults.

## Logging Artifacts

| Option | Description |
|--------|-------------|
| `--log` | Enable all log outputs (trace, events, json, report, resources) |
| `--run-name <string>` | Label for output filenames (default: timestamp) |
| `--log-output-dir <path>` | Output directory for log artifacts (default: `logs`) |
| `--log-trace` | Emit Chrome trace (`.trace.json`) for timeline inspection |
| `--log-trace-filter <filter>` | Trace filter (default: `off,stateless_history_node=trace,...`) |
| `--log-trace-include-args` | Include span/event args in trace output (default: true) |
| `--log-trace-include-locations` | Include file+line info in trace output (default: true) |
| `--log-events` | Emit JSONL event log (`.events.jsonl`) for analysis |
| `--log-json` | Emit JSON structured logs (`.logs.jsonl`) |
| `--log-json-filter <filter>` | JSON log filter (default: `warn,stateless_history_node=debug`) |
| `--log-report` | Emit run summary report (`.report.json`) |
| `--log-resources` | Emit separate resource metrics JSONL file (CPU/memory/disk) |

## Operational Modes

| Option | Description |
|--------|-------------|
| `--repair` | Run storage repair/recovery without starting sync |
| `--defer-compaction` | Skip inline shard compaction during fast-sync; compact at finalize only (useful for HDD/slow storage) |

## Commands

### Sync (default)

```bash
cargo run --release --manifest-path node/Cargo.toml -- [OPTIONS]
```

### DB Stats

```bash
# Human-readable output
cargo run --release --manifest-path node/Cargo.toml -- \
  db stats --data-dir <path>

# JSON output
cargo run --release --manifest-path node/Cargo.toml -- \
  db stats --data-dir <path> --json
```

### DB Compact

```bash
# Compact all dirty shards and seal completed ones
cargo run --release --manifest-path node/Cargo.toml -- \
  db compact --data-dir <path>
```

### Repair

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  --repair --data-dir <path>
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Override log filter (e.g., `stateless_history_node=debug`) |
| `MALLOC_ARENA_MAX` | Limit glibc malloc arenas (Linux; e.g., `2`) |

## Allocator

Jemalloc is enabled by default. To disable:

```bash
cargo build --manifest-path node/Cargo.toml --no-default-features
```

## Storage Layout

After running, `data_dir` contains:

```
data_dir/
├── meta.json                          # Schema, chain id, checkpoints
├── peers.json                         # Persisted peer cache
└── static/shards/<shard_start>/       # One per shard
    ├── shard.json                     # Shard metadata (stats, compaction phase)
    ├── present.bitset                 # Block presence tracking
    ├── state/staging.wal              # WAL (during ingestion)
    └── sorted/                        # Compacted segments
        ├── headers
        ├── tx_hashes
        ├── tx_meta
        ├── receipts
        └── block_sizes
```

## Configuration Persistence

Storage-affecting settings are persisted in `meta.json` and validated on startup:
- `--shard-size`
- `--retention-mode`
- `--head-source`
- `--reorg-strategy`

If you change these, use a new `data_dir`. Runtime-only settings (verbosity, RPC limits, `--no-tui`) can be changed freely.
