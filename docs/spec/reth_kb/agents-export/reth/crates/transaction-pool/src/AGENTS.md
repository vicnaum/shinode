# src

## Purpose
Transaction pool core implementation: configuration, traits, ordering, validation, and maintenance.

## Contents (one hop)
### Subdirectories
- [x] `blobstore/` - Blob sidecar storage implementations and tracking.
- [x] `pool/` - Internal subpool logic, events, and update helpers.
- [x] `test_utils/` - Mock pools and transaction generators for tests.
- [x] `validate/` - Transaction validation APIs and executors.

### Files
- `lib.rs` - Crate entrypoint and public pool type exports.
  - **Key items**: `Pool`, `EthTransactionPool`
- `batcher.rs` - Batch insertion processor to reduce lock contention.
  - **Key items**: `BatchTxRequest`, `BatchTxProcessor`
- `config.rs` - Pool limits and fee configuration.
  - **Key items**: `PoolConfig`, `SubPoolLimit`, `PriceBumpConfig`, `LocalTransactionConfig`
- `error.rs` - Pool error types and validation error variants.
  - **Key items**: `PoolError`, `PoolErrorKind`, `InvalidPoolTransactionError`
- `identifier.rs` - Sender/transaction identifiers for indexing.
  - **Key items**: `SenderIdentifiers`, `SenderId`, `TransactionId`
- `maintain.rs` - Pool maintenance loop for canonical state updates and backups.
  - **Key items**: `MaintainPoolConfig`, `maintain_transaction_pool_future()`, `TxBackup`
- `metrics.rs` - Pool metrics structs for pool/validation/maintenance.
  - **Key items**: `TxPoolMetrics`, `MaintainPoolMetrics`, `TxPoolValidatorMetrics`
- `noop.rs` - No-op pool implementation for wiring/testing.
  - **Key items**: `NoopTransactionPool`, `NoopInsertError`
- `ordering.rs` - Transaction ordering traits and default ordering.
  - **Key items**: `TransactionOrdering`, `Priority`, `CoinbaseTipOrdering`
- `traits.rs` - Core pool and transaction trait definitions.
  - **Key items**: `TransactionPool`, `PoolTransaction`, `EthPoolTransaction`

## Key APIs (no snippets)
- `Pool`, `TransactionPool`, `PoolTransaction`
- `TransactionOrdering`, `PoolConfig`, `PoolError`
