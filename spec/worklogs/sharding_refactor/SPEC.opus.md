I've created the comprehensive `SHARDING_REFACTOR_PLAN.md` document below. Here's a summary of the key design decisions captured:

**Core Architecture:**
- 10,000 blocks per shard, aligned to `floor(n/10000)*10000`
- Each shard has its own NippyJar file + JSON metadata file
- Presence tracking via per-shard bitmaps (1.25 KB each)

**Write Strategy (Append-Then-Sort):**
- During fetching: append blocks in fetch order (unsorted)
- When shard queue empties: sort the shard and mark as complete/partial
- Blocks have a fixed 8-byte block number prefix for efficient sorting without full decode

**Queue/Scheduler Changes:**
- Queue split by shard for completion detection
- Pre-filter queue using presence bitmaps (skip already-stored blocks)
- Per-shard completion callbacks trigger immediate sorting

**Shard States:**
- `empty` → `writing` → `complete`/`partial`
- `sorted: true/false` tracks whether shard is binary-searchable
- Partial shards are still shareable once sorted

**Metadata Files:**
- `shard_{id}.json`: per-shard state, presence bitmap, checksums
- `meta.json`: global state, schema version 2, max_present_block

**Migration:**
- No migration needed - delete existing data and start fresh
- Schema version bump to 2 with incompatibility check

**Open Questions for Discussion:**
1. NippyJar row format (I recommended block number prefix)
2. Sort execution model (sync vs async vs dedicated thread)
3. Shard file format optimization
4. Partial shard query behavior
5. Shard size tuning (10K vs other sizes)

Would you like me to:
- Elaborate on any section?
- Add more detail to the queue/scheduler refactor?
- Address any of the open questions with a specific recommendation?
- Start implementing Phase A (storage layer types)?

---

# Sharding Refactor Plan

This document specifies the storage refactor from monolithic NippyJar segments to a **shard-based architecture** that enables parallel ingestion, efficient sharing, and incremental verification.

## Goals

1. **Parallel ingestion**: Allow multiple block ranges to be fetched/written concurrently without conflicts
2. **Incremental sharing**: Enable node-to-node transfer of complete shard files
3. **Presence tracking**: Efficiently track which blocks are stored via bitmap indexes
4. **Out-of-order writes**: Accept blocks in any order during fetch, sort on completion
5. **Resume safety**: Correctly resume from any partial state

---

## Design Decisions

### Shard Alignment

Shards are **10,000 blocks** each, aligned to `floor(block_number / 10000) * 10000`:

| Shard ID | Block Range |
|----------|-------------|
| 0 | 0 – 9,999 |
| 1 | 10,000 – 19,999 |
| 2 | 20,000 – 29,999 |
| ... | ... |

**Naming convention**: `shard_{shard_id:08}.jar` (e.g., `shard_00002000.jar` for blocks 20M–20,009,999)

### Block Definition

A **block** is a complete `BlockBundle`:
- Header
- Transaction hashes
- Transaction metadata (from, to, value, nonce, signature, signing_hash)
- Withdrawals (if present)
- Block size
- Receipts
- Logs

We **never** write partial blocks. A block is either fully present or absent.

### Range Boundaries

- **Start block**: User-configured `start_block` (may be mid-shard)
- **End block**: Current `max_present_block` in storage (not necessarily from genesis)
- **Head source**: P2P-derived head (as configured)

When starting from a non-genesis block (e.g., 18,000,000), only shards from shard 1800+ are populated.

---

## Storage Layout

### Directory Structure

```
data_dir/
├── meta.json                    # Global metadata
├── peers.json                   # Peer cache
├── shards/                      # Shard directory
│   ├── shard_00001800.jar       # NippyJar file (blocks 18M-18,009,999)
│   ├── shard_00001800.jar.off   # NippyJar offset file
│   ├── shard_00001800.jar.idx   # NippyJar index file
│   ├── shard_00001800.json      # Shard metadata
│   ├── shard_00001801.jar       # Next shard
│   ├── shard_00001801.json
│   └── ...
└── global.bitmap                # Optional: global presence bitmap for fast queries
```

### Shard Metadata (`shard_{id}.json`)

Each shard has a JSON metadata file tracking its state:

```json
{
  "shard_id": 1800,
  "start_block": 18000000,
  "end_block": 18009999,
  "block_count": 10000,
  "blocks_present": 8542,
  "state": "complete",
  "sorted": true,
  "presence_bitmap": "base64-encoded-bitmap-or-path",
  "checksum": "sha256-of-jar-file",
  "created_at": "2025-01-22T10:00:00Z",
  "sorted_at": "2025-01-22T10:15:00Z",
  "last_write_at": "2025-01-22T10:14:59Z"
}
```

**State machine**:
- `empty` → No blocks written yet
- `writing` → Actively receiving blocks (unsorted append mode)
- `complete` → All blocks for the range present
- `partial` → Some blocks present, ingestion run finished (sorted, shareable)

**Sorted flag**:
- `false` → Blocks are appended in fetch order (unsorted)
- `true` → Blocks are sorted by block number (binary-searchable, shareable)

### Global Metadata (`meta.json`)

Updated to track shard-aware state:

```json
{
  "schema_version": 2,
  "chain_id": 1,
  "storage_key": { ... },
  "start_block": 18000000,
  "max_present_block": 21500000,
  "shard_size": 10000,
  "last_indexed_block": 21499999
}
```

---

## Presence Tracking

### Bitmap Index

Each shard maintains a **presence bitmap** (10,000 bits = 1.25 KB per shard):
- Bit `i` = 1 → block `shard_start + i` is present
- Bit `i` = 0 → block is absent

**Storage options** (in order of preference):
1. Inline in `shard_{id}.json` as base64 (for small shards, <2KB)
2. Separate `.bitmap` file alongside the shard

### Global Bitmap (Optional)

For fast `is_block_present(n)` queries across all shards:
- Concatenated bitmap of all shards
- Updated lazily or on-demand
- Useful for queue pre-filtering

---

## Write Path (Append-Then-Sort)

### Phase 1: Unsorted Append

During active fetching:

1. Blocks arrive in **fetch order** (not block order)
2. Each block is **appended** to the end of its shard's NippyJar
3. Shard metadata updates:
   - `blocks_present += 1`
   - `state = "writing"`
   - `sorted = false`
   - Presence bitmap bit set

**NippyJar row format** (single column per row):
```
Row N: [block_number: u64 | bundle_data: bincode-encoded BlockBundle]
```

The block number prefix enables sorting without decoding the full bundle.

### Phase 2: Sort on Completion

When a shard's ingestion run is done:

1. **Trigger conditions**:
   - Shard queue is empty (all requested blocks fetched)
   - Explicit flush request
   - Graceful shutdown

2. **Sorting process**:
   ```
   a. Read all rows from unsorted jar
   b. Sort by block_number (first 8 bytes of each row)
   c. Write to new temporary jar in sorted order
   d. Atomic rename: temp.jar → shard_{id}.jar
   e. Update metadata: sorted = true, state = "complete" or "partial"
   ```

3. **Optimization**: If blocks arrived mostly in order, sorting is fast (nearly-sorted data)

### Partial Shards

A shard may be **partial** when:
- We start from `start_block` in the middle of a shard
- We only fetch a specific range (e.g., for debugging)
- Some blocks failed all retry attempts

Partial shards are still **sorted and shareable** after the ingestion run completes.

---

## Read Path

### Sorted Shard (Normal Case)

1. Compute shard ID: `shard_id = block_number / 10000`
2. Compute row index: `row = block_number - (shard_id * 10000)`
3. Check presence bitmap (fast rejection for absent blocks)
4. Binary search or direct index into NippyJar

### Unsorted Shard (Edge Case)

During active writing, reads from unsorted shards:
1. Linear scan (acceptable for small shard sizes)
2. Or maintain an in-memory block→row index

For v1, we accept that reads during active writing may be slower. The shard is sorted on completion.

---

## Queue and Scheduler Refactor

### Shard-Aware Queue

The ingestion queue is reorganized by shard:

```rust
struct ShardQueue {
    shard_id: u64,
    pending: BinaryHeap<Reverse<u64>>,  // blocks to fetch
    in_flight: HashSet<u64>,
    completed: HashSet<u64>,
}

struct ShardAwareScheduler {
    shards: HashMap<u64, ShardQueue>,
    shard_order: VecDeque<u64>,  // round-robin or priority
    presence_index: ShardPresenceIndex,  // for pre-filtering
}
```

### Pre-filtering by Presence

When building the initial queue:

```rust
fn build_queue(start: u64, end: u64, presence: &ShardPresenceIndex) -> ShardAwareScheduler {
    let mut scheduler = ShardAwareScheduler::new();
    for block in start..=end {
        if !presence.is_present(block) {
            let shard_id = block / SHARD_SIZE;
            scheduler.enqueue(shard_id, block);
        }
    }
    scheduler
}
```

This avoids re-fetching blocks we already have.

### Shard Completion Detection

When a shard's queue becomes empty:

```rust
async fn on_shard_queue_empty(&mut self, shard_id: u64) {
    // All blocks for this shard in this run have been fetched
    self.trigger_shard_sort(shard_id).await;
}
```

This enables immediate sorting without waiting for the entire sync to finish.

---

## Migration Strategy

### No Migration Required

Since all current data is benchmark/test data:

1. **Delete existing data directory** (or use new `--data-dir`)
2. Start fresh with schema version 2
3. No backward compatibility code needed

### Schema Version Bump

```rust
const SCHEMA_VERSION: u64 = 2;
```

If `meta.json` exists with `schema_version = 1`:
- Print warning: "Incompatible storage version. Please delete data directory and restart."
- Exit with error

---

## Implementation Phases

### Phase A: Storage Layer Refactor

1. **New `ShardedStorage` struct**:
   ```rust
   pub struct ShardedStorage {
       data_dir: PathBuf,
       shard_size: u64,
       meta: Mutex<GlobalMeta>,
       shards: RwLock<HashMap<u64, ShardHandle>>,
       presence: ShardPresenceIndex,
   }
   ```

2. **`ShardHandle` for per-shard operations**:
   ```rust
   struct ShardHandle {
       shard_id: u64,
       jar_path: PathBuf,
       meta_path: PathBuf,
       state: Mutex<ShardState>,
       writer: Option<Mutex<NippyJarWriter>>,
   }
   ```

3. **Presence index implementation**:
   ```rust
   struct ShardPresenceIndex {
       bitmaps: RwLock<HashMap<u64, BitVec>>,
   }
   
   impl ShardPresenceIndex {
       fn is_present(&self, block: u64) -> bool;
       fn set_present(&self, block: u64);
       fn load_from_shards(&mut self, shards_dir: &Path);
   }
   ```

### Phase B: Write Path Changes

1. **Unsorted append**:
   ```rust
   impl ShardedStorage {
       pub fn append_block(&self, bundle: BlockBundle) -> Result<()> {
           let shard_id = bundle.number / self.shard_size;
           let shard = self.get_or_create_shard(shard_id)?;
           shard.append_unsorted(bundle)?;
           self.presence.set_present(bundle.number);
           Ok(())
       }
   }
   ```

2. **Sort-on-completion**:
   ```rust
   impl ShardHandle {
       pub async fn sort_and_finalize(&self) -> Result<()> {
           // Read all rows, sort by block number, write new jar, atomic rename
       }
   }
   ```

### Phase C: Queue/Scheduler Refactor

1. **Shard-aware queue builder**
2. **Pre-filtering by presence bitmap**
3. **Per-shard completion callbacks**
4. **Round-robin or priority-based shard selection**

### Phase D: DB Writer Integration

1. **Remove contiguous-block requirement**
2. **Accept blocks in any order**
3. **Route to correct shard based on block number**
4. **Trigger shard sorting on flush/shutdown**

### Phase E: Read Path Updates

1. **Shard-aware block lookup**
2. **Range queries spanning multiple shards**
3. **Handle mix of sorted and unsorted shards**

---

## Performance Considerations

### Write Performance

- **Unsorted append**: O(1) per block (just append to end)
- **Sort**: O(n log n) once per shard per ingestion run
- **Shard size tradeoff**: 10,000 blocks ≈ 50-100 MB per shard
  - Small enough to sort quickly
  - Large enough to minimize shard count overhead

### Read Performance

- **Sorted shard**: O(1) direct index or O(log n) binary search
- **Presence check**: O(1) bitmap lookup
- **Range query**: O(shards) + O(blocks per shard)

### Memory Usage

- **Per shard bitmap**: 1.25 KB
- **Global bitmap (20M blocks)**: 2,000 shards × 1.25 KB ≈ 2.5 MB
- **In-flight write buffer**: Controlled by `max_buffered_blocks`

---

## Future: Node-to-Node Shard Sync

(Deferred to post-refactor)

### Sharing Protocol Sketch

1. **Shard advertisement**: Node publishes available shards with checksums
2. **Shard request**: Peer requests specific shard files
3. **Transfer**: Direct file transfer (gzip compressed)
4. **Verification**: Checksum validation, optional Merkle proofs

### Trust Model

- Shards from trusted peers: Accept with checksum verification
- Shards from untrusted peers: Verify receipts root against header

---

## Open Questions

1. **NippyJar block number prefix**: The current NippyJar format stores bincode-encoded data. Should we:
   - a) Prefix each row with a fixed 8-byte block number for efficient sorting?
   - b) Decode full bundle to get block number (slower sort)?
   - c) Maintain a separate block→row index during unsorted phase?

2. **Sort parallelism**: Should sorting happen:
   - a) Synchronously in the writer thread (simpler, blocks writes briefly)?
   - b) In a background task (async, needs coordination)?
   - c) In a dedicated sorter thread (parallel with fetching)?

3. **Shard file format**: Should sorted shards use:
   - a) Same NippyJar with ordered rows?
   - b) Different format optimized for random access?
   - c) Memory-mapped format for even faster reads?

4. **Partial shard handling during queries**: When a shard is partial:
   - a) Return only present blocks (current plan)?
   - b) Return error for absent blocks?
   - c) Fall back to RPC for missing blocks?

5. **Shard size tuning**: Is 10,000 blocks optimal? Consider:
   - Smaller (1,000): More shards, finer granularity, more metadata overhead
   - Larger (100,000): Fewer shards, slower sorts, coarser sharing

---

## Testing Strategy

### Unit Tests

- Shard ID calculation
- Presence bitmap operations
- Unsorted append + sort correctness
- Shard metadata serialization

### Integration Tests

- Multi-shard write + read roundtrip
- Resume from partial shard
- Range query spanning shards
- Concurrent writes to different shards

### Stress Tests

- Out-of-order block arrival
- Large shard sorting performance
- Memory usage under load

---

## Timeline (Non-Binding Estimates)

| Phase | Description | Scope |
|-------|-------------|-------|
| A | Storage layer types + presence index | Foundation |
| B | Write path (append + sort) | Core functionality |
| C | Queue/scheduler refactor | Efficient ingestion |
| D | DB writer integration | End-to-end flow |
| E | Read path + RPC updates | Query support |
| - | Node-to-node sync | Future roadmap |

---

## Appendix: Row Format Options

### Option 1: Block Number Prefix

```
Row: [block_number: u64 LE][bundle_bytes: bincode]
```

Pros:
- Sort without full decode
- 8 bytes overhead per row

### Option 2: Indexed Header

```
Row: [bundle_bytes: bincode]
Separate: block_index.idx mapping block_number → row_offset
```

Pros:
- No row overhead
- Existing NippyJar compatibility

Cons:
- Need index during unsorted phase
- Extra file to manage

### Option 3: Tuple Format

```
Row: bincode::serialize(&(block_number, bundle))
```

Pros:
- Single bincode decode
- Natural Rust tuple

Cons:
- Must decode to sort

**Recommendation**: Option 1 (block number prefix) for simplicity and sort efficiency.
