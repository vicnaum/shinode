# sharded

## Purpose
Schema v2 on-disk storage backend. Stores block data in fixed-size shards using a persistent
presence bitset, a staging WAL for out-of-order writes, and compacted "sorted" segment files
(NippyJar-based) for efficient reads. Also persists a peer cache used by the P2P layer.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `bitset.rs` - Persistent bitset used to track which block offsets exist in a shard.
  - **Key items**: `Bitset::load()`, `Bitset::flush()`, `Bitset::set()`, `Bitset::clear()`, `Bitset::bytes()`
- `hash.rs` - Computes a stable content hash for a sealed shard based on metadata + sorted segment files.
  - **Key items**: `compute_shard_hash()`, `to_hex()`, `Sha256`
- `mod.rs` - `Storage` implementation: meta loading, WAL writes, compaction, sealing, rollback, and read APIs.
  - **Key items**: `Storage::open()`, `write_block_bundles_wal()`, `dirty_shards()`, `compact_shard()`, `seal_completed_shards()`, `rollback_to()`
- `nippy_raw.rs` - Raw segment copy utilities for NippyJar (used during compaction).
  - **Key items**: `NippyJarConfig`, `SegmentRawSource::row_bytes()`, `SegmentRawWriter::create()`, `SegmentRawWriter::push_bytes()`
- `wal.rs` - WAL record format + indexing, including mmap-based slice parsing for compaction fast-paths.
  - **Key items**: `WalRecord`, `append_records()`, `build_index()`, `WalSliceIndex`, `build_slice_index()`

## Key APIs (no snippets)
- **Types**: `Storage`, `DirtyShardInfo`, `StorageDiskStats`, `MetaState`, `ShardMeta`, `Bitset`, `WalRecord`, `WalSliceIndex`
- **Functions**: `Storage::open()`, `Storage::disk_usage_at()`, `Storage::write_block_bundles_wal()`, `Storage::dirty_shards()`, `Storage::compact_shard()`, `Storage::missing_ranges_in_range()`

## Relationships
- **Used by**: `node/src/storage/mod.rs` (`pub use sharded::Storage`), `node/src/rpc` (read APIs), `node/src/sync/historical/db_writer.rs` (write + compaction).
- **Depends on**: `reth_nippy_jar` (segment format), `zstd` (compressed columns), `memmap2` (WAL slice indexing), filesystem layout under `NodeConfig.data_dir`.

## Files (detailed)

### `mod.rs`
- **Role**: Implements `Storage` with crash-safe writes: store bundles into a per-shard WAL, track presence via bitsets, compact WAL into sorted segment files, and expose read/rollback APIs.
- **Key items**: `SCHEMA_VERSION`, `META_FILE_NAME`, `PEER_FILE_NAME`, `WAL_NAME`, `DirtyShardInfo`, `Storage::dirty_shards()`, `Storage::missing_ranges_in_range()`
- **Interactions**:
  - Uses `wal::append_records()` for staging writes and `wal::build_slice_index()` to avoid copying during compaction.
  - Uses `nippy_raw::{SegmentRawSource, SegmentRawWriter}` to rebuild segment files via raw byte copy.
  - Uses `hash::compute_shard_hash()` when sealing complete shards.
  - Reports dirty shards (WAL bytes + sorted state) for `db_writer` finalization logging.
  - `write_block_bundle_follow()` expects in-order appends; the follow DB writer enforces ordering via a reorder buffer.
- **Knobs / invariants**:
  - `MetaState` must match `NodeConfig` (`chain_id`, `StorageConfigKey`, `shard_size`); mismatches require rebuilding the data dir.
  - WAL payload format must match `WalBundleRecord` (shared with `wal.rs` slice parser).
  - Compaction swaps `sorted.tmp` -> `sorted/` atomically and keeps `sorted.old` as a recovery safety net.

### `bitset.rs`
- **Role**: Tracks per-shard block presence with a fixed-size byte vector persisted as `present.bitset`.
- **Key items**: `Bitset { bytes, size_bits }`, `is_set()`, `set()`, `clear()`, `count_ones()`
- **Interactions**: Used by `Storage::has_block()` / `missing_blocks_in_range()` and updated during WAL writes/rollback.
- **Knobs / invariants**: On-disk size must match `shard_size` bits; mismatches are treated as corruption.

### `wal.rs`
- **Role**: Implements `staging.wal` append-only records with CRC32 validation and optional mmap slice indexing for compaction.
- **Key items**: `WalRecord`, `WalIndexEntry`, `WalBundleSlices`, `WalSliceIndex`, `ByteRange`
- **Interactions**: `Storage::write_block_bundles_wal()` writes records; `Storage::compact_shard()` uses `build_slice_index()` to locate per-field byte ranges.
- **Knobs / invariants**: CRC is computed over `(block_number, payload_len, payload_bytes)`; invalid/partial tails are truncated or ignored.

### `nippy_raw.rs`
- **Role**: Provides a minimal, serializable mirror of NippyJar config plus low-level readers/writers for single-column jars.
- **Key items**: `NippyJarConfig<H>`, `load_config()`, `SegmentRawSource::open()`, `SegmentRawWriter::finish()`
- **Interactions**: Compaction carries over existing segment bytes and writes a new `.conf` and `.off` offsets file.
- **Knobs / invariants**: Assumes `columns == 1` and that offset file format starts with an 8-byte offset size marker.

### `hash.rs`
- **Role**: Computes a deterministic shard "content hash" from shard metadata, the presence bitset, and sorted segment file contents.
- **Key items**: `compute_shard_hash()`, `Sha256`, stable file ordering by filename
- **Interactions**: Called by `Storage::seal_shard_locked()` to populate `ShardMeta.content_hash`.
- **Knobs / invariants**: Hash input includes shard_start/shard_size/tail_block; changes to layout require bumping the version header string.

## End-to-end flow (high level)
- Open: load `meta.json`, validate against `NodeConfig`, discover shard dirs, load `shard.json` + `present.bitset`, and repair any shard WAL state.
- Write (fast sync): serialize block bundles into per-shard `staging.wal` records and mark offsets present in the bitset.
- Compact: build a WAL slice index (mmap), merge existing sorted segments with WAL rows, and atomically swap in rebuilt segment files.
- Seal: when a shard becomes complete and sorted, compute and persist a content hash in `shard.json`.
- Read: check the bitset, then read rows from sorted segments (headers/tx hashes/receipts/sizes) by block number.
- Rollback: prune sorted segments, clear bitset entries above an ancestor, and recompute `max_present_block` for the store.
- Peer cache: load/update/persist `peers.json` for the P2P static peer seeding path.
