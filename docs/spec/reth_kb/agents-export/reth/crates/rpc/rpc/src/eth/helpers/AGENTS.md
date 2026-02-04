# helpers

## Purpose
Helper implementations and utilities for the `eth` namespace: trait adapters, signer helpers, pending block support, and sync utilities.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for `eth` helper implementations and re-exports.
- **Key items**: `signer`, `sync_listener`, `types`
- **Interactions**: Re-exports `SyncListener` for `eth` APIs.

### `block.rs`
- **Role**: Implements block-related helper traits for `EthApi`.
- **Key items**: `EthBlocks`, `LoadBlock`, `EthApi`
- **Interactions**: Uses `RpcConvert` and `RpcNodeCore` trait bounds.

### `call.rs`
- **Role**: Implements call/estimate helpers for EVM execution endpoints.
- **Key items**: `EthCall`, `Call`, `EstimateCall`, `call_gas_limit()`, `max_simulate_blocks()`
- **Interactions**: Delegates limits to `EthApi` inner config.
- **Knobs / invariants**: Exposes gas cap, simulate block cap, and EVM memory limits.

### `fees.rs`
- **Role**: Fee history and gas oracle helpers for `EthApi`.
- **Key items**: `EthFees`, `LoadFee`, `GasPriceOracle`, `FeeHistoryCache`
- **Interactions**: Reads provider headers and cached fee history.

### `pending_block.rs`
- **Role**: Pending-block helpers and access to pending block state.
- **Key items**: `LoadPendingBlock`, `PendingEnvBuilder`, `PendingBlock`, `PendingBlockKind`
- **Interactions**: Accesses `EthApi` pending block mutex and env builder.

### `receipt.rs`
- **Role**: Receipt loading adapter for `EthApi`.
- **Key items**: `LoadReceipt`, `EthApi`
- **Interactions**: Ties `RpcConvert` to receipt conversion.

### `signer.rs`
- **Role**: Developer signer implementation for `eth_sign`/tx signing.
- **Key items**: `DevSigner`, `random_signers()`, `from_mnemonic()`, `sign_transaction()`, `sign_typed_data()`
- **Interactions**: Implements `EthSigner` over local private keys.
- **Knobs / invariants**: Mnemonic derivation uses `m/44'/60'/0'/0/{index}` path.

### `spec.rs`
- **Role**: Spec helper trait implementation for `EthApi`.
- **Key items**: `EthApiSpec`, `starting_block()`
- **Interactions**: Uses `RpcNodeCore` and `RpcConvert` bounds.

### `state.rs`
- **Role**: State access helpers for account/storage/proof queries.
- **Key items**: `EthState`, `LoadState`, `max_proof_window()`
- **Interactions**: Requires `LoadPendingBlock` for consistent state reads.
- **Knobs / invariants**: Proof window limit is configured via `eth_proof_window`.

### `sync_listener.rs`
- **Role**: Utility future that waits for sync completion.
- **Key items**: `SyncListener`, `new()`, `Future` impl
- **Interactions**: Polls `NetworkInfo::is_syncing` on a tick stream.
- **Knobs / invariants**: Completes immediately if not syncing.

### `trace.rs`
- **Role**: Trace helper trait implementation for `EthApi`.
- **Key items**: `Trace`, `EthApi`
- **Interactions**: Binds `RpcConvert` and EVM error conversions.

### `transaction.rs`
- **Role**: Transaction helpers: sending raw txs, signer access, pool integration.
- **Key items**: `EthTransactions`, `LoadTransaction`, `send_transaction()`, `send_raw_transaction_sync_timeout()`, `signers()`
- **Interactions**: Uses `TransactionPool`, blob sidecar conversion, and optional raw-tx forwarder.
- **Knobs / invariants**: Converts legacy blob sidecars to EIP-7594 when Osaka is active.

### `types.rs`
- **Role**: Ethereum-specific RPC converter type alias.
- **Key items**: `EthRpcConverter`
- **Interactions**: Combines `RpcConverter` with `EthEvmConfig` and receipt converter.

## End-to-end flow (high level)
- `EthApi` builds with an `EthRpcConverter` for Ethereum types.
- Helper traits (`EthBlocks`, `EthCall`, `EthFees`, `EthState`, `Trace`) are implemented for `EthApi`.
- Pending-block helpers provide access to local pending execution context.
- Transaction helpers validate/forward raw transactions and integrate with the pool.
- Fee/history helpers read cached fee data and gas oracle configuration.
- `SyncListener` exposes an awaitable sync-completion future for clients.

## Key APIs (no snippets)
- `EthRpcConverter`, `DevSigner`, `SyncListener`
- `EthBlocks`, `EthCall`, `EthFees`, `EthState`, `EthTransactions`
