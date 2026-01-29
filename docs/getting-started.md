# Getting Started

## Prerequisites

- Rust toolchain (1.70+)
- Git access for Reth dependencies

## Installation

Clone the repository and build:

```bash
git clone https://github.com/vicnaum/stateless-history-node.git
cd stateless-history-node

# Development build
cargo build --manifest-path node/Cargo.toml

# Production build (recommended)
cargo build --manifest-path node/Cargo.toml --release
```

## First Run

Start syncing from Uniswap V2 deployment (block 10,000,000):

```bash
cargo run --release --manifest-path node/Cargo.toml
```

The node will:
1. Show a DOS-style splash screen while connecting to Ethereum mainnet peers
2. Display a fullscreen TUI dashboard with real-time sync progress
3. Begin backfilling blocks from 10,000,000 to head
4. Start the RPC server on `127.0.0.1:8545` once synced
5. Switch to follow mode once caught up

Press `q` to quit gracefully. The node persists checkpoints and resumes on restart.

### Headless / CI environments

If you don't want the fullscreen TUI (e.g., running in a script or pipe):

```bash
cargo run --release --manifest-path node/Cargo.toml -- --no-tui
```

This falls back to legacy indicatif progress bars on stderr.

## Basic Usage

### Custom start block

```bash
cargo run --release --manifest-path node/Cargo.toml -- --start-block 15000000
```

### Sync a specific range

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  --start-block 18000000 --end-block 18100000
```

### Custom data directory

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  --data-dir /path/to/data
```

### Custom RPC port

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  --rpc-bind 0.0.0.0:9545
```

## Verify RPC

Once synced, test the RPC:

```bash
# Check chain ID
curl -s -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_chainId","params":[]}' \
  http://127.0.0.1:8545

# Check synced block number
curl -s -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","id":1,"method":"eth_blockNumber","params":[]}' \
  http://127.0.0.1:8545
```

## TUI Dashboard

The default UI is a fullscreen terminal dashboard showing:
- Sync phase and progress
- Blocks coverage map (braille visualization)
- Speed chart with current/average/peak
- Network, queue, storage, and DB panels
- Real-time log viewer

Press `q` to quit. The dashboard requires a terminal with UTF-8 support.

## Debugging

### Increase verbosity

```bash
# Info level
cargo run --release --manifest-path node/Cargo.toml -- -v

# Debug level
cargo run --release --manifest-path node/Cargo.toml -- -vv

# Trace level
cargo run --release --manifest-path node/Cargo.toml -- -vvv
```

### Debug dump via signal

Send `SIGUSR1` to print sync + peer-health status:

```bash
kill -USR1 $(pgrep stateless-history)
```

### Check storage stats

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  db stats --data-dir data
```

## Next Steps

- See [Configuration](configuration.md) for all CLI options
- See [Architecture](../ARCHITECTURE.md) for system internals
- See [SPEC.md](../SPEC.md) for RPC semantics and error codes
- See [UI Designs](UI_DESIGNS.md) for TUI dashboard design reference
