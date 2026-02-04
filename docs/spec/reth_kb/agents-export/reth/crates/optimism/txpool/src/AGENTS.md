# src

## Purpose
OP transaction pool extensions: pooled transaction wrapper, OP-specific validation, and maintenance for conditional/interop txs.

## Contents (one hop)
### Subdirectories
- [x] `supervisor/` - Supervisor client and helpers for interop transaction validation.

### Files
- `lib.rs` - Crate wiring and public exports for OP txpool.
  - **Key items**: `OpTransactionPool`, `OpTransactionValidator`, `OpPooledTransaction`, `InvalidCrossTx`
- `transaction.rs` - OP pooled transaction wrapper with conditional/interop metadata and DA size helpers.
  - **Key items**: `OpPooledTransaction`, `OpPooledTx`, `estimated_compressed_size()`, `encoded_2718()`
- `validator.rs` - OP transaction validator with L1 gas checks and interop validation.
  - **Key items**: `OpL1BlockInfo`, `OpTransactionValidator`, `update_l1_block_info()`, `validate_one_with_state()`
- `conditional.rs` - Trait helpers for attaching and checking transaction conditionals.
  - **Key items**: `MaybeConditionalTransaction`, `has_exceeded_block_attributes()`
- `interop.rs` - Trait helpers and utilities for interop deadlines.
  - **Key items**: `MaybeInteropTransaction`, `is_valid_interop()`, `is_stale_interop()`
- `estimated_da_size.rs` - Data availability sizing trait.
  - **Key items**: `DataAvailabilitySized`
- `maintain.rs` - Pool maintenance loops for conditional and interop transactions.
  - **Key items**: `maintain_transaction_pool_conditional_future()`, `maintain_transaction_pool_interop_future()`
- `error.rs` - Cross-chain validation errors for the pool.
  - **Key items**: `InvalidCrossTx`

## Key APIs (no snippets)
- `OpTransactionPool`, `OpTransactionValidator`
- `OpPooledTransaction`, `OpPooledTx`
- `SupervisorClient` (from `supervisor/`)
