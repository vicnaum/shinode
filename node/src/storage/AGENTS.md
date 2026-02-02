# storage

## Purpose

Defines serializable storage types (headers, transactions, receipts, block sizes) and bincode
encoding/decoding helpers, and re-exports the concrete sharded on-disk storage backend. This is the
bridge between sync processing and the sharded storage implementation.

Note: Withdrawals and logs are not stored -- logs are derived on-demand from receipts at query time.

## Contents (one hop)

### Subdirectories
- [x] `sharded/` - Sharded static-file backend (schema v2) with WAL-based writes, per-shard bitset tracking, atomic three-phase compaction, LRU-cached reads, and crash recovery. See `sharded/AGENTS.md`.

### Files
- `mod.rs` - Storage types and bincode serialization helpers (~170 lines). Defines all data containers and re-exports `Storage`/`StorageAggregateStats` from `sharded`.
  - **Key items**: `StoredTxHashes`, `StoredTransaction`, `StoredTransactions`, `StoredBlockSize`, `StoredReceipts`, `BlockBundle`, `SegmentDiskStats`, `StorageDiskStats`, `StoredPeer`, `PeerCacheLoad`, `StorageConfigKey`, `ZSTD_DICT_MAX_SIZE`
  - **Key items** (encoding): `encode_bincode_value`, `decode_bincode`, `encode_bincode_compat_value`, `decode_bincode_compat_value`, `encode_u64_value`, `decode_u64`

## Key APIs (no snippets)

- **Types**: `Storage` (re-exported from sharded), `StorageAggregateStats` (re-exported), `BlockBundle`, `StoredReceipts`, `StoredTxHashes`, `StoredTransaction`, `StoredTransactions`, `StoredBlockSize`, `StoredPeer`, `StorageDiskStats`, `SegmentDiskStats`, `StorageConfigKey`, `PeerCacheLoad`
- **Constants**: `ZSTD_DICT_MAX_SIZE`
- **Functions**: `encode_bincode_value()`, `decode_bincode()`, `encode_bincode_compat_value()`, `decode_bincode_compat_value()`, `encode_u64_value()`, `decode_u64()`

## Relationships

- **Depends on**: `reth_primitives_traits::Header`, `reth_ethereum_primitives::Receipt`, `bincode`, `serde`, `alloy_primitives`
- **Used by**: All subsystems -- `sync/historical` (builds and writes `BlockBundle`s), `rpc` (reads headers/receipts/tx_hashes, derives logs on-demand), `p2p` (peer cache persistence), `run` (storage init/repair/stats), `logging` (disk stats for reports)
- **Data/control flow**:
  1. `mod.rs` defines serializable data containers and encoding helpers
  2. `sharded/` implements the actual read/write/compact/recover operations
  3. Sync pipeline writes `BlockBundle`s via `Storage::write_block_bundles_wal()`
  4. RPC reads blocks via `Storage::block_header()`/`block_receipts()` and derives logs from receipts
  5. Peer cache managed via `Storage::upsert_peer()`/`load_peers()`/`flush_peer_cache()`
