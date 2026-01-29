# storage

## Purpose
Defines serialized storage types (headers, tx hashes, receipts, block sizes) and encoding helpers, and
re-exports the concrete on-disk storage backend. This is the bridge between sync processing and
the sharded storage implementation.

Note: Transaction details, withdrawals, and logs are not stored--logs are derived on-demand from
receipts at query time.

## Contents (one hop)
### Subdirectories
- [x] `sharded/` - Schema v2 sharded backend (segments + WAL + bitset + peer cache).

### Files
- `mod.rs` - Stored types, `BlockBundle` assembly struct, config compatibility key, and bincode helpers.
  - **Key items**: `BlockBundle`, `StoredReceipts`, `StoredPeer`, `PeerCacheLoad`, `StorageConfigKey`, `encode_bincode_value()`

## Key APIs (no snippets)
- **Types**: `BlockBundle` (header, tx_hashes, size, receipts), `StoredTxHashes`, `StoredReceipts`, `StoredBlockSize`, `StorageDiskStats`, `StoredPeer`
- **Functions**: `encode_bincode_value()`, `decode_bincode()`, `encode_bincode_compat_value()`, `decode_bincode_compat_value()`, `encode_u64_value()`, `decode_u64()`
- **Backend**: `Storage` - re-export of `sharded::Storage`.

## Relationships
- **Used by**: `node/src/sync/historical/process.rs` (builds `BlockBundle`), `node/src/rpc` (reads bundles/ranges), `node/src/p2p` (peer cache persistence).
- **Depends on**: `reth_*` primitives for `Header` / `Receipt` encoding, `bincode` for serialization, `zstd` sizing constants.
