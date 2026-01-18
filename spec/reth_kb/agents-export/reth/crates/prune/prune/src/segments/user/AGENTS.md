# user

## Purpose
User-configured prune segments for receipts, history, senders, and transaction lookup.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for user prune segments.
- **Key items**: `AccountHistory`, `StorageHistory`, `Receipts`, `ReceiptsByLogs`, `SenderRecovery`, `TransactionLookup`, `Bodies`

### `account_history.rs`
- **Role**: Prunes account history change sets and history indices.
- **Key items**: `AccountHistory`, `ACCOUNT_HISTORY_TABLES_TO_PRUNE`
- **Interactions**: Uses `prune_history_indices` to prune `AccountsHistory` shards.

### `storage_history.rs`
- **Role**: Prunes storage change sets and history indices.
- **Key items**: `StorageHistory`, `STORAGE_HISTORY_TABLES_TO_PRUNE`
- **Interactions**: Uses `prune_history_indices` on `StoragesHistory` shards.

### `history.rs`
- **Role**: Shared helpers for pruning history index shards.
- **Key items**: `PrunedIndices`, `prune_history_indices()`
- **Interactions**: Used by account/storage history segments.

### `bodies.rs`
- **Role**: Prunes transaction bodies stored in static files.
- **Key items**: `Bodies`
- **Interactions**: Delegates to `segments::prune_static_files`.

### `receipts.rs`
- **Role**: User-configured receipts pruning segment wrapper.
- **Key items**: `Receipts`
- **Interactions**: Delegates to `segments::receipts::{prune, save_checkpoint}`.

### `receipts_by_logs.rs`
- **Role**: Prunes receipts while retaining logs from configured addresses.
- **Key items**: `ReceiptsByLogs`
- **Interactions**: Uses `ReceiptsLogPruneConfig` to build block ranges and filters.
- **Knobs / invariants**: Applies `MINIMUM_PRUNING_DISTANCE` to ensure safe retention window.

### `sender_recovery.rs`
- **Role**: Prunes transaction sender recovery table entries.
- **Key items**: `SenderRecovery`

### `transaction_lookup.rs`
- **Role**: Prunes transaction hash lookup table with static-file-aware checkpoints.
- **Key items**: `TransactionLookup`
- **Interactions**: Computes tx hashes in parallel via rayon before pruning.

### `merkle_change_sets.rs`
- **Role**: Prunes trie change set tables for accounts and storages.
- **Key items**: `MerkleChangeSets`
- **Interactions**: Requires `StageId::MerkleChangeSets` before pruning.

## Key APIs (no snippets)
- `AccountHistory`, `StorageHistory`, `Receipts`, `ReceiptsByLogs`
- `SenderRecovery`, `TransactionLookup`, `Bodies`
