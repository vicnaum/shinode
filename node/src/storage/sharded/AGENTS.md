# sharded

## Purpose

Sharded static-file storage backend (schema v2) for Ethereum history data. Divides the chain into fixed-size shards, each containing headers, transaction hashes, transaction metadata, receipts, and block sizes in compressed nippy-jar segments. Provides write-ahead logging (WAL) for out-of-order ingestion, atomic multi-phase compaction, and LRU-cached read-only access with crash recovery.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `bitset.rs` - Persistent bitset used to track which block offsets exist in a shard.
  - **Key items**: `Bitset::load()`, `Bitset::flush()`, `Bitset::set()`, `Bitset::clear()`, `Bitset::count_ones()`
- `hash.rs` - Computes a stable content hash for a sealed shard based on metadata + sorted segment files.
  - **Key items**: `compute_shard_hash()`, `to_hex()`, `Sha256`
- `mod.rs` - `Storage` implementation: meta loading, WAL writes, compaction, sealing, rollback, read APIs, aggregate stats, and repair.
  - **Key items**: `Storage::open()`, `write_block_bundles_wal()`, `write_block_bundle_follow()`, `dirty_shards()`, `compact_shard()`, `seal_completed_shards()`, `seal_completed_shards_with_progress()`, `rollback_to()`, `aggregate_stats()`, `repair()`
- `nippy_raw.rs` - Raw segment copy utilities for NippyJar (used during compaction).
  - **Key items**: `NippyJarConfig`, `SegmentRawSource::row_bytes()`, `SegmentRawWriter::create()`, `SegmentRawWriter::push_bytes()`
- `wal.rs` - WAL record format + indexing, including mmap-based slice parsing for compaction fast-paths.
  - **Key items**: `WalRecord`, `append_records()`, `build_index()`, `WalSliceIndex`, `build_slice_index()`

## Key APIs (no snippets)
- **Types**: `Storage` (main public interface), `ShardState` (in-memory shard), `ShardMeta` (persisted JSON metadata), `MetaState` (global storage metadata), `StorageAggregateStats` (aggregated counters), `DirtyShardInfo`, `SegmentReader` / `ShardSegmentReaders` (read-only, no exclusive lock), `WalRecord`, `WalBundleSlices`, `WalSliceIndex`, `Bitset`, `NippyJarConfig`, `ShardRecoveryResult`, `RepairReport`
- **Functions**: `Storage::open`, `Storage::write_block_bundles_wal`, `Storage::write_block_bundle_follow`, `Storage::compact_shard`, `Storage::compact_all_dirty_with_progress`, `Storage::dirty_shards`, `Storage::block_header`/`tx_hashes`/`receipts`, `Storage::missing_blocks_in_range`, `Storage::missing_ranges_in_range`, `Storage::rollback_to`, `Storage::seal_completed_shards_with_progress`, `Storage::aggregate_stats`, `Storage::repair`, `Storage::disk_usage_at`, `recover_shard`

## Relationships
- **Depends on**: `reth_nippy_jar` (segment read/write), `reth_primitives_traits::Header`, `zstd` (compression), `bincode` (payload encoding), `parking_lot::Mutex`, `lru::LruCache` (20-shard capacity), `crc32fast` (WAL integrity), `memmap2` (WAL slice indexing), filesystem layout under `NodeConfig.data_dir`
- **Used by**: `node/src/storage/mod.rs` (`pub use sharded::Storage`), `node/src/rpc` (read APIs), `node/src/sync/historical/db_writer.rs` (write + compaction)
- **Data/control flow**:
  1. P2P fetches blocks -> sync packages into BlockBundles
  2. `write_block_bundles_wal` stages each bundle to per-shard WAL (compressed tx_meta/receipts)
  3. `compact_shard` (three-phase): reads WAL slices + old segments -> writes to temp -> atomically swaps
  4. Block reads: check bitset (fast path) -> load shard from segment_cache -> decode from nippy-jar
  5. Bitset rebuilt during compaction to ensure consistency with sorted segments
  6. Rollback prunes segments and clears bitset bits
  7. Sealing computes content hash after compaction completes

## ShardMeta Enhancements
`ShardMeta` now tracks comprehensive statistics to avoid expensive recomputation:
- **`total_transactions: u64`** - Accumulated transaction count (recomputed authoritatively during compaction)
- **`total_receipts: u64`** - Accumulated receipt count (recomputed during compaction)
- **`total_logs: u64`** - Total log count (accumulated during WAL writes; preserved, not recomputed, during compaction because receipts are zstd-compressed)
- **`disk_bytes_headers/transactions/receipts/total: u64`** - On-disk bytes per segment type

## StorageAggregateStats
Cheap aggregation across all shards via `Storage::aggregate_stats()` (no disk I/O):
- `total_blocks`, `total_transactions`, `total_receipts`, `total_logs`
- `total_shards`, `compacted_shards`
- `disk_bytes_headers`, `disk_bytes_transactions`, `disk_bytes_receipts`, `disk_bytes_total`

## Files (detailed)

### `mod.rs`
- **Role**: Core Storage struct and compaction logic (~2470 lines). Implements open/read/write/compact/recovery for the sharded backend.
- **Key constants**: `SCHEMA_VERSION` (2), `META_FILE_NAME`, `PEER_FILE_NAME`, `WAL_NAME`, `SEGMENT_CACHE_SIZE` (20)
- **Key types**: `Storage`, `ShardState`, `ShardMeta`, `MetaState`, `WalBundleRecord`, `SegmentWriter`, `SegmentReader`, `ShardSegmentReaders`, `StorageAggregateStats`, `DirtyShardInfo`, `ShardRecoveryResult`, `RepairReport`
- **Interactions**:
  - Uses `wal::append_records()` for staging writes and `wal::build_slice_index()` to avoid copying during compaction.
  - Uses `nippy_raw::{SegmentRawSource, SegmentRawWriter}` to rebuild segment files via raw byte copy.
  - Uses `hash::compute_shard_hash()` when sealing complete shards.
  - Reports dirty shards (WAL bytes + sorted state) for `db_writer` finalization logging.
  - `write_block_bundle_follow()` expects in-order appends; the follow DB writer enforces ordering via a reorder buffer.
  - Segment cache invalidation on compaction, rollback, and follow writes.
  - Exports `Storage` and `StorageAggregateStats` to parent module.
- **Knobs / invariants**:
  - `SCHEMA_VERSION = 2`; mismatch on open triggers error
  - `SEGMENT_CACHE_SIZE = 20` LRU shard reader sets
  - `MetaState` must match `NodeConfig` (`chain_id`, `StorageConfigKey`, `shard_size`); mismatches require rebuilding the data dir.
  - Shard layout: `data_dir/static/shards/<shard_start>/{shard.json, present.bitset, state/staging.wal, sorted/{headers,tx_hashes,tx_meta,receipts,block_sizes}}`
  - Compression: headers/tx_hashes/block_sizes uncompressed; tx_meta/receipts use zstd (max_dict_size=5000)
  - WAL payload format must match `WalBundleRecord` (shared with `wal.rs` slice parser).
  - Compaction swaps `sorted.tmp` -> `sorted/` atomically and keeps `sorted.old` as a recovery safety net.
  - `total_logs` accumulated during WAL writes; preserved (not recomputed) during compaction.
  - Bitset only reflects sorted segment data, NOT WAL writes
  - Atomic swap: sorted -> sorted.old, sorted.tmp -> sorted; same for bitset

### `wal.rs`
- **Role**: Write-ahead log for out-of-order block writes (~410 lines). Append, index rebuild, and zero-copy slice parsing.
- **Key items**: `WalRecord`, `WalIndexEntry`, `WalBundleSlices`, `WalSliceIndex`, `ByteRange`, `append_records`, `read_records`, `build_index`, `build_slice_index`
- **Interactions**: `append_records` writes to `staging.wal`; `build_slice_index` memory-maps WAL for zero-copy compaction reads in `compact_shard`.
- **Knobs / invariants**:
  - WAL format: `block_number(u64LE) | payload_len(u32LE) | payload | crc32(u32LE)`
  - CRC32 covers block_number + payload_len + payload bytes (all in little-endian)
  - Partial/corrupted tail truncated on read/index rebuild
  - Memory-mapping is read-only; file handle owned by `WalSliceIndex`

### `bitset.rs`
- **Role**: Dense bitarray for per-shard presence tracking (~128 lines). Tracks which block offsets within a shard are present.
- **Key items**: `Bitset { bytes, size_bits }`, `new`, `load`, `flush`, `is_set`, `set`, `clear`, `count_ones`, `bytes`
- **Interactions**: Two bitsets per shard:
  1. `present.bitset` - tracks blocks in sorted segment (compacted data)
  2. `pending.bitset` - tracks blocks in WAL (pending compaction, for resume support)

  Presence checks (`has_block`, `missing_*`) check BOTH bitsets. On startup, both are loaded. During WAL writes, `pending.bitset` is updated. During compaction, WAL data is merged into sorted segment and `pending.bitset` is deleted. During follow writes, only `present.bitset` is updated (data goes directly to sorted segment).
- **Knobs / invariants**:
  - `size_bits = shard_size`; stored as `ceil(size_bits/8)` bytes
  - Bit indexing: `byte_idx = offset/8`, `bit = 1u8 << (offset%8)`
  - On-disk size must match `shard_size` bits; mismatches are treated as corruption.
  - `present.bitset` reflects sorted segment data; `pending.bitset` reflects WAL data.

### `nippy_raw.rs`
- **Role**: Low-level wrapper for `reth_nippy_jar` segment file handling (~166 lines). Provides a minimal, serializable mirror of NippyJar config plus low-level readers/writers for single-column jars.
- **Key items**: `NippyJarConfig<H>`, `SegmentRawSource`, `SegmentRawWriter`, `load_config`, `open`, `create`, `row_bytes`, `push_bytes`, `push_empty`, `finish`
- **Interactions**: Used during compaction to read existing segments (`SegmentRawSource`) and write new ones (`SegmentRawWriter`). Also used by `SegmentWriter`/`SegmentReader` in `mod.rs`. Compaction carries over existing segment bytes and writes a new `.conf` and `.off` offsets file.
- **Knobs / invariants**:
  - Segment files: `name`, `name.off` (offsets), `name.conf` (config)
  - `OFFSET_SIZE_BYTES = 8` (u64)
  - `push_empty` writes offset only (for gaps); `row_bytes` returns `Ok(None)` for empty rows
  - Assumes `columns == 1` and that offset file format starts with an 8-byte offset size marker.

### `hash.rs`
- **Role**: Compute SHA256 integrity hash for sealed shards (~66 lines).
- **Key items**: `compute_shard_hash()`, `to_hex()`, `Sha256`, stable file ordering by filename
- **Interactions**: Called by `seal_shard_locked()` after compaction completes. Hashes shard metadata + all sorted segment files. Populates `ShardMeta.content_hash`.
- **Knobs / invariants**:
  - Hash prefix: `"stateless-history-shard-v1\n"`
  - Hash input includes shard_start/shard_size/tail_block; changes to layout require bumping the version header string.
  - Files hashed in sorted alphabetical order for determinism
  - Chunked 1MB reads for large files

## Segment Reader Cache

To avoid reopening segment files on every block read, the storage maintains an LRU cache of `ShardSegmentReaders`:

- **Cache size**: 20 shards (configurable via `SEGMENT_CACHE_SIZE`)
- **Key**: `shard_start` block number
- **Value**: `Arc<ShardSegmentReaders>` (5 segment readers: headers, tx_hashes, tx_meta, receipts, sizes)
- **Invalidation**: Cache entries are invalidated on compaction, rollback, or follow writes
- **Read-only design**: `SegmentReader` does NOT open `NippyJarWriter` (no exclusive file lock), allowing multiple concurrent readers with an active writer.

**Performance impact** (10k blocks query):
- Without cache: ~26,000ms (shard-size 100)
- With cache: ~3,900ms (shard-size 1000)
- V1 baseline: ~4,000ms

## Atomic Compaction Model

The storage uses a phase-based atomic compaction model to prevent data corruption:

### Key Invariant (Two-Bitset Model)
The storage uses **two bitsets** per shard for resume support:
- **`present.bitset`**: Tracks blocks in the sorted segment (compacted data). Rebuilt during compaction.
- **`pending.bitset`**: Tracks blocks in WAL (pending compaction). Updated during WAL writes, deleted after compaction.

Presence checks (`has_block`, `missing_ranges_in_range`) check BOTH bitsets. This allows resume to correctly skip blocks that are in WAL but not yet compacted, avoiding re-fetching data that already exists.

### Compaction Phases
Tracked in `shard.json.compaction_phase`:
1. **"writing"**: Creating `sorted.tmp/` segments and `bitset.tmp`
2. **"swapping"**: Atomic renames: `sorted/` -> `sorted.old/`, `sorted.tmp/` -> `sorted/`, then bitset swap
3. **"cleanup"**: Deleting `sorted.old/`, `bitset.old`, and `staging.wal`
4. **null**: Idle, clean state

During compaction, `total_transactions` and `total_receipts` are recomputed authoritatively from actual segment data; `total_logs` is preserved from the previous state (no zstd decompression).

### Recovery on Startup
`recover_shard()` checks `compaction_phase` and performs appropriate recovery:
- **"writing"**: Discard partial `sorted.tmp/` and `bitset.tmp`, reset phase
- **"swapping"**: Restore from backups if needed, reset to pre-swap state
- **"cleanup"**: Complete pending deletes
- **null**: Check for orphan `.tmp`/`.old` files from hard crashes

### Partial Data Detection
During compaction, the code verifies ALL segment files (headers, tx_hashes, tx_meta, receipts, sizes) have data for each row before considering existing data valid. This handles corrupt shards where some segments have data but others don't. Blocks with partial data are skipped with a warning and will be re-downloaded.

### File Layout During Compaction
```
shards/<shard_start>/
+-- shard.json           # metadata + compaction_phase
+-- present.bitset       # live bitset (reflects sorted segment)
+-- pending.bitset       # WAL bitset (reflects staging.wal blocks, for resume)
+-- bitset.tmp           # new bitset during compaction
+-- bitset.old           # backup during swap
+-- state/
|   +-- staging.wal      # pending blocks (tracked in pending.bitset)
+-- sorted/              # live segment files
+-- sorted.tmp/          # new segments during compaction
+-- sorted.old/          # backup during swap
```

## End-to-end flow (high level)
1. **Open**: `Storage::open` loads `meta.json`, validates against `NodeConfig`, discovers shard dirs, loads `shard.json` + `present.bitset` + `pending.bitset` (if exists), runs phase-aware `recover_shard()`, marks `sorted=false` if WAL exists, and initializes LRU cache.
2. **Write (fast sync)**: `write_block_bundles_wal` groups blocks by shard, compresses tx_meta/receipts with zstd, appends CRC32-checksummed WAL records, accumulates log counts, updates `total_logs`. Updates `pending.bitset` (for resume support) but NOT `present.bitset`.
3. **Compact**:
   1. Phase WRITING: Set `compaction_phase="writing"`, persist. Build `sorted.tmp/` segments and `bitset.tmp` from sorted + WAL data (memory-maps WAL via `build_slice_index`, merges WAL + existing segments into new sorted segments). Recompute `total_transactions` and `total_receipts` from actual segment data; preserve `total_logs`.
   2. Phase SWAPPING: Set `compaction_phase="swapping"`, persist. Atomic swaps with `.old` backups: `sorted/` -> `sorted.old/`, `sorted.tmp/` -> `sorted/`; same for bitset.
   3. Phase CLEANUP: Set `compaction_phase="cleanup"`, persist. Delete backups, WAL, and `pending.bitset`. Update `sorted=true`, clear `compaction_phase`.
4. **Follow**: `write_block_bundle_follow` directly appends to sorted segments (head-following mode, already sorted).
5. **Seal**: when a shard becomes complete and sorted, `seal_shard_locked` computes and persists a SHA256 content hash in `shard.json`.
6. **Read**: check the bitset (fast path), then LRU segment cache, then open NippyJar fresh on cache miss. Read rows from cached segment readers (headers/tx hashes/receipts/sizes) by block number.
7. **Rollback**: prune sorted segments, clear bitset entries above an ancestor, and recompute `max_present_block` for the store.
8. **Gap detection**: `missing_ranges_in_range` scans BOTH bitsets (present + pending) in chunks for gap detection, ensuring blocks in WAL are not re-fetched.
9. **Repair**: standalone operation that scans all shards, performs phase-aware recovery, and returns a detailed `RepairReport`.
10. **Aggregate stats**: cheap in-memory aggregation from shard metadata via `aggregate_stats()` (no disk I/O).
11. **Peer cache**: load/update/persist `peers.json` for the P2P static peer seeding path.

## Notes
