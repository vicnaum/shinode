# helpers

## Purpose
Trait definitions and helper utilities for `eth_` RPC handling (loaders, execution helpers, config).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for `eth` helper traits.
- **Key items**: `EthBlocks`, `EthTransactions`, `EthFees`, `EthState`, `EthCall`, `TraceExt`

### `block.rs`
- **Role**: Block loading traits and helpers.
- **Key items**: `EthBlocks`, `LoadBlock`

### `blocking_task.rs`
- **Role**: Spawn-blocking helper trait for offloading heavy RPC work.
- **Key items**: `SpawnBlocking`

### `call.rs`
- **Role**: Call execution traits and EVM call helpers.
- **Key items**: `EthCall`, `Call`

### `config.rs`
- **Role**: `eth_config` endpoint and fork/precompile config builder.
- **Key items**: `EthConfigApi`, `EthConfigHandler`, `config()`

### `estimate.rs`
- **Role**: Gas estimation logic and helper trait.
- **Key items**: `EstimateCall`, `estimate_gas_with()`
- **Knobs / invariants**: Disables EIP-3607/basefee for estimate; uses binary search.

### `fee.rs`
- **Role**: Fee history and gas oracle helpers.
- **Key items**: `EthFees`, `LoadFee`, `fee_history()`, `gas_price()`

### `pending_block.rs`
- **Role**: Pending block access and environment builders.
- **Key items**: `LoadPendingBlock`, `PendingEnvBuilder`, `BuildPendingEnv`

### `receipt.rs`
- **Role**: Receipt loading trait for `eth_` endpoints.
- **Key items**: `LoadReceipt`

### `signer.rs`
- **Role**: Signing abstraction for transaction requests.
- **Key items**: `EthSigner`

### `spec.rs`
- **Role**: Chain spec accessor trait for `eth_` requests.
- **Key items**: `EthApiSpec`

### `state.rs`
- **Role**: State access traits for account/storage/proofs.
- **Key items**: `EthState`, `LoadState`

### `trace.rs`
- **Role**: Tracing trait for transaction/block traces.
- **Key items**: `Trace`

### `transaction.rs`
- **Role**: Transaction submission and lookup traits.
- **Key items**: `EthTransactions`, `LoadTransaction`

## End-to-end flow (high level)
- Implement `Load*` traits to provide database access primitives.
- Compose `Eth*` traits to expose RPC behaviors by namespace.
- Use `SpawnBlocking` for CPU or blocking IO operations.
- Extend with `TraceExt` for tracing-specific flows.

## Key APIs (no snippets)
- `EthBlocks`, `EthTransactions`, `EthFees`, `EthState`, `EthCall`
- `LoadBlock`, `LoadTransaction`, `LoadState`, `LoadFee`
