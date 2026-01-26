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
1. Connect to Ethereum mainnet peers via devp2p
2. Begin backfilling blocks from 10,000,000 to head
3. Start the RPC server on `127.0.0.1:8545`
4. Switch to follow mode once caught up

Stop with Ctrl+C. The node persists checkpoints and resumes on restart.

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
