# stages

## Purpose
Concrete stage implementations used by the sync pipeline: download, execution, hashing, indexing, pruning, and finish steps.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `headers.rs`
- **Role**: Downloads headers (reverse) and writes them into static files and header indices.
- **Key items**: `HeaderStage`, `write_headers()`, `HeaderSyncGap`
- **Interactions**: Uses `HeaderDownloader` and ETL collectors for hash/headers.
- **Knobs / invariants**: Uses ETL file size; checkpoint updates after gap filled.

### `bodies.rs`
- **Role**: Downloads block bodies and writes ommers, bodies, transactions, and tx blocks.
- **Key items**: `BodyStage`, `ensure_consistency()`
- **Interactions**: Depends on headers; writes to DB/static files via `BlockWriter`.
- **Knobs / invariants**: Requires downloader buffer; handles empty-block shortcut rules.

### `sender_recovery.rs`
- **Role**: Recovers transaction senders and persists them to DB or static files.
- **Key items**: `SenderRecoveryStage`, `BATCH_SIZE`, `WORKER_CHUNK_SIZE`
- **Interactions**: Reads transactions and block body indices; writes senders via `EitherWriter`.
- **Knobs / invariants**: `commit_threshold` controls batch size and checkpoint cadence.

### `tx_lookup.rs`
- **Role**: Builds transaction hash -> transaction number index.
- **Key items**: `TransactionLookupStage`
- **Interactions**: Uses ETL collector; supports RocksDB batch when enabled.
- **Knobs / invariants**: `chunk_size` controls ETL batching; respects prune mode.

### `execution.rs`
- **Role**: Executes blocks, writes state changes, receipts, and changesets.
- **Key items**: `ExecutionStage`, `calculate_gas_used_from_headers()`
- **Interactions**: Uses `ConfigureEvm`, `FullConsensus`, and `StateProviderDatabase`.
- **Knobs / invariants**: `ExecutionStageThresholds` and `external_clean_threshold` tune batching.

### `hashing_account.rs`
- **Role**: Hashes plain account state into `HashedAccounts`.
- **Key items**: `AccountHashingStage`, `SeedOpts`
- **Interactions**: Reads `PlainAccountState` and change sets; uses ETL collector.
- **Knobs / invariants**: `clean_threshold` decides full vs incremental hashing.

### `hashing_storage.rs`
- **Role**: Hashes plain storage state into `HashedStorages`.
- **Key items**: `StorageHashingStage`, `HASHED_ZERO_ADDRESS`
- **Interactions**: Reads `PlainStorageState` and storage changesets.
- **Knobs / invariants**: `clean_threshold` decides full vs incremental hashing.

### `merkle.rs`
- **Role**: Computes state roots and intermediate trie hashes from hashed state.
- **Key items**: `MerkleStage`, `MERKLE_STAGE_DEFAULT_REBUILD_THRESHOLD`, `MERKLE_STAGE_DEFAULT_INCREMENTAL_THRESHOLD`
- **Interactions**: Uses `MerkleCheckpoint` and trie DB updates.
- **Knobs / invariants**: Execution vs unwind modes must be ordered correctly.

### `merkle_changesets.rs`
- **Role**: Maintains trie changesets from finalized block to latest block.
- **Key items**: `MerkleChangeSets`
- **Interactions**: Computes `TrieUpdates` and validates state roots.
- **Knobs / invariants**: Retains changesets based on finalized block or retention window.

### `index_account_history.rs`
- **Role**: Builds `AccountsHistory` shard indices from account change sets.
- **Key items**: `IndexAccountHistoryStage`
- **Interactions**: Uses ETL and `collect_account_history_indices()`.
- **Knobs / invariants**: `commit_threshold` and `prune_mode` control range and pruning.

### `index_storage_history.rs`
- **Role**: Builds `StoragesHistory` shard indices from storage change sets.
- **Key items**: `IndexStorageHistoryStage`
- **Interactions**: Uses `collect_history_indices()` and storage sharded keys.
- **Knobs / invariants**: `commit_threshold` and `prune_mode` control range and pruning.

### `prune.rs`
- **Role**: Runs pruning segments for configured prune modes.
- **Key items**: `PruneStage`, `PruneSenderRecoveryStage`
- **Interactions**: Uses `PrunerBuilder` and prune checkpoints.
- **Knobs / invariants**: `commit_threshold` caps deletions before commit.

### `finish.rs`
- **Role**: Marks the pipeline as fully synced at the target block.
- **Key items**: `FinishStage`

### `era.rs`
- **Role**: Imports pre-merge history from ERA1 files or URLs into storage.
- **Key items**: `EraStage`, `EraImportSource`
- **Interactions**: Streams `Era1Reader` items and builds header indices.
- **Knobs / invariants**: Optional import source; uses ETL for hash indexing.

### `utils.rs`
- **Role**: Shared helpers for history index collection/loading.
- **Key items**: `collect_history_indices()`, `collect_account_history_indices()`, `load_history_indices()`
- **Knobs / invariants**: `DEFAULT_CACHE_THRESHOLD` controls cache flush cadence.

## End-to-end flow (high level)
- Optionally import pre-merge headers/bodies with `EraStage`.
- Download headers (`HeaderStage`) and bodies (`BodyStage`) from the network.
- Recover senders (`SenderRecoveryStage`) and execute blocks (`ExecutionStage`).
- Hash accounts and storage, then compute state roots (`MerkleStage`).
- Build transaction lookup and history indices (`TransactionLookupStage`, `Index*HistoryStage`).
- Prune configured segments (`PruneStage`/`PruneSenderRecoveryStage`).
- Mark completion with `FinishStage`.

## Key APIs (no snippets)
- `HeaderStage`, `BodyStage`, `ExecutionStage`, `MerkleStage`
- `SenderRecoveryStage`, `TransactionLookupStage`, `PruneStage`
- `AccountHashingStage`, `StorageHashingStage`, `IndexAccountHistoryStage`
