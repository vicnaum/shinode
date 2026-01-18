# src

## Purpose
Foundational traits and helpers for reth primitives: blocks, headers, transactions, receipts, and supporting utilities.

## Contents (one hop)
### Subdirectories
- [x] `block/` - Block traits plus sealed/recovered wrappers and recovery errors.
- [x] `constants/` - Protocol constants and gas unit helpers.
- [x] `header/` - Sealed header wrapper and mutable/test header helpers.
- [x] `transaction/` - Transaction and signed-transaction traits with recovery helpers.

### Files
- `lib.rs` - Crate entrypoint, feature flags, and re-exports of core traits/types.
  - **Key items**: `MaybeSerde`, `MaybeCompact`, `MaybeSerdeBincodeCompat`, `NodePrimitives`
- `account.rs` - Account and bytecode primitives with codec helpers.
  - **Key items**: `Account`, `Bytecode`, `compact_ids`
- `crypto.rs` - Crypto helper re-exports.
  - **Key items**: `crypto` re-exports
- `error.rs` - Generic mismatch error types.
  - **Key items**: `GotExpected`, `GotExpectedBoxed`
- `extended.rs` - Wrapper enum for extending built-in transaction/receipt types.
  - **Key items**: `Extended`
- `log.rs` - (tests) codec compatibility helpers for logs.
  - **Key items**: test-only `Log` wrapper
- `node.rs` - Node primitives trait and type aliases.
  - **Key items**: `NodePrimitives`, `HeaderTy`, `BlockTy`, `TxTy`
- `proofs.rs` - Merkle root calculation helpers.
  - **Key items**: `calculate_transaction_root`, `calculate_receipt_root`, `calculate_withdrawals_root`
- `receipt.rs` - Receipt trait abstraction and gas accounting helper.
  - **Key items**: `Receipt`, `FullReceipt`, `gas_spent_by_transactions()`
- `serde_bincode_compat.rs` - Bincode-compatible serde helpers for consensus types.
  - **Key items**: `SerdeBincodeCompat`, `BincodeReprFor`, `RlpBincode`
- `size.rs` - Heuristic in-memory size trait and implementations.
  - **Key items**: `InMemorySize`
- `storage.rs` - Storage entry types and subkey extraction.
  - **Key items**: `StorageEntry`, `ValueWithSubKey`
- `sync.rs` - Sync primitives re-exports for std/no-std.
  - **Key items**: `LazyLock`, `OnceLock`
- `withdrawal.rs` - (tests) withdrawal codec compatibility helpers.
  - **Key items**: test-only `RethWithdrawal` wrapper

## Key APIs (no snippets)
- `NodePrimitives`, `Block`, `BlockBody`, `SignedTransaction`
- `SealedBlock`, `SealedHeader`, `RecoveredBlock`
- `InMemorySize`, `SerdeBincodeCompat`
