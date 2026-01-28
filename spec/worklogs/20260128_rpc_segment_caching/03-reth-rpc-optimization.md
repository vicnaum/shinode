# Reth RPC Optimization Patterns

## Overview

This document summarizes how Reth optimizes RPC performance, particularly for `eth_getLogs` and static file access. These patterns inform our segment caching fix for the stateless history node.

---

## 1. Multi-Layer Caching Architecture

Reth uses a **three-layer caching strategy**:

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: RPC Cache (EthStateCache)                          │
│   - LRU cache for blocks, receipts, headers                 │
│   - Async service with concurrent request handling          │
│   - Cache key: block hash (B256)                            │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: Static File Provider Cache (DashMap)               │
│   - Caches open NippyJar + mmap handles                     │
│   - Cache key: (block_range_end, segment_type)              │
│   - No eviction (TODO in Reth code: add size limit)         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: Memory-Mapped I/O                                  │
│   - DataReader uses mmap for data + offsets files           │
│   - OS-level page cache handles hot/cold data               │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Static File Provider Caching (Most Relevant to Our Fix)

**File:** `reth/crates/storage/provider/src/providers/static_file/manager.rs`

### Core Data Structures

```rust
/// StaticFileProviderInner manages all NippyJar files
pub struct StaticFileProviderInner<N> {
    /// Concurrent cache of open segment files
    /// Key: (block_range_end, segment_type)
    /// Value: LoadedJar (NippyJar + mmap handle)
    map: DashMap<(BlockNumber, StaticFileSegment), LoadedJar>,

    /// Index of available files per segment
    indexes: RwLock<StaticFileMap<StaticFileSegmentIndex>>,

    /// ... other fields
}
```

**File:** `reth/crates/storage/provider/src/providers/static_file/mod.rs`

```rust
/// Cached jar with shared mmap handle
pub struct LoadedJar {
    jar: NippyJar<SegmentHeader>,
    mmap_handle: Arc<reth_nippy_jar::DataReader>,
}

impl LoadedJar {
    fn new(jar: NippyJar<SegmentHeader>) -> ProviderResult<Self> {
        // Opens data file with mmap, wraps in Arc for sharing
        let mmap_handle = Arc::new(jar.open_data_reader()?);
        Ok(Self { jar, mmap_handle })
    }
}
```

### Cache Access Pattern

**File:** `reth/crates/storage/provider/src/providers/static_file/manager.rs:945-968`

```rust
fn get_or_create_jar_provider(
    &self,
    segment: StaticFileSegment,
    fixed_block_range: &SegmentRangeInclusive,
) -> ProviderResult<StaticFileJarProvider<'_, N>> {
    let key = (fixed_block_range.end(), segment);

    // Avoid write lock in common case - check cache first
    if let Some(jar) = self.map.get(&key) {
        return Ok(jar.into());
    }

    // Cache miss: load jar and insert
    let path = self.path.join(segment.filename(fixed_block_range));
    let jar = NippyJar::load(&path)?;
    Ok(self.map.entry(key).insert(LoadedJar::new(jar)?).downgrade().into())
}
```

### Key Insights

1. **DashMap for concurrent access** - Multiple RPC handlers can access different segments simultaneously
2. **Arc<DataReader> for shared mmap** - Multiple cursors can share the same memory-mapped file
3. **Cache key is range end** - Enables O(1) lookup for any block in a range
4. **No eviction (yet)** - Reth has a TODO: "we should check the size and pop N if there's too many"

---

## 3. RPC Cache Layer (EthStateCache)

**File:** `reth/crates/rpc/rpc-eth-types/src/cache/mod.rs`

### Configuration

**File:** `reth/crates/rpc/rpc-eth-types/src/cache/config.rs`

```rust
pub struct EthStateCacheConfig {
    pub max_blocks: u32,           // Default: 5000
    pub max_receipts: u32,         // Default: 2000
    pub max_headers: u32,          // Default: 1000
    pub max_concurrent_db_requests: usize,  // Default: 512
}
```

### MultiConsumerLruCache

**File:** `reth/crates/rpc/rpc-eth-types/src/cache/multi_consumer.rs`

```rust
/// LRU cache that queues consumers waiting for cache miss resolution
pub struct MultiConsumerLruCache<K, V, L, S> {
    cache: LruMap<K, V, L>,           // schnellru LRU
    queued: HashMap<K, Vec<S>>,       // Pending consumers
    metrics: CacheMetrics,
    memory_usage: usize,
}
```

Key features:
- Uses `schnellru::LruMap` (faster than `lru` crate)
- Tracks memory usage for metrics
- Queues multiple consumers waiting for same key (avoids duplicate fetches)

---

## 4. eth_getLogs Implementation

**File:** `reth/crates/rpc/rpc/src/eth/filter.rs`
**Knowledge Base:** `spec/reth_kb/questions/Q022-eth-getlogs.md`

### Two-Stage Filtering Algorithm

```
Stage 1: Bloom Filter Scan (fast, in-memory)
├── Iterate headers in range (chunks of 1000)
├── Check: filter.matches_bloom(header.logs_bloom())
├── If match: add to candidate list
└── If no match: skip entirely (no DB access)

Stage 2: Receipt Fetch + Log Match (slower, I/O)
├── For each candidate header:
│   ├── Try cache first (CachedMode)
│   └── Fall back to provider (RangeBlockMode)
├── For each receipt:
│   ├── For each log:
│   │   └── Check: filter.matches(log)
│   └── Append matching logs
└── Enforce max_logs_per_response limit
```

### Execution Modes

| Mode | When Used | How It Works |
|------|-----------|--------------|
| CachedMode | Recent blocks, small ranges | Uses `EthStateCache` LRU |
| RangeBlockMode | Historical, large ranges | Parallel `spawn_blocking` tasks |

### Key Optimizations

1. **Bloom filter pre-filtering** - Eliminates most blocks without I/O
2. **Parallel receipt fetching** - For ranges > 1000 headers, spawns parallel tasks
3. **Rate limiting** - Semaphore limits concurrent DB operations (default: 512)
4. **Cache check before provider** - `maybe_cached_block_and_receipts()` hits LRU first

---

## 5. Static Files + NippyJar Architecture

**Knowledge Base:** `spec/reth_kb/questions/Q009-static-files-nippyjar.md`

### File Layout

```
static_files/
├── static_file_headers_0_499999       # Data (no extension)
├── static_file_headers_0_499999.off   # Offsets (row positions)
├── static_file_headers_0_499999.conf  # Config (compression, column count)
├── static_file_headers_0_499999.idx   # Index (optional)
├── static_file_receipts_0_499999
├── static_file_receipts_0_499999.off
├── ...
```

### Memory-Mapped Reading

```rust
pub struct DataReader {
    data_mmap: Mmap,      // Memory-mapped data file
    offset_mmap: Mmap,    // Memory-mapped offsets file
}
```

Benefits:
- Zero-copy reads
- OS page cache handles frequently accessed pages
- Multiple cursors share same mmap handle via `Arc<DataReader>`

---

## 6. Comparison: Reth vs Our v2 Storage

| Aspect | Reth | Our v2 (Current) | Our v2 (Proposed) |
|--------|------|------------------|-------------------|
| File granularity | 500K blocks/file | 100-10K blocks/shard | Same |
| File handle caching | DashMap (no eviction) | None (opens every read) | LRU cache |
| Mmap sharing | Arc<DataReader> | New mmap per read | Arc<ShardSegments> |
| Concurrent access | DashMap (lock-free reads) | Mutex per shard | LRU with Mutex |
| Range query | Batch read with cursor | Per-block iteration | Batch read |

### Key Takeaways for Our Fix

1. **Cache segment readers, not individual reads** - Reth caches `LoadedJar` which includes open file handles
2. **Share mmap handles with Arc** - Avoid reopening files for each cursor
3. **Use concurrent map** - `DashMap` or similar for lock-free read access
4. **Consider LRU eviction** - Reth doesn't have this (TODO in their code), but we need it with smaller shards

---

## 7. Implementation Recommendations

Based on Reth's patterns, our fix should:

### Option A: DashMap + LRU (Recommended)

```rust
pub struct Storage {
    // ... existing fields

    /// LRU cache of segment readers per shard
    /// Key: shard_start
    /// Value: Arc<ShardSegments> with open file handles
    segment_cache: Mutex<LruCache<u64, Arc<ShardSegments>>>,
}
```

### Option B: DashMap without LRU (Simpler, matches Reth)

```rust
pub struct Storage {
    // ... existing fields

    /// Concurrent cache of segment readers
    segment_cache: DashMap<u64, Arc<ShardSegments>>,
}
```

### Invalidation Strategy

Follow Reth's pattern:
1. **Before compaction** - Invalidate affected shard
2. **On file modification** - Reth uses `notify` crate to watch for changes
3. **Manual invalidation** - `remove_cached_provider()` equivalent

---

## 8. References

### Reth Files

| Purpose | File Path |
|---------|-----------|
| Static file manager | `reth/crates/storage/provider/src/providers/static_file/manager.rs` |
| LoadedJar struct | `reth/crates/storage/provider/src/providers/static_file/mod.rs` |
| RPC cache | `reth/crates/rpc/rpc-eth-types/src/cache/mod.rs` |
| Cache config | `reth/crates/rpc/rpc-eth-types/src/cache/config.rs` |
| MultiConsumerLruCache | `reth/crates/rpc/rpc-eth-types/src/cache/multi_consumer.rs` |
| eth_getLogs | `reth/crates/rpc/rpc/src/eth/filter.rs` |
| NippyJar format | `reth/crates/storage/nippy-jar/src/lib.rs` |

### Knowledge Base

| Question | Path |
|----------|------|
| eth_getLogs implementation | `spec/reth_kb/questions/Q022-eth-getlogs.md` |
| Static files + NippyJar | `spec/reth_kb/questions/Q009-static-files-nippyjar.md` |
| Storage entry points | `spec/reth_kb/questions/Q004-storage-entrypoints.md` |

### Key Code Patterns

**Cache-aside pattern (Reth):**
```rust
// manager.rs:954-962
if let Some(jar) = self.map.get(&key) {
    return Ok(jar.into());  // Cache hit
}
// Cache miss: load and insert
let jar = NippyJar::load(&path)?;
self.map.entry(key).insert(LoadedJar::new(jar)?).downgrade().into()
```

**LRU with metrics (Reth):**
```rust
// multi_consumer.rs:81-89
pub fn get(&mut self, key: &K) -> Option<&mut V> {
    let entry = self.cache.get(key);
    if entry.is_some() {
        self.metrics.hits_total.increment(1);
    } else {
        self.metrics.misses_total.increment(1);
    }
    entry
}
```
