# Sharding Refactor Plan (10k block shards)

## Goal

Replace the current **single contiguous append-only static-file store** with a **range-sharded storage layout**:

- Shards are aligned by \( \text{shard\_start} = \lfloor n / 10000 \rfloor \cdot 10000 \).
- Each shard tracks **block presence** via a **bitset** (full bundle only).
- Shards can be **partially filled out-of-order**, and later **rewritten (“sorted/compacted”)** after we stop adding new blocks to that shard for a run.
- This enables:
  - Backfilling older ranges into an existing DB without “start_block is storage-affecting” constraints.
  - Localizing “rare/missing block” pain to a shard instead of stalling the entire DB.
  - Future node-to-node sync via shipping **compressed shard packs**.

This doc is intentionally specific about storage layout and the ingestion/scheduler implications, but defers node-to-node shard sync protocol details to roadmap.

---

## Current constraints (why we’re doing this)

Today storage is a single set of static segments under `data_dir/static/` written via `reth_nippy_jar` in append-only mode. Two critical properties:

- Segment writers are append-only and track a `next_block` cursor.
- The DB writer only commits a **contiguous prefix** (global contiguity constraint).

Result: one missing block can stall all later writes, causing huge buffering and backpressure pathologies.

Sharding changes the unit of contiguity and compaction from “whole range” to “per 10k shard”.

---

## Shard model

### Definitions

- **Shard size**: `SHARD_SIZE = 10_000` blocks (configurable later, but hardcoded for v1).
- **Shard start**: `shard_start = (block_number / SHARD_SIZE) * SHARD_SIZE`.
- **Shard end**: `shard_end = shard_start + SHARD_SIZE - 1`.
- **Presence**: a block is considered present only if the **full BlockBundle** is stored (header + tx hashes + tx meta + receipts + sizes; withdrawals/logs as currently handled).

---

## On-disk layout (proposed)

Under `data_dir/`:

- `meta.json` (schema v2+)
- `static/`
  - `shards/`
    - `<shard_start>/`
      - `shard.json`
      - `present.bitset`
      - `state/` (scratch files for the current run)
        - `unsorted.log` (append-only)
        - `unsorted.index` (optional; see below)
      - `sorted/` (canonical files after compaction)
        - `headers` + `.conf` + `.off`
        - `tx_hashes` + `.conf` + `.off`
        - `tx_meta` + `.conf` + `.off`
        - `receipts` + `.conf` + `.off`
        - `block_sizes` + `.conf` + `.off`

Notes:
- The `sorted/` directory is the shard artifact we can later package/share.
- `state/` contains run-local and/or crash-recovery info for shards that are currently being populated.
- We will keep shard paths stable and deterministic to support sharing and partial replenishment.

---

## Shard metadata (`shard.json`)

Per-shard metadata should include:

- `schema_version`: u64 (shard-level)
- `shard_start`: u64
- `shard_size`: u32 (10_000)
- `present_count`: u32 (number of bits set to 1)
- `complete`: bool (derived: `present_count == shard_size`)
- `sorted`: bool (whether `sorted/` reflects the currently known set for this shard)
- `last_updated_ms`: u64 (optional; for diagnostics)
- `run_generation`: u64 (optional; monotonic counter for “last compaction of this run”)

Key rule:
- We should never need to scan the entire bitset to decide completeness:
  - `complete` is equivalent to `present_count == shard_size`.

---

## Presence bitset (`present.bitset`)

Binary bitset of length `shard_size`.

- Bit index: `i = block_number - shard_start` (0..9999).
- 1 = present, 0 = missing.
- On write, only increment `present_count` when flipping 0→1.

Crash safety:
- Update ordering should ensure `present.bitset` is consistent with `sorted/` and/or `unsorted.log`.
- For correctness, treat bitset as source-of-truth for “is block usable”.

---

## Write strategy: “append unsorted, rewrite sorted when shard is done for this run”

User-selected approach: do not maintain multi-run files; instead:

1. While ingesting blocks for a shard, append bundles to `state/unsorted.log` (append-only).
2. When we decide we are “done adding new blocks to this shard for this run”, perform a **shard rewrite**:
   - Read all available blocks for the shard (from previous sorted state + unsorted additions),
   - Create a fresh `sorted/` set of segment files, written in shard order, using empty placeholders for missing blocks if we choose to allow “sorted but incomplete”.
   - Update `shard.json` (`sorted=true`, update `present_count`, maybe `complete`).
   - Reset/clear `state/unsorted.log` (or rotate it with `run_generation`).

### Important nuance: sorted-but-incomplete

We may want to mark `sorted=true` even if not complete, because:
- We can read from it efficiently (row-by-number semantics),
- And we can share “partial but run-stable” shards once the run ends.

However:
- Because `reth_nippy_jar` is append-only and cannot fill empty rows later, any future additions require **another full rewrite** of that shard’s `sorted/` files.
  - This is acceptable as long as shard size is small enough (10k), and we rewrite infrequently (end-of-run or shard-queue-drained).

### Unsorted storage format for `unsorted.log`

We need a simple append-only record format. One option:

- Record = `(block_number: u64, encoded_block_bundle: Vec<u8>)`
- Encoding: bincode (reuse existing `encode_bincode_value` / compat encoders as needed)
- The log may contain duplicates (if a block is re-fetched); during compaction, last-write-wins or first-write-wins (prefer last-write-wins).

To avoid O(n) scan per lookup during rewrite:
- Maintain `state/unsorted.index` mapping `block_number -> file_offset` (or record index).
- Alternatively, accept linear scan during rewrite (bounded by “blocks appended for this shard in this run”), which is acceptable if we only rewrite once per shard and shard size is 10k.

Recommendation:
- Start with **linear scan on rewrite** (simpler).
- Add the index file later if rewrite cost becomes measurable.

---

## Scheduling / ingestion refactor implications

### 1) Split work by shard from the start

Instead of a single global `pending` heap of blocks:

- Build shard queues for the target range:
  - For each shard touched by `start..end`, create `Vec<u64>` (or range slices) for blocks in that shard.
  - Prefilter each shard queue by `present.bitset` so we never schedule blocks we already have.

This allows:
- Knowing when a shard is “done for this run”:
  - when its shard queue is empty AND no in-flight blocks remain for that shard.
- Triggering shard rewrite/compaction deterministically at the right time.

### 2) Rare/missing handling becomes shard-local

We should replace the current “failed” semantics with “rare” semantics:

- A “rare block” is one that is missing after repeated attempts.
- Rare blocks should be tried in parallel with normal work (budgeted).
- Rare blocks should be attempted across many peers (fanout), not serially.

Sharding makes it safe to keep making progress on other shards while one shard has rare gaps.

### 3) Lookahead semantics change

Current lookahead is tied to a global contiguous writer watermark.

In sharded storage, lookahead becomes:
- “How many blocks we are willing to have ‘in-flight or buffered’ across all shards”
- Better expressed as a **bytes budget** or “max buffered bundles”, not a strict block-number window.

We can keep a block-based cap initially, but long-term:
- Estimate memory cost from payload sizes / moving average and cap by bytes.

### 4) When to rewrite/compact shards

We will trigger rewrite when:

- **Shard queue becomes empty for this run**, i.e. we have no more scheduled blocks to fetch for that shard (after prefiltering and after any rare retries we plan to do in this run).

Additionally:
- If we only fetched a small middle range, we can still rewrite (sorted-but-incomplete) so the shard becomes shareable/stable at end of run.

We need to decide:
- whether we rewrite immediately when shard queue drains, or batch rewrites (e.g., one at a time to avoid IO spikes).

Recommendation:
- Rewrite shards one at a time with a small semaphore (e.g. 1–2 concurrent compactions).

---

## Read path / RPC semantics

### `eth_blockNumber`

User decision: return **max present block** (not “highest contiguous”).

Implication:
- The system can report a higher block number even if there are holes below it.
- `eth_getBlockByNumber` should return “not found” for missing blocks.

### Range reads

Range reads should:
- route per shard,
- filter missing rows via bitset (or by empty rows in sorted files),
- return only present blocks.

---

## Migration

User decision: no migration needed now; benchmarks-only.

Practical outcome:
- We can bump storage `SCHEMA_VERSION` to 2 and reject opening schema 1.
- Or gate by config flag.

---

## Implementation staging (recommended order)

### Stage 0 — Spec + schema constants

- Add `SHARD_SIZE = 10_000` constant.
- Bump `SCHEMA_VERSION` to 2.
- Extend `meta.json` to include shard settings.

### Stage 1 — Shard metadata + bitset library

Implement:
- `ShardId` helpers: `shard_start_for(block)`, `shard_range(shard_start)`.
- Bitset read/write and atomic “set bit if unset” with `present_count`.
- `shard.json` load/store.

### Stage 2 — Sharded storage read API

Add a new storage layer that can answer:
- `has_block(n) -> bool`
- `max_present_block() -> Option<u64>`
- `block_header(n)`, `block_receipts(n)`, etc by routing to shard.

Start by requiring `sorted/` to exist for reads (simplest), then extend to read from `unsorted.log` if needed.

### Stage 3 — Sharded writer (ingest)

Replace the global db-writer contiguous prefix model with:
- A shard router + per-shard append-to-`unsorted.log`
- Per-shard bitset update on successful write
- Compaction trigger when shard queue drains

### Stage 4 — Scheduler refactor (shard-first)

Replace global heap with:
- Shard queues + peer assignment policy
- Prefilter queue by shard bitsets
- Rare queue parallelism + fanout (future work but should be planned alongside)

### Stage 5 — Tests + benchmarks

Add:
- unit tests for bitset + metadata correctness
- “backfill older range into existing DB” integration test:
  1) write blocks in shard middle, compact
  2) later write earlier blocks, compact again
  3) confirm reads show union

---

## Future queueing/fetching model with shards (high-level)

Scheduler becomes a “work coordinator” across shard queues:

- Normal work:
  - choose a shard with remaining missing blocks
  - assign a consecutive run from that shard to a peer (still desirable for network efficiency)
- Rare work:
  - maintain a set of rare blocks per shard
  - schedule rare blocks with fanout and per-peer dedupe
  - do not let normal queue starve rare queue or vice-versa

Backpressure:
- should be based on inflight/buffered **bytes**, not a global “next commit block”.

---

## Open questions / decisions still needed

1) **Read semantics while a shard is unsorted**
   - Do we need RPC to read blocks that are only in `state/unsorted.log`?
   - Or is it acceptable that RPC only sees blocks after shard rewrite?

2) **Compaction write format**
   - During rewrite, do we write a full 10k row set with empty rows for missing blocks?
   - Or do we write only present contiguous ranges into `sorted/` and keep an index?
   - (Full 10k rows is simplest; index-only is smaller but more complex.)

3) **Rewrite frequency**
   - Only when shard queue drains?
   - Also when memory pressure rises?
   - Also when “unsorted log grows beyond X MB”?

4) **Duplicate records in `unsorted.log`**
   - Last-write-wins vs first-write-wins?
   - (Recommend last-write-wins.)

5) **Crash recovery**
   - On restart, do we replay `unsorted.log` into bitset/metadata?
   - Or do we require `unsorted.log` to be ignored unless the run completes?

6) **Max present block computation**
   - Maintain global counter in `meta.json` updated on each successful block?
   - Or compute by scanning shard metadata on startup (fast) and track incrementally in-memory?

