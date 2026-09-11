# Getting Started

## Prerequisites

- Recent stable Rust toolchain (tested with 1.93.1)
- C compiler and linker (for native dependencies)
- Git access for Reth dependencies

**Linux (Ubuntu/Debian):**

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install build tools (gcc, make, linker)
apt update && apt install -y build-essential
```

**macOS:** Xcode Command Line Tools (`xcode-select --install`) provides the compiler.

## Installation

### Pre-built binaries

Download from the [latest release](https://github.com/vicnaum/shinode/releases/latest):

```bash
# Download, extract, and make executable
tar xzf shinode-linux-x86_64.tar.gz
chmod +x shinode-linux-x86_64
./shinode-linux-x86_64
```

### Install from source

```bash
cargo install --git https://github.com/vicnaum/shinode.git --path node
```

### Clone and build

```bash
git clone https://github.com/vicnaum/shinode.git
cd shinode

# Development build
cargo build

# Production build (recommended)
cargo build --release --locked
```

## First Run

Start syncing from Uniswap V2 deployment (block 10,000,000):

```bash
cargo run --release
```

The node will:
1. Show a DOS-style splash screen while connecting to Ethereum mainnet peers
2. Display a fullscreen TUI dashboard with real-time sync progress
3. Begin backfilling blocks from 10,000,000 to head
4. Start the RPC server on `127.0.0.1:8545` once synced
5. Switch to follow mode once caught up

Press `q` to quit gracefully. The node persists checkpoints and resumes on restart.

### Base mainnet

Build the current development checkout for Base support. Pick a recent Base block
number, replace the placeholder below, and use a directory separate from Ethereum:

```bash
cargo run --release -- --chain base --data-dir data/base \
  --start-block <recent-base-block> --p2p-port 30309 --rpc-bind 127.0.0.1:8546
```

On restart, use the same command without `--start-block` to reuse the saved lower
bound. For an older backfill, add `--trusted-peer enode://<public-key>@<host>:<port>`
for a node that serves that range; ordinary discovered peers may have pruned it.
Plan around roughly 30 days of history when relying on public Base P2P peers:
Base's [published pruned snapshots](https://github.com/base/base/blob/main/.env.mainnet)
retain 1,339,200 blocks (about 31 days at two seconds per block). Operators can
configure a different window, so this is not a guarantee for every peer. Older
backfills need a peer retaining and serving the requested bodies and receipts.
Shinode keeps the history it has already downloaded; upstream pruning does not
delete it locally.
Metadata prevents opening a populated directory as a different chain.

After catching up, query `http://127.0.0.1:8546`: `eth_chainId` returns `0x2105`.
Block queries return transaction hashes with `includeTransactions=false`; `true`
is unsupported. Log queries include user and deposit transaction logs with their
block-wide indices. A bounded run with `--end-block` exits after backfill rather
than serving continuously.

The initial target is the probed P2P head minus the rollback window; live follow
tracks the probed tip. Neither represents OP safe/finalized status or L1 finality.
Read [protocol compatibility](base-protocol.md) before choosing a historical range;
full live ingestion/restart/follow smoke tests remain part of release validation.

### Headless / CI environments

If you don't want the fullscreen TUI (e.g., running in a script or pipe):

```bash
cargo run --release --manifest-path node/Cargo.toml -- --no-tui
```

This falls back to legacy indicatif progress bars on stderr.

## Basic Usage

### Custom start block

```bash
cargo run --release -- --start-block 15000000
```

The default start block is 10,000,000 (Uniswap V2 Factory deployment). Here are deployment blocks for popular protocols you might want to index from:

| Protocol | Contract | Block | Date (UTC) |
|----------|----------|------:|------------|
| Uniswap V2 | Factory | 10,000,835 | 2020-05-04 |
| Aave V2 | LendingPool | 11,362,579 | 2020-12-03 |
| Uniswap V3 | Factory | 12,369,621 | 2021-05-04 |
| Aave V3 | Pool | 16,291,127 | 2022-12-29 |
| Uniswap V4 | PoolManager | 21,688,329 | 2025-01-23 |

### Sync a specific range

```bash
cargo run --release -- \
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
kill -USR1 $(pgrep shinode)
```

### Check storage stats

```bash
cargo run --release --manifest-path node/Cargo.toml -- \
  db stats --data-dir data
```

## Next Steps

- See [Configuration](configuration.md) for all CLI options
- See [Architecture](architecture.md) for system internals
- See [UI Designs](UI_DESIGNS.md) for TUI dashboard design reference
