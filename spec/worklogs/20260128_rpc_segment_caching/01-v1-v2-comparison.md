# Storage Implementation Comparison: v1 vs v2

## Overview

This document compares the storage implementations between:
- **v1**: Commit `11721f1` (Jan 21, 2026) - Schema version 1, FAST
- **v2**: Commit `4e59cbc` onwards (Jan 22, 2026) - Schema version 2, SLOW

Files compared:
- v1: `old-v1/node/src/storage/mod.rs` (1518 lines)
- v2: `node/src/storage/sharded/mod.rs` (~1700 lines)

---

## Architecture Comparison

### v1 Architecture: Single Global Segment Store

```
data/
├── meta.json
├── peers.json
└── static/
    ├── headers.nj          # Single file for ALL headers
    ├── tx_hashes.nj        # Single file for ALL tx_hashes
    ├── tx_meta.nj          # Single file for ALL tx_meta
    ├── receipts.nj         # Single file for ALL receipts
    └── block_sizes.nj      # Single file for ALL sizes
```

**Key struct:**
```rust
struct SegmentStore {
    headers: SegmentWriter,     // Opened once at startup
    tx_hashes: SegmentWriter,   // Opened once at startup
    tx_meta: SegmentWriter,     // Opened once at startup
    receipts: SegmentWriter,    // Opened once at startup
    sizes: SegmentWriter,       // Opened once at startup
}
```

### v2 Architecture: Sharded Segment Store

```
data/
├── meta.json
├── peers.json
└── static/
    └── shards/
        ├── 24320000/           # Shard for blocks 24320000-24320099
        │   ├── meta.json
        │   ├── bitset.bin
        │   └── sorted/
        │       ├── headers.nj
        │       ├── tx_hashes.nj
        │       ├── tx_meta.nj
        │       ├── receipts.nj
        │       └── block_sizes.nj
        ├── 24320100/           # Shard for blocks 24320100-24320199
        │   └── ...
        └── ...
```

**Key struct:**
```rust
// Shards are stored in a HashMap, but segments are NOT cached
pub struct Storage {
    shards: Mutex<HashMap<u64, Arc<Mutex<ShardState>>>>,
    // Note: ShardState only contains meta + bitset, NOT segment readers!
}
```

---

## File Handle Management

### v1: Segments Opened Once at Startup

**Location:** `old-v1/node/src/storage/mod.rs:538-552`

```rust
// In Storage::open()
let segments = SegmentStore {
    headers: SegmentWriter::open(static_dir.join("headers"), start_block, ...)?,
    tx_hashes: SegmentWriter::open(static_dir.join("tx_hashes"), start_block, ...)?,
    tx_meta: SegmentWriter::open(static_dir.join("tx_meta"), start_block, ...)?,
    receipts: SegmentWriter::open(static_dir.join("receipts"), start_block, ...)?,
    sizes: SegmentWriter::open(static_dir.join("block_sizes"), start_block, ...)?,
};
```

These segment writers are stored in the Storage struct and reused for all operations.

### v2: Segments Opened on EVERY Read

**Location:** `node/src/storage/sharded/mod.rs:1334-1355`

```rust
fn with_readers_for_present_block<T>(&self, number: u64, f: impl FnOnce(&ShardSegments) -> Result<Option<T>>) -> Result<Option<T>> {
    let shard_start = shard_start(number, self.shard_size());
    let shard = self.get_shard(shard_start)?;
    // ...
    let segments = shard_segment_readers(&sorted_dir, shard_start)?;  // OPENS 5 FILES!
    f(&segments)
}
```

**Location:** `node/src/storage/sharded/mod.rs:1580-1614`

```rust
fn shard_segment_writers(sorted_dir: &Path, shard_start: u64) -> Result<ShardSegments> {
    Ok(ShardSegments {
        headers: SegmentWriter::open(...)?,      // File open
        tx_hashes: SegmentWriter::open(...)?,    // File open
        tx_meta: SegmentWriter::open(...)?,      // File open
        receipts: SegmentWriter::open(...)?,     // File open
        sizes: SegmentWriter::open(...)?,        // File open
    })
}
```

---

## Range Read Implementation

### v1: Batch Read with Single File Open

**Location:** `old-v1/node/src/storage/mod.rs:754-756`

```rust
pub fn block_headers_range(&self, range: RangeInclusive<u64>) -> Result<Vec<(u64, Header)>> {
    self.segments.headers.read_range_compat(range)
}
```

**Location:** `old-v1/node/src/storage/mod.rs:449-475`

```rust
fn read_range_compat<T>(&self, range: RangeInclusive<u64>) -> Result<Vec<(u64, T)>> {
    let jar = NippyJar::<SegmentHeader>::load(&self.path)?;  // Open ONCE
    let mut cursor = NippyJarCursor::new(&jar)?;              // Cursor ONCE
    let mut out = Vec::new();
    for block in range {                                       // Loop reuses cursor
        // ... read using cursor
        let Some(row_vals) = cursor.row_by_number(row)? else { continue };
        out.push((block, decoded));
    }
    Ok(out)
}
```

### v2: Per-Block Read with Repeated File Opens

**Location:** `node/src/storage/sharded/mod.rs:1380-1391`

```rust
pub fn block_headers_range(&self, range: std::ops::RangeInclusive<u64>) -> Result<Vec<(u64, Header)>> {
    let mut out = Vec::new();
    for block in range {
        if let Some(header) = self.block_header(block)? {  // Each opens 5 files!
            out.push((block, header));
        }
    }
    Ok(out)
}
```

---

## Performance Impact Analysis

### For a 21-block eth_getLogs query (blocks 24328130-24328151):

#### v1 (FAST):
| Operation | File Opens | Notes |
|-----------|------------|-------|
| `block_headers_range(21)` | 1 | Single read_range call |
| `block_tx_hashes_range(21)` | 1 | Single read_range call |
| `block_receipts_range(21)` | 1 | Single read_range call |
| **TOTAL** | **3** | |

#### v2 (SLOW):
| Operation | File Opens | Notes |
|-----------|------------|-------|
| `has_block` check × 21 | 0 | Just bitset lookups (fast) |
| `block_headers_range(21)` | 21 × 5 = 105 | Each block opens 5 segment files |
| `block_tx_hashes_range(21)` | 21 × 5 = 105 | Each block opens 5 segment files |
| `block_receipts_range(21)` | 21 × 5 = 105 | Each block opens 5 segment files |
| **TOTAL** | **315** | |

**Result: v2 does 105x more file operations than v1 for the same query!**

---

## Additional v2 Overhead: Pre-validation Loop

**Location:** `node/src/rpc/mod.rs:288-293` (added in 4e59cbc)

```rust
for block in from_block..=to_block {
    if !ctx.storage.has_block(block).map_err(internal_error)? {
        return Err(missing_block_error(block));
    }
}
```

This adds 21 additional calls to `has_block()` before any data is read. While `has_block()` is relatively fast (just bitset lookup), it still adds overhead.

---

## Root Cause Summary

1. **No segment caching**: v2 opens segment files on every read, never caches them
2. **No batch reads**: v2's `*_range()` functions iterate and call per-block reads
3. **5x multiplier**: Each block read opens ALL 5 segment types
4. **Pre-validation**: Additional loop checking block existence before reading

---

## Proposed Fixes

### Option 1: Cache Segment Readers per Shard (Recommended)

Add segment readers to `ShardState`:

```rust
struct ShardState {
    dir: PathBuf,
    meta: ShardMeta,
    bitset: Bitset,
    segments: Option<ShardSegments>,  // NEW: Cached segment readers
}
```

Lazily initialize on first read, reuse for subsequent reads.

**Pros:**
- Minimal code change
- Preserves sharded architecture benefits
- Segment readers cached per shard

**Cons:**
- Need to handle segment invalidation on writes/compaction
- Memory usage increases (one set of readers per active shard)

### Option 2: Implement True Batch Reads

Add `block_headers_range_batched()` that:
1. Groups blocks by shard
2. Opens each shard's segments once
3. Reads all blocks in that shard
4. Closes and moves to next shard

```rust
pub fn block_headers_range_batched(&self, range: RangeInclusive<u64>) -> Result<Vec<(u64, Header)>> {
    let mut out = Vec::new();
    let mut current_shard: Option<(u64, ShardSegments)> = None;

    for block in range {
        let shard_start = shard_start(block, self.shard_size());

        // Only open new segments when crossing shard boundary
        if current_shard.as_ref().map(|(s, _)| *s) != Some(shard_start) {
            let segments = shard_segment_readers(...)?;
            current_shard = Some((shard_start, segments));
        }

        let (_, segments) = current_shard.as_ref().unwrap();
        if let Some(header) = segments.headers.read_row_compat(block)? {
            out.push((block, header));
        }
    }
    Ok(out)
}
```

**Pros:**
- No caching, simpler lifecycle
- Only opens segments for shards actually accessed

**Cons:**
- More invasive code change
- Need to implement for each data type

### Option 3: Hybrid (Cache + Batch)

Implement both: cache at shard level + batch read functions.

---

## Files to Modify for Fix

1. `node/src/storage/sharded/mod.rs`:
   - Add segment caching to `ShardState`
   - Modify `with_readers_for_present_block()` to use cached readers
   - Or add new `*_range_batched()` functions

2. `node/src/rpc/mod.rs`:
   - Consider removing or optimizing the pre-validation loop
   - Or use `missing_blocks_in_range()` which is more efficient

---

## Test Plan

After implementing fix:

1. Run benchmark script:
   ```bash
   ./rpc-benchmark/bench.sh --no-node --no-build
   ```

2. Expected improvement:
   - Before: ~6000ms for 21 blocks
   - After: <100ms for 21 blocks (similar to v1)

3. Verify no regressions:
   - Run full test suite
   - Test write operations still work
   - Test compaction still works
   - Test across shard boundaries

---

## References

- v1 code: `./old-v1/node/src/storage/mod.rs`
- v2 code: `./node/src/storage/sharded/mod.rs`
- RPC handler: `./node/src/rpc/mod.rs`
- Benchmark script: `./rpc-benchmark/bench.sh`
