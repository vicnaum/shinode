# Sharding Refactor Plan

## Overview
Refactor the storage layer to split data into **10,000-block shards**. This removes the global contiguity constraint, allowing the node to ingest blocks out-of-order, backfill older history, and isolate "rare/missing" block stalls to specific shards without blocking the entire pipeline.

## 1. Storage Architecture

### Directory Layout
Data is split into shards aligned to 10k boundaries (e.g., `0`, `10000`, `20000`).

```text
data_dir/
  meta.json          (Global metadata: schema_version, etc.)
  static/
    shards/
      0/
        shard.json       (Shard metadata)
        present.bitset   (Availability map)
        index.bin        (Map: BlockNumber -> RowIndex)
        headers.nippy    (Data segments...)
        receipts.nippy
        ...
      10000/
        ...
```

### Shard Alignment
- **Size**: 10,000 blocks.
- **Naming**: Directory name is the start block of the shard (e.g., `1250000` for range `[1250000, 1259999]`).
- **Calculation**: `shard_start = (block_number / 10000) * 10000`.

### File Formats

#### `shard.json`
Tracks the state of the shard.
```json
{
  "start_block": 1250000,
  "end_block": 1259999,
  "present_count": 450,
  "is_sorted": false,
  "is_complete": false,
  "last_updated": 1709823423
}
```

#### `present.bitset`
- **Format**: Binary file, 1250 bytes (10,000 bits).
- **Semantics**: Bit `i` corresponds to block `start_block + i`. `1` = present (full bundle stored), `0` = missing.
- **Usage**: Quick check for "do we have this block?" and "is shard complete?".

#### `index.bin` (The "Sparse Index")
- **Problem**: We append blocks randomly (unsorted). `NippyJar` stores rows sequentially. We need to map `LogicalBlockNumber` -> `PhysicalRowIndex`.
- **Format**: Simple binary array of `u16` (since 10k < 65535).
  - Or a sorted map `[(u16 offset, u16 row_idx)]`.
  - **Proposal**: Since shards are small (10k), we can keep the index in memory as a `HashMap<u64, usize>` or `Vec<u16>` (if dense) while running, and persist it as a simple list of `block_offset -> row_index` pairs or a direct lookup table if sorted.
  - **For Unsorted**: Append `(block_offset_u16, row_index_u16)` pairs.
  - **For Sorted**: If fully contiguous, index is implicit (`row = block - start`). If gaps exist, we still need the map (or binary search on a "block number" column).
  - **Decision**: Persist as `index.u16` (array of `u16` row indices).
    - If `index.u16` size < shard size, it's a sparse map? No, let's just store `(u16 block_offset, u16 row_index)` pairs for the unsorted version.

### Write Strategy (Append-Only + Sort Later)

1.  **Ingest (Unsorted)**:
    - Incoming blocks for a shard are **appended** to the `.nippy` files in arrival order.
    - Update `present.bitset`.
    - Append entry to `index` map.
    - Update `shard.json` (`present_count++`).
    - **No global lock**: Only lock the specific shard being written.

2.  **Compaction (Sorting)**:
    - **Trigger**: When a shard is "done" for the current run (queue empty for this shard) OR when `is_complete` becomes true.
    - **Process**:
        1.  Read all rows from the unsorted shard.
        2.  Sort them by block number in memory.
        3.  Rewrite the `.nippy` files in sorted order.
        4.  Update `shard.json` -> `"is_sorted": true`.
        5.  (Optional) If contiguous/complete, delete the `index` file (implicit mapping).

## 2. Refactoring Plan

### Phase A: Storage Engine Implementation
1.  **`ShardedStorage` Struct**: Create new storage struct that manages shard directories.
2.  **Shard Management**: Logic to open/create shard directories, load/save `shard.json` and `present.bitset`.
3.  **Bitset Logic**: Helper to set/get bits and calculate completeness.
4.  **Schema Version**: Bump `meta.json` schema version to `2`.

### Phase B: Read Path (RPC & Internal)
1.  **Routing**: `storage.block_header(N)` calculates shard ID -> loads shard metadata -> checks bitset -> looks up row index -> reads from NippyJar.
2.  **Caching**: Keep frequently accessed shard metadata/indices in an LRU cache (don't open 1000 files on every request).
3.  **`eth_blockNumber`**: Return the highest block `N` such that all blocks `start..N` are present (contiguous tip). Or just `max_present` if we relax semantics. *Decision: Max present block (per user preference).*

### Phase C: Write Path (The "Shard Writer")
1.  **ShardWriter**: Replaces the current `DbWriter`.
    - Accepts `BlockBundle`.
    - Routes to appropriate `Shard` handle.
    - Appends to shard (unsorted).
    - Updates bitset/index.
2.  **Duplicate Guard**: Check `present.bitset` before writing. If present, skip.

### Phase D: Compaction / Sorting Logic
1.  **Sorter**: Function to take a `Shard`, read all bundles, sort, rewrite.
2.  **Triggering**:
    - Explicit method `storage.sort_shard(id)`.
    - Called by the pipeline when conditions are met.

### Phase E: Pipeline & Scheduler Integration
1.  **Queue Organization**:
    - Scheduler maintains queues **per shard** (or groups pending blocks by shard).
    - "Active Shards": Focus on N shards at a time to minimize open files/memory.
2.  **End-of-Shard Event**:
    - When Scheduler sees `pending` count for Shard X drops to 0:
        - Send `SortShard(X)` signal to `ShardWriter`.
3.  **Backpressure**:
    - Instead of `max_lookahead_blocks`, use **Active Shards limit**.
    - E.g., "Work on max 5 shards in parallel".
    - If a shard is stuck (rare blocks), it stays "Active" but idle. Other shards continue.

## 3. Future / Roadmap Items
- **Node-to-Node Sync**: Ship `shard.tar.zst` when `is_complete && is_sorted`.
- **Backfill**: User can request range `[0, 10000]`. Node activates Shard 0, fills it, sorts it.

## Questions / Open Items
1.  **Index Format**: Confirm `(u16 offset, u16 row)` binary format for unsorted index? (Simple and compact).
2.  **Memory Limit**: With 10k blocks, a full shard rewrite requires reading ~2.6GB (if full) into memory to sort?
    - *Mitigation*: 2.6GB is fine for most modern nodes. If tight, we can sort indices and read-write stream (random read, sequential write).
    - *Decision*: In-memory sort is fine for v1.

---
**Next Step**: Begin Phase A (Storage Engine scaffolding).
