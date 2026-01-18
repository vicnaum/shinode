# src

## Purpose
Types, configs, caches, and helpers for implementing `eth` RPC server behavior.

## Contents (one hop)
### Subdirectories
- [x] `builder/` - `eth` namespace configuration types.
- [x] `cache/` - Async caching layer for blocks/receipts/headers.
- [x] `error/` - `eth` RPC error types and conversion helpers.

### Files
- `lib.rs` - Module wiring and public re-exports.
  - **Key items**: `EthConfig`, `EthStateCache`, `EthApiError`, `FeeHistoryCache`, `GasPriceOracle`
- `block.rs` - Block + receipts helper types and conversions.
  - **Key items**: `BlockAndReceipts`, `convert_transaction_receipt()`
- `fee_history.rs` - Fee history cache and entry types.
  - **Key items**: `FeeHistoryCache`, `FeeHistoryCacheConfig`, `FeeHistoryEntry`
- `gas_oracle.rs` - Gas price oracle implementation and config.
  - **Key items**: `GasPriceOracle`, `GasPriceOracleConfig`, `GasPriceOracleResult`, `GasCap`
- `id_provider.rs` - Subscription ID provider for `eth` RPC.
  - **Key items**: `EthSubscriptionIdProvider`
- `logs_utils.rs` - Log filtering helpers for `eth_getLogs`.
  - **Key items**: `ProviderOrBlock`, `matching_block_logs_with_tx_hashes()`, `FilterBlockRangeError`
- `pending_block.rs` - Pending block structures and helpers.
  - **Key items**: `PendingBlockEnv`, `PendingBlockEnvOrigin`, `PendingBlock`, `PendingBlockAndReceipts`
- `receipt.rs` - Receipt conversion helpers.
  - **Key items**: `EthReceiptConverter`
- `simulate.rs` - Simulation error types for `eth_simulate`-style calls.
  - **Key items**: `EthSimulateError`
- `transaction.rs` - Transaction source and conversion helpers.
  - **Key items**: `TransactionSource`
- `tx_forward.rs` - Raw transaction forwarder configuration.
  - **Key items**: `ForwardConfig`
- `utils.rs` - Misc utilities used across eth RPC helpers.
  - **Key items**: `checked_blob_gas_used_ratio()`, `calculate_gas_used_and_next_log_index()`

## Key APIs (no snippets)
- `EthConfig`, `EthStateCache`, `EthApiError`
- `FeeHistoryCache`, `GasPriceOracle`, `PendingBlock`
