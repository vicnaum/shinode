# src

## Purpose
Storage access traits and helper types for blocks, state, receipts, and metadata.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports.
- **Key items**: module re-exports and `StorageSettings` gating

### `account.rs`
- **Role**: Account and changeset reader traits.
- **Key items**: `AccountReader`, `AccountExtReader`, `ChangeSetReader`

### `block.rs`
- **Role**: Block reader trait and `BlockSource` enum.
- **Key items**: `BlockReader`, `BlockSource`, `ProviderBlock`

### `block_hash.rs`
- **Role**: Block hash reader trait.
- **Key items**: `BlockHashReader`

### `block_id.rs`
- **Role**: Block id/number conversion traits.
- **Key items**: `BlockNumReader`, `BlockIdReader`

### `block_indices.rs`
- **Role**: Block body index readers.
- **Key items**: `BlockBodyIndicesProvider`

### `block_writer.rs`
- **Role**: Block and execution write traits.
- **Key items**: `BlockWriter`, `BlockExecutionWriter`

### `chain.rs`
- **Role**: Chain storage reader/writer traits and Eth storage impls.
- **Key items**: `BlockBodyReader`, `BlockBodyWriter`, `ChainStorageReader`, `ChainStorageWriter`,
  `EthStorage`

### `chain_info.rs`
- **Role**: Canon chain tracking interface.
- **Key items**: `CanonChainTracker`

### `database_provider.rs`
- **Role**: DB provider trait and helpers for table access.
- **Key items**: `DBProvider`

### `full.rs`
- **Role**: Full RPC provider trait alias.
- **Key items**: `FullRpcProvider`

### `hashing.rs`
- **Role**: Hashing writer trait for account/storage hashing tables.
- **Key items**: `HashingWriter`

### `header.rs`
- **Role**: Header reader trait.
- **Key items**: `HeaderProvider`, `ProviderHeader`

### `header_sync_gap.rs`
- **Role**: Sync gap provider for headers.
- **Key items**: `HeaderSyncGapProvider`

### `history.rs`
- **Role**: History index writer trait.
- **Key items**: `HistoryWriter`

### `macros.rs`
- **Role**: Macro helpers for provider trait delegation.
- **Key items**: `delegate_impls_to_as_ref!`, `delegate_provider_impls!`

### `metadata.rs`
- **Role**: Metadata and storage settings providers.
- **Key items**: `MetadataProvider`, `MetadataWriter`, `StorageSettingsCache`

### `noop.rs`
- **Role**: No-op provider implementations for tests.
- **Key items**: `NoopProvider`

### `primitives.rs`
- **Role**: Node primitives provider trait.
- **Key items**: `NodePrimitivesProvider`

### `prune_checkpoint.rs`
- **Role**: Prune checkpoint read/write traits.
- **Key items**: `PruneCheckpointReader`, `PruneCheckpointWriter`

### `receipts.rs`
- **Role**: Receipt reader traits and id extensions.
- **Key items**: `ReceiptProvider`, `ReceiptProviderIdExt`, `ProviderReceipt`

### `stage_checkpoint.rs`
- **Role**: Stage checkpoint read/write traits.
- **Key items**: `StageCheckpointReader`, `StageCheckpointWriter`

### `state.rs`
- **Role**: State provider and state-reading traits.
- **Key items**: `StateProvider`, `StateReader`, `StateProviderBox`, `HashedPostStateProvider`,
  `BytecodeReader`, `TryIntoHistoricalStateProvider`

### `state_writer.rs`
- **Role**: State write traits and configuration.
- **Key items**: `StateWriter`, `StateWriteConfig`

### `stats.rs`
- **Role**: Provider statistics trait.
- **Key items**: `StatsReader`

### `storage.rs`
- **Role**: Storage readers and changeset readers.
- **Key items**: `StorageReader`, `StorageChangeSetReader`

### `transactions.rs`
- **Role**: Transaction provider traits.
- **Key items**: `TransactionsProvider`, `TransactionsProviderExt`, `TransactionVariant`

### `trie.rs`
- **Role**: Trie-related read/proof and write traits.
- **Key items**: `StateRootProvider`, `StorageRootProvider`, `StateProofProvider`, `TrieWriter`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockReader`, `StateProvider`, `ReceiptProvider`, `StateWriter`
