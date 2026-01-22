# Sharding Refactor Plan (10k shards + bitset + compaction)

## Goals
- Store history in fixed-size shards (default 10,000 blocks) aligned to `floor(n/size)*size`.
- Track per-shard availability via a bitset + metadata with `complete` indicator.
- Allow out-of-order ingestion within a shard, then **sort/compact** the shard when no more blocks are expected for that shard in the current run.
- Enable later backfill of older ranges without requiring global contiguity.
- Lay groundwork for shard-level P2P sync (ship compressed shard artifacts).

## Non-goals (for this refactor)
- Node-to-node shard sync protocol details (will follow in a later roadmap).
- Migration from existing storage (benchmarks only; safe to delete).
- Changing RPC method surface (only storage backend).

---

## Current constraints that drive design
- Current storage uses NippyJar append-only segments, which require **monotonic block numbers**.
- NippyJar segments cannot be updated in-place, so "write placeholders then fill later" is not viable without rewriting.
- Therefore, **unsorted appends must go to a staging format**, and compaction will produce sorted NippyJar shards.

---

## Proposed on-disk layout

```
data/
  meta.json                       # schema_version=2, shard_size, storage_key
  shards/
    24180000/                     # shard_start (aligned by size)
      shard.json                  # shard metadata (see below)
      present.bitset              # 10,000 bits (1,250 bytes)
      incoming.log                # append-only unsorted BlockBundle records
      incoming.idx                # block_number -> offset index for incoming.log
      segments/                   # sorted shard content (NippyJar)
        headers.njar
        tx_hashes.njar
        tx_meta.njar
        receipts.njar
        block_sizes.njar
```

### `shard.json` (per shard)
Suggested fields (exact schema TBD):
```
{
  "shard_start": 24180000,
  "shard_size": 10000,
  "present_count": 5821,
  "complete": false,
  "sorted": false,
  "last_updated_ms": 1737543360000,
  "min_present": 24180012,
  "max_present": 24189998,
  "format_version": 1
}
```

### `present.bitset`
- Fixed length: `shard_size` bits (default 10,000).
- 1 = block present (full bundle), 0 = missing.
- Bit index = `block_number - shard_start`.

### `incoming.log` format (staging)
- Append-only records; each record contains:
  - `block_number: u64`
  - `bundle_bytes: Vec<u8>` (bincode of `BlockBundle`)
- Simple and fast append, easy to scan for compaction.

### `incoming.idx`
- Map `block_number -> offset` in `incoming.log` for fast reads.
- Persisted regularly (append-friendly or periodic rewrite).
- If missing/corrupt, can be rebuilt by scanning `incoming.log`.

---

## Read path semantics
1. Compute `shard_start`.
2. If bitset says **missing**, return `None`.
3. If bitset says **present**:
   - Check `incoming.idx` first (if present, read from `incoming.log`).
   - Else read from sorted `segments/` (NippyJar).
4. Range reads:
   - Read sorted segments by range.
   - Overlay any `incoming.log` entries from index within the range.

### `eth_blockNumber` semantics (per user decision)
- Return **max present block** (not necessarily contiguous).

---

## Write path semantics
- Only write **full bundles** (header + body + receipts-derived data).
- For each bundle:
  1. Compute `shard_start`.
  2. If bit already set → skip (idempotent).
  3. Append to `incoming.log` + update `incoming.idx`.
  4. Flip bit in `present.bitset`, increment `present_count`.
  5. If `present_count == shard_size` → set `complete=true`.

---

## Compaction (sorting) strategy

### Trigger
- **Primary trigger:** when shard queue is empty for this run (no more blocks to fetch for that shard).
- **Secondary trigger:** shard becomes `complete=true`.

### Behavior
- Read all entries from `incoming.log` + `incoming.idx`.
- Merge with any existing sorted `segments/` (if present).
- Produce **sorted NippyJar segments** for the shard.
- Replace/overwrite `segments/` with compacted output.
- Clear `incoming.log` and `incoming.idx`.
- Set `sorted=true`.

### Notes
- If new blocks arrive after compaction, they go back into `incoming.log` and `sorted=false` until next compaction.
- Compaction should be done in a background task (bounded concurrency).

---

## Scheduler / fetch pipeline changes (post-storage)

### Queue building
- At startup, build target queue by scanning shard bitsets.
- **Do not enqueue blocks whose bit is already present**.

### Shard-aware scheduling
- Maintain per-shard queues.
- Assign batches within a shard until it is exhausted for the run.
- Track “active shards” and trigger compaction when a shard queue is empty.

### Rare/gap handling (from previous discussion)
- Replace “failed” with **rare** queue.
- Rare queue runs **in parallel** with normal queue.
- Fanout attempts: try a rare block across multiple peers concurrently.
- Empty response handling:
  - If batch > 1 and response is empty → penalize peer, **do not** increment per-block attempts.
  - If batch == 1 and response is empty → treat as rare but **do not** permanently fail immediately.

### Lookahead
- With sharding, lookahead is less critical, but still useful for throughput.
- Update default `max_lookahead_blocks` to **50,000** (as agreed).
- Consider moving to a **bytes-based** buffer cap later.

---

## Config changes
- Add `--shard-size` (default 10000).
- Add storage schema version bump to `meta.json` (`schema_version: 2`).
- Retain `--fast-sync-max-lookahead-blocks` default 50k.

---

## Migration
- **No migration** required (benchmarks-only data). New schema requires fresh data dir.

---

## Test plan
- Unit tests:
  - Bitset set/clear + present_count correctness.
  - Incoming log append + index lookup correctness.
  - Compaction sorts and produces contiguous segments.
- Integration tests:
  - Partial shard ingest → compaction → read path works.
  - Backfill older range after newer range → shards fill correctly.
  - Restart with existing incoming log + index rebuild works.

---

## Open questions (need decisions)
1. `incoming.log` storage format:
   - Simple bincode records (fast, but larger)?
   - Optional compression (zstd) per record?
2. Compaction scheduling:
   - Immediate when shard queue empty?
   - Background worker with concurrency limit?
3. Shard read preference:
   - Always consult `incoming.log` first (safest), or only if `sorted=false`?
4. Should `present.bitset` be memory-mapped for speed?
5. `shard.json` update cadence:
   - update every block, or batch updates?

---

## Next steps (if approved)
1. Add schema v2 scaffolding + shard metadata/bitset types in storage module.
2. Implement incoming log + index for unsorted appends.
3. Implement compaction to NippyJar segments.
4. Wire read path to check bitset + incoming + segments.
5. Update ingest pipeline to write per-shard.
6. Update scheduler to shard-aware queues + rare queue parallelization.
