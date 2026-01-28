# Implementation Plan: Segment Reader Caching

## Overview

This document describes the recommended fix for the RPC performance regression: adding an LRU cache for segment readers.

## Why This Approach

### v2 Sharding is Good Architecture

The sharded storage design enables:
- **Retention policies** - delete old shards without rewriting everything
- **Parallel compaction** - compact multiple shards concurrently
- **Sparse data** - not all shards need all blocks (useful for partial sync)
- **Bounded file sizes** - each shard is small, easier to manage

The problem is purely implementation - no caching of file handles.

### Why LRU Cache?

- **Recent blocks are hot** - follow mode, RPC queries for "latest"
- **Old blocks are cold** - rarely accessed historical data
- **LRU naturally prioritizes** - keeps hot shards open, evicts cold ones
- **Bounded memory** - configurable limit (e.g., 10-20 shards max)

### Cache Size Math

- Each shard has 5 segment readers (headers, tx_hashes, tx_meta, receipts, sizes)
- With `shard_size=100`, caching 20 shards = 2000 blocks of instant access
- Most RPC queries ask for recent data - this covers them

---

## Architecture

### Current (Slow)

```
Storage
├── shards: HashMap<shard_start, ShardState>
│   └── ShardState { dir, meta, bitset }  // No segment readers!
│
└── Every read:
    └── shard_segment_readers() → Opens 5 files → Read → Close
```

### Proposed (Fast)

```
Storage
├── shards: HashMap<shard_start, ShardState>
│   └── ShardState { dir, meta, bitset }
│
├── segment_cache: LruCache<shard_start, Arc<ShardSegments>>  // NEW
│   └── Caches open segment readers per shard
│
└── Every read:
    └── get_cached_segments() → Cache hit? Return cached : Open + cache → Read
```

---

## Implementation Steps

### Step 1: Add Dependency

**File:** `node/Cargo.toml`

```toml
[dependencies]
lru = "0.12"
```

Alternative options:
- `quick_cache` - concurrent, faster for high contention
- `moka` - async-friendly, TTL support

For sync mutex-based approach, `lru` is simple and sufficient.

---

### Step 2: Add Cache to Storage Struct

**File:** `node/src/storage/sharded/mod.rs`

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct Storage {
    data_dir: PathBuf,
    meta: Mutex<StorageMeta>,
    shards: Mutex<HashMap<u64, Arc<Mutex<ShardState>>>>,
    peer_cache: Mutex<HashMap<String, StoredPeer>>,
    peer_cache_dir: PathBuf,

    // NEW: LRU cache of open segment readers
    segment_cache: Mutex<LruCache<u64, Arc<ShardSegments>>>,
}
```

---

### Step 3: Initialize Cache in Storage::open()

**File:** `node/src/storage/sharded/mod.rs`

```rust
const SEGMENT_CACHE_SIZE: usize = 20;  // Cache up to 20 shards

impl Storage {
    pub fn open(config: &NodeConfig) -> Result<Self> {
        // ... existing code ...

        Ok(Self {
            data_dir: config.data_dir.clone(),
            meta: Mutex::new(meta),
            shards: Mutex::new(shards),
            peer_cache: Mutex::new(peer_cache),
            peer_cache_dir,

            // NEW
            segment_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(SEGMENT_CACHE_SIZE).unwrap()
            )),
        })
    }
}
```

---

### Step 4: Add Cache Access Methods

**File:** `node/src/storage/sharded/mod.rs`

```rust
impl Storage {
    /// Get cached segment readers for a shard, opening if not cached.
    fn get_cached_segments(&self, shard_start: u64) -> Result<Option<Arc<ShardSegments>>> {
        // Check cache first
        {
            let mut cache = self.segment_cache.lock();
            if let Some(segments) = cache.get(&shard_start) {
                return Ok(Some(Arc::clone(segments)));
            }
        }

        // Not in cache - check if shard exists and has sorted segments
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(None);
        };

        let sorted_dir = {
            let state = shard.lock();
            sorted_dir(&state.dir)
        };

        if !sorted_dir.exists() {
            return Ok(None);
        }

        // Open segments and cache them
        let segments = shard_segment_readers(&sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };

        let segments = Arc::new(segments);
        {
            let mut cache = self.segment_cache.lock();
            cache.put(shard_start, Arc::clone(&segments));
        }

        Ok(Some(segments))
    }

    /// Invalidate cached segments for a shard (call before compaction).
    pub fn invalidate_segment_cache(&self, shard_start: u64) {
        let mut cache = self.segment_cache.lock();
        cache.pop(&shard_start);
    }

    /// Clear entire segment cache.
    pub fn clear_segment_cache(&self) {
        let mut cache = self.segment_cache.lock();
        cache.clear();
    }
}
```

---

### Step 5: Update Read Functions to Use Cache

**File:** `node/src/storage/sharded/mod.rs`

**Before:**
```rust
fn with_readers_for_present_block<T>(
    &self,
    number: u64,
    f: impl FnOnce(&ShardSegments) -> Result<Option<T>>,
) -> Result<Option<T>> {
    let shard_start = shard_start(number, self.shard_size());
    let shard = self.get_shard(shard_start)?;
    let Some(shard) = shard else {
        return Ok(None);
    };
    let state = shard.lock();
    let offset = (number - shard_start) as usize;
    if !state.bitset.is_set(offset) {
        return Ok(None);
    }
    let sorted_dir = sorted_dir(&state.dir);
    let segments = shard_segment_readers(&sorted_dir, shard_start)?;  // SLOW!
    let Some(segments) = segments else {
        return Ok(None);
    };
    f(&segments)
}
```

**After:**
```rust
fn with_readers_for_present_block<T>(
    &self,
    number: u64,
    f: impl FnOnce(&ShardSegments) -> Result<Option<T>>,
) -> Result<Option<T>> {
    let shard_start = shard_start(number, self.shard_size());

    // Check if block is present (bitset check)
    let shard = self.get_shard(shard_start)?;
    let Some(shard) = shard else {
        return Ok(None);
    };
    {
        let state = shard.lock();
        let offset = (number - shard_start) as usize;
        if !state.bitset.is_set(offset) {
            return Ok(None);
        }
    }

    // Get cached segments (opens if not cached)
    let segments = self.get_cached_segments(shard_start)?;  // FAST!
    let Some(segments) = segments else {
        return Ok(None);
    };

    f(&segments)
}
```

---

### Step 6: Add Cache Invalidation Before Compaction

**File:** `node/src/storage/sharded/mod.rs`

Find all places where compaction modifies sorted segments and add invalidation:

```rust
// Before starting compaction on a shard
self.invalidate_segment_cache(shard_start);

// Example in compact_shard() or similar:
pub fn compact_shard(&self, shard_start: u64) -> Result<()> {
    // Invalidate cache BEFORE modifying files
    self.invalidate_segment_cache(shard_start);

    // ... existing compaction code ...
}
```

**Places to add invalidation:**
- `compact_shard()` or equivalent
- `seal_shard()` if it modifies segments
- Any function that rewrites sorted segment files

---

### Step 7: Optimize Range Reads (Optional Enhancement)

**File:** `node/src/storage/sharded/mod.rs`

For better range query performance, group blocks by shard:

```rust
pub fn block_headers_range(
    &self,
    range: std::ops::RangeInclusive<u64>,
) -> Result<Vec<(u64, Header)>> {
    let shard_size = self.shard_size();
    let mut out = Vec::new();

    // Group blocks by shard for efficient access
    let start = *range.start();
    let end = *range.end();

    let mut current_shard_start = shard_start(start, shard_size);
    let mut current_segments: Option<Arc<ShardSegments>> = None;

    for block in start..=end {
        let block_shard = shard_start(block, shard_size);

        // Check if we crossed into a new shard
        if current_segments.is_none() || block_shard != current_shard_start {
            current_shard_start = block_shard;

            // Check bitset first
            if !self.has_block(block)? {
                current_segments = None;
                continue;
            }

            current_segments = self.get_cached_segments(block_shard)?;
        }

        if let Some(ref segments) = current_segments {
            if let Some(header) = segments.headers.read_row_compat(block)? {
                out.push((block, header));
            }
        }
    }

    Ok(out)
}
```

Apply similar pattern to `block_tx_hashes_range()` and `block_receipts_range()`.

---

## Cache Invalidation Matrix

| Event | Action | Reason |
|-------|--------|--------|
| WAL write | None | WAL is separate from sorted segments |
| Compaction start | Invalidate shard | About to rewrite segment files |
| Compaction finish | None | Next read will re-cache |
| Shard sealed | Invalidate shard | Content hash computed, files may change |
| Storage close | Clear all | Clean shutdown |

---

## Testing Plan

### 1. Benchmark Test

```bash
# Before fix - baseline
./rpc-benchmark/bench.sh --no-node --no-build
# Expected: ~6000ms

# After fix
cargo build --release --manifest-path node/Cargo.toml
./rpc-benchmark/bench.sh
# Expected: <100ms
```

### 2. Unit Tests

Add tests for:
- Cache hit returns same Arc
- Cache miss opens segments
- Cache eviction when full
- Invalidation removes entry
- Concurrent access safety

### 3. Integration Tests

- Read after write (cache doesn't serve stale data)
- Read during compaction (invalidation works)
- Cross-shard range queries
- Cache behavior with many shards

### 4. Stress Tests

- Many concurrent RPC requests
- Large range queries
- Rapid shard access patterns

---

## Expected Results

| Metric | Before | After |
|--------|--------|-------|
| File opens (21-block query) | 315 | 5-15 |
| Response time | ~6000ms | <100ms |
| Memory overhead | None | ~20 shard handles |
| Cache hit rate (typical) | N/A | >90% |

---

## Risks and Mitigations

### Risk: Stale Cache

**Scenario:** Cache serves old data after compaction.

**Mitigation:** Always invalidate before modifying segment files.

### Risk: Memory Growth

**Scenario:** Too many shards cached.

**Mitigation:** LRU with fixed size naturally bounds memory.

### Risk: Lock Contention

**Scenario:** Many threads competing for cache lock.

**Mitigation:**
- Keep lock held briefly
- Consider `quick_cache` or `moka` for lock-free access
- Split into read/write locks if needed

### Risk: File Handle Exhaustion

**Scenario:** Too many open files.

**Mitigation:**
- Each cached shard = ~5 file handles
- 20 shards = ~100 handles
- Well within typical ulimit (usually 1024+)

---

## Future Enhancements

1. **Configurable cache size** - CLI flag `--segment-cache-size`
2. **Cache metrics** - Track hit rate, evictions for monitoring
3. **Warm cache on startup** - Pre-load recent shards
4. **Read-through pattern** - Combine bitset check + segment access

---

## Additional Recommendations from Reth Analysis (Gemini)

See `04-reth-gemini-research.md` for full details.

### Immediate Value

1. **Per-cursor decompression buffers** - Reuse `Vec<u8>` buffers to avoid malloc per block
2. **Arc-wrapped mmap handles** - Share handles across cursors without duplicating file descriptors

### Future Considerations

3. **Thundering herd protection** - If multiple RPC requests hit same missing block simultaneously, only fetch once and broadcast result to all waiters (MultiConsumer pattern)

4. **Hot vs Cold access differentiation** - For `eth_getLogs`:
   - Small/recent queries → use LRU cache
   - Large/historical queries → bypass cache, go direct to storage (avoids cache pollution)

5. **Zstd dictionaries** - Train per-column dictionaries for better compression ratios on repetitive data (headers, receipts)

6. **Binary search on sorted segments** - For range queries, use binary search on mmap'd data instead of linear scan

> **Note:** Future items added to `ROADMAP.md` under "Future: Performance & Storage"
