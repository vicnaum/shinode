# Atomic Compaction & Recovery Plan

## Problem Statement

The current compaction process can leave shards in an inconsistent state if interrupted:
- Bitset is updated during WAL writes, before data is compacted to segments
- If compaction fails mid-way, bitset claims blocks exist that aren't in the segment
- This causes `missing tx_meta for block X` errors on restart

## Core Principles

1. **Never modify live files during operations** - always write to `.tmp` files first
2. **Atomic swaps** - rename `.tmp` → live, keeping `.old` backups
3. **Track operation state in metadata** - know exactly where we were interrupted
4. **Recovery = resume from interrupted step** - don't lose downloaded data

---

## Design

### Shard Metadata (shard.json)

```json
{
  "shard_start": 24280000,
  "shard_size": 10000,
  "present_count": 7996,
  "complete": true,
  "sorted": true,
  "sealed": false,
  "tail_block": 24289999,
  "content_hash": null,
  "compaction_phase": null
}
```

**New field: `compaction_phase`**
- `null` - not compacting, clean state
- `"writing"` - writing tmp files (phase 1)
- `"swapping"` - performing atomic swaps (phase 2)
- `"cleanup"` - deleting backups and WAL (phase 3)

### File Layout

```
shards/24280000/
├── shard.json           # metadata (source of truth)
├── present.bitset       # live bitset
├── bitset.tmp           # new bitset during compaction
├── bitset.old           # backup during swap
├── state/
│   └── staging.wal      # write-ahead log for pending blocks
├── sorted/              # live segment files
│   ├── headers
│   ├── headers.off
│   ├── tx_hashes
│   ├── tx_hashes.off
│   ├── tx_meta
│   ├── tx_meta.off
│   ├── receipts
│   ├── receipts.off
│   ├── block_sizes
│   └── block_sizes.off
├── sorted.tmp/          # new segments during compaction
└── sorted.old/          # backup during swap
```

---

## Operations

### INGEST (Writing Blocks to WAL)

**Current behavior (problematic):**
```
1. Write block to WAL
2. Update bitset immediately  ← CAUSES DESYNC
```

**New behavior:**
```
1. Write block to WAL (append-only)
2. DO NOT touch bitset
```

The bitset only reflects what's in the sorted segment, never the WAL.

### COMPACT (Merge WAL into Sorted Segment)

#### Phase 1: WRITING

```
1. Set compaction_phase = "writing" in shard.json
2. Write sorted.tmp/headers (+ .off)
3. Write sorted.tmp/tx_hashes (+ .off)
4. Write sorted.tmp/tx_meta (+ .off)
5. Write sorted.tmp/receipts (+ .off)
6. Write sorted.tmp/block_sizes (+ .off)
7. Compute bitset.tmp = merge(present.bitset, WAL block numbers)
```

**If interrupted:** Delete `sorted.tmp/` and `bitset.tmp`, reset `compaction_phase = null`. WAL remains, will re-compact later.

#### Phase 2: SWAPPING

```
8.  Set compaction_phase = "swapping" in shard.json
9.  Rename sorted → sorted.old
10. Rename sorted.tmp → sorted
11. Rename present.bitset → bitset.old
12. Rename bitset.tmp → present.bitset
```

**If interrupted:** Recover to consistent state (see Recovery Logic), then reset to phase 1 state.

#### Phase 3: CLEANUP

```
13. Set compaction_phase = "cleanup" in shard.json
14. Delete sorted.old/
15. Delete bitset.old
16. Delete staging.wal
17. Update shard.json:
    - sorted = true
    - present_count = count from new bitset
    - compaction_phase = null
```

**If interrupted:** Complete the remaining deletes and metadata update.

---

## Startup Recovery Logic

```rust
for each shard_dir:
    load shard.json

    match compaction_phase:
        "writing":
            // Interrupted while writing tmp files
            delete sorted.tmp/ if exists
            delete bitset.tmp if exists
            set compaction_phase = null
            persist shard.json
            // WAL still exists, will re-compact on next trigger

        "swapping":
            // Interrupted during atomic swaps - recover to consistent state
            recover_swap_state(shard)

        "cleanup":
            // Interrupted during cleanup - just finish it
            delete sorted.old/ if exists
            delete bitset.old if exists
            delete staging.wal if exists
            set sorted = true
            set compaction_phase = null
            persist shard.json

        null:
            // Clean state, but check for orphan files from hard crashes
            if sorted.tmp/ exists:
                log warning, delete it
            if sorted.old/ exists:
                log warning, delete it
            if bitset.tmp exists:
                log warning, delete it
            if bitset.old exists:
                log warning, delete it
```

### recover_swap_state()

```rust
fn recover_swap_state(shard):
    // Goal: restore to pre-compaction state, then retry

    // Step 1: Ensure sorted/ is valid
    if sorted/ does not exist and sorted.old/ exists:
        rename sorted.old/ → sorted/

    // Step 2: Ensure present.bitset is valid
    if present.bitset does not exist and bitset.old exists:
        rename bitset.old → present.bitset

    // Step 3: Clean up all tmp/old files
    delete sorted.tmp/ if exists
    delete sorted.old/ if exists
    delete bitset.tmp if exists
    delete bitset.old if exists

    // Step 4: Reset state - WAL still exists, will re-compact
    set compaction_phase = null
    set sorted = false  // Force re-compaction
    persist shard.json
```

---

## State Diagram

```
                    ┌─────────────┐
                    │    EMPTY    │
                    └──────┬──────┘
                           │ create shard
                           ▼
                    ┌─────────────┐
         ┌─────────▶│  INGESTING  │◀─────────┐
         │          │ (WAL only)  │          │
         │          └──────┬──────┘          │
         │                 │ trigger compact │
         │                 ▼                 │
         │          ┌─────────────┐          │
         │          │   WRITING   │          │
   recovery         │ (phase 1)   │     recovery
         │          └──────┬──────┘          │
         │                 │ tmp files done  │
         │                 ▼                 │
         │          ┌─────────────┐          │
         └──────────│   SWAPPING  │──────────┘
                    │ (phase 2)   │
                    └──────┬──────┘
                           │ swaps done
                           ▼
                    ┌─────────────┐
                    │   CLEANUP   │
                    │ (phase 3)   │
                    └──────┬──────┘
                           │ cleanup done
                           ▼
                    ┌─────────────┐
                    │    READY    │
                    │(sorted=true)│
                    └──────┬──────┘
                           │ complete + seal
                           ▼
                    ┌─────────────┐
                    │   SEALED    │
                    └─────────────┘
```

---

## UI Changes

### New Startup Phase: Recovery

Add orange status bars during shard recovery:

```
[Orange] Recovering shard 24280000: incomplete compaction (phase: writing)
[Orange] Recovering shard 24310000: cleaning orphan files
```

### Color Scheme Update

| Phase | Color | RGB |
|-------|-------|-----|
| Startup | Yellow | (255, 200, 0) |
| **Recovery** | **Orange** | **(255, 140, 0)** |
| Syncing | Cyan | (0, 200, 255) |
| Finalizing | Teal | (0, 200, 200) |
| Following | Green | (0, 128, 0) |
| Failed | Red | (255, 0, 0) |

---

## CLI Addition

### `--repair` Flag

```bash
# Run recovery/repair without starting sync
cargo run --release -- --repair

# Output:
Checking storage integrity...
Shard 24280000: recovering from incomplete compaction (phase: swapping)
  - Restored sorted/ from backup
  - Cleaned up tmp files
  - Reset compaction state
Shard 24310000: cleaning orphan sorted.tmp/
Shard 24290000: OK
Shard 24300000: OK
Recovery complete. 2 shards repaired, 2 shards OK.
```

---

## Implementation Order

### Phase 1: Atomic Compaction (prevent future corruption)

1. Add `compaction_phase` field to `ShardMeta`
2. Modify `compact_shard_impl()` to:
   - Set phase before each stage
   - Write to tmp files
   - Perform atomic swaps
   - Clean up and reset phase
3. **Remove bitset updates from WAL write path**
4. Compute bitset.tmp during compaction instead

### Phase 2: Recovery Logic (handle existing corruption)

5. Add `recover_shard()` function
6. Call recovery during `Storage::open()` for each shard
7. Add orange "Recovering" UI phase

### Phase 3: CLI Repair Command

8. Add `--repair` CLI flag
9. Run recovery logic without starting sync
10. Report results

### Phase 4: Testing

11. Unit tests for each recovery scenario
12. Integration test: kill process during compaction, verify recovery
13. Test with actual corrupted shard from the bug

---

## Migration

Existing shards without `compaction_phase` field:
- Treat `null`/missing as clean state
- Check for orphan tmp/old files and clean them
- If inconsistency detected (WAL exists but sorted=true), set sorted=false to trigger re-compaction

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Data loss during recovery | Always restore from .old backup before deleting |
| Infinite recovery loop | Track recovery attempts, fail after N retries |
| Disk full during compaction | Check disk space before starting, clean tmp on failure |
| Corrupted WAL | WAL has CRC checksums, skip bad records |

---

## Success Criteria

1. No more "missing tx_meta" errors from bitset/segment desync
2. Process can be killed at any point during compaction and recover cleanly
3. No data loss - interrupted compactions resume from WAL
4. Clear UI feedback during recovery
5. `--repair` command for manual intervention
