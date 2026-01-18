# models

## Purpose
Database model types and encoding helpers used by table definitions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Model re-exports and shared `Encode/Decode/Compress/Decompress` impls.
- **Key items**: primitive encode/decode impls, `impl_compression_for_compact!`
- **Interactions**: Bridges model types with `reth_db_api::table` codecs.

### `accounts.rs`
- **Role**: Account-related composite key types.
- **Key items**: `BlockNumberAddress`, `BlockNumberHashedAddress`, `AddressStorageKey`,
  `BlockNumberAddressRange`

### `blocks.rs`
- **Role**: Block-related model types.
- **Key items**: `StoredBlockOmmers`, `HeaderHash`

### `integer_list.rs`
- **Role**: Compressed integer list backed by Roaring bitmaps.
- **Key items**: `IntegerList`, `IntegerListError`
- **Interactions**: Implements `Compress`/`Decompress` and serde.

### `metadata.rs`
- **Role**: Storage configuration metadata models.
- **Key items**: `StorageSettings`
- **Knobs / invariants**: flags for static files and RocksDB usage.

### `sharded_key.rs`
- **Role**: Generic sharded key helper for large datasets.
- **Key items**: `ShardedKey`, `NUM_OF_INDICES_IN_SHARD`

### `storage_sharded_key.rs`
- **Role**: Sharded key for storage (address + slot + block).
- **Key items**: `StorageShardedKey`, `NUM_OF_INDICES_IN_SHARD`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockNumberAddress`, `IntegerList`, `StorageSettings`,
  `ShardedKey`
- **Modules / Packages**: `accounts`, `blocks`, `integer_list`

## Relationships
- **Used by**: `tables` definitions and table codecs.
- **Depends on**: `reth_codecs` for compact encoding and `reth_primitives_traits` types.
