# Sharded DB Refactor SPEC (Storage v2)

This document specifies the v2 storage format refactor: from a single globally-contiguous append-only static store to **range-aligned shards** that can be filled out-of-order and compacted independently.

It is written to enable:
- **Backfilling older ranges into an existing DB** (no more “start block is storage-affecting”).
- Isolating “rare/missing blocks” to a shard instead of stalling the whole DB.
- Future **shard-level sync** between Stateless History Nodes (ship a compressed shard artifact rather than block-by-block).

This spec intentionally focuses on **storage format + compaction** and the **minimum scheduler changes needed to make it work**. Details of node-to-node shard transfer are deferred.

---

## 1) Key decisions (locked in)

- **Shard size**: `SHARD_SIZE = 10_000` blocks (configurable later; default 10k).
- **Shard alignment**: `shard_start = floor(block_number / SHARD_SIZE) * SHARD_SIZE`.
- **Presence definition**: a block is “present” iff the **full bundle** is stored (header + tx hashes + tx meta + receipts + block size; logs remain derived as today).
- **Write strategy**: during ingestion we **append unsorted** records; when a shard is “done for this run”, we **rewrite/compact** it into a sorted, canonical representation.
- **Queue prefilter**: blocks whose presence bit is already set must not be enqueued.
- **Migration**: none required (bench-only data); schema v2 expects a fresh data dir.

---

## 2) Constraints from current implementation (why compaction is required)

Current storage uses `reth_nippy_jar` segments that are:
- **append-only** (plus tail-prune for rollback)
- cannot patch “holes” in-place

Therefore, to support “write block 125000 first, later write 100000” in the same shard without losing data, we need a **staging format** that supports out-of-order arrivals, plus a **rewrite step** that produces canonical sorted shard files.

---

## 3) Glossary

- **Shard**: fixed block interval `[shard_start, shard_start + SHARD_SIZE - 1]`.
- **Shard offset**: `i = block_number - shard_start` in `[0..SHARD_SIZE)`.
- **Shard state**:
  - `sorted=false`: shard has pending unsorted staged writes
  - `sorted=true`: shard has a canonical sorted representation suitable for fast reads and shipping
  - `complete=true`: `present_count == SHARD_SIZE` (all bits set)
- **Present bitset**: binary bitmap with one bit per shard offset indicating stored bundle presence.

---

## 4) On-disk format

### 4.1 Global layout (under `data_dir/`)

```
data_dir/
  meta.json
  static/
    shards/
      24180000/                     # decimal shard_start, no zero-padding
        shard.json
        present.bitset
        state/
          staging.wal
        sorted/
          headers        (+ .conf/.off/.idx as created by NippyJar)
          tx_hashes      (+ ...)
          tx_meta        (+ ...)
          receipts       (+ ...)
          block_sizes    (+ ...)
```

Notes:
- `sorted/` is the unit we will eventually package/share.
- `state/` exists only if new blocks have been appended since the last compaction.
- Shard directory names are **decimal `shard_start`** values (no zero-padding).
- Peer cache is stored separately under `--peer-cache-dir` as `peers.json` (default: `~/.stateless-history-node/peers.json`).

### 4.2 `meta.json` (schema v2)

`meta.json` must include:
- `schema_version: 2`
- `chain_id`
- `storage_key` (retention/head_source/reorg_strategy; same compatibility checks as today)
- `shard_size` (default 10k)
- `max_present_block: Option<u64>` (for fast `eth_blockNumber` and operator visibility)

Important change from v1:
- `start_block` is **not storage-affecting** anymore. It becomes an ingest configuration, not a schema constraint.

Shard inventory note:
- We do **not** store a full list of shards (or their hashes) in `meta.json` in v1.
- The canonical source of truth for shard existence/state is the `static/shards/<shard_start>/` directory tree.
- Operator tooling can derive “how many shards exist / complete / sealed” by scanning `shard.json` files (cheap at ~3k shards for a 30M-block chain).

### 4.3 `shard.json` (per shard metadata)

Minimal fields:

```json
{
  "format_version": 1,
  "shard_start": 24180000,
  "shard_size": 10000,
  "present_count": 5821,
  "complete": false,
  "sorted": false,
  "sealed": false,
  "tail_block": 24189998,
  "content_hash": null,
  "content_hash_algo": "sha256"
}
```

Rules:
- `complete` MUST be equivalent to `present_count == shard_size` (no full-bitset scan required).
- `sorted=false` means `state/` contains staged writes that are not yet folded into `sorted/`.
- `tail_block` is the highest block number materialized in `sorted/` (i.e., `sorted/` contains rows for `[shard_start..=tail_block]`).
  - This allows **live-follow** to keep appending in-order without having pre-written trailing empty rows.
- `sealed=true` means the shard is **stable** (no further writes expected unless an explicit future backfill targets it).
- `content_hash` is populated only when `sealed=true` (see hashing below).

Determinism note:
- `shard.json` should contain only stable, content-derived fields (no timestamps) so sealed shards can be reproducibly identified across machines.

### 4.4 `present.bitset`

- Fixed size: `SHARD_SIZE` bits (10,000 bits = 1,250 bytes).
- Bit `i` corresponds to block `shard_start + i`.
- `1` = present full bundle, `0` = missing.
- The bitset is stored for **all shards**, including `complete=true` shards (where it is expected to be all-ones).
  - Rationale: simpler code paths (no special-casing), cheap size, and it remains useful for validation and for future backfill/reorg logic.

Runtime caching (important):
- The node should **not** read the bitset from disk on every check.
- On startup, load shard bitsets into memory (either eagerly for all shards, or lazily per shard and cache).
- During a run, the **in-memory bitset** is the source of truth for scheduling decisions and write de-duplication.
- The on-disk `present.bitset` is a persisted checkpoint and should be flushed:
  - always at shard compaction/seal and graceful shutdown, and
  - optionally periodically during long runs to reduce restart/recovery work.

### 4.5 `state/staging.wal` (staging WAL)

Append-only **binary** write-ahead/staging log of “new bundle writes since last compaction”.

Record format (v1):
- `block_number: u64` (LE)
- `bundle_len: u32` (LE)
- `bundle_bytes[bundle_len]` (bincode of a WAL record payload; see note below)
- `crc32: u32` (LE) over `(block_number || bundle_len || bundle_bytes)` for tail truncation detection

Notes:
- Duplicate records are allowed (should be rare because we pre-check the bitset). On compaction, “last write wins” for safety.
- This is not “logs” in the human-readable sense; it is a record-oriented binary file. (`.wal` is used to avoid confusion.)
- Implementation note: payload bytes are stored in a format that is stable and easy to replay/compact (pre-encoded per-segment rows).

### 4.6 `sorted/` canonical shard segments

Each shard’s canonical data is stored in the same logical “segments” we have today, but scoped per shard:
- `headers` (bincode compat)
- `tx_hashes` (bincode)
- `tx_meta` (bincode, zstd)
- `receipts` (bincode compat, zstd)
- `block_sizes` (u64 LE)

Canonical row addressing:
- In `sorted/`, row index is the **shard offset** (for offsets that exist on disk).
- Missing blocks (within `[shard_start..=tail_block]`) are represented by an **empty row** (same semantics as current `SegmentWriter` gap-fill).
- Rows beyond `tail_block` are not present yet and will be appended later (for live-follow).

This makes “sorted reads”:
- fast (direct row-by-number)
- simple (bitset says present; empty row yields None if bitset says missing or metadata drift)

### 4.7 Shard content hashing (integrity + future distribution)

We want a stable identifier to verify shard integrity and to support future distribution (P2P, torrents, etc.).

When a shard becomes **sealed**, we compute and persist `content_hash` in `shard.json`.

Hash input definition (v1):
- `content_hash_algo`: `"sha256"`
- `content_hash` is computed over a deterministic concatenation of:
  1. a domain separator string: `stateless-history-shard-v1\n`
  2. `shard_start` (u64 LE)
  3. `shard_size` (u32 LE)
  4. `tail_block` (u64 LE)
  5. `present.bitset` bytes (exactly 1250 bytes)
  6. every file in `sorted/`, in lexicographic order by relative path, with:
     - path bytes + `\0`
     - file length (u64 LE) + `\0`
     - raw file bytes

Notes:
- The hash explicitly **does not** include `shard.json` to avoid circularity.
- For **complete** shards, `content_hash` is expected to be identical on every machine (given identical canonical formats).
- For **partial** shards, hashing is still a good idea as an **integrity/artifact hash**:
  - it will (correctly) differ depending on which blocks are present,
  - but it still lets us verify “did we receive/store exactly this shard artifact?” during future distribution.

---

## 5) Read semantics

### 5.0 Operational invariant (RPC/follow gating)

Per the agreed run model:
- **RPC is not started** until the ranged fast-sync is finished.
- Before starting **live-follow + RPC**, the node must ensure **all shards are sorted**.

Therefore:
- Reads in RPC mode only need to read `sorted/`.
- If the process crashes mid-fast-sync and leaves `state/staging.wal` behind, the next start must run a **repair step**:
  - compact/sort any shard that has staged data (`sorted=false` or `state/staging.wal` exists),
  - then start follow/RPC.

### 5.1 Source of truth

The presence map (bitset) is the source of truth for “is block queryable”.

Implementation note:
- Use the **in-memory** bitset for hot-path checks.
- Persist changes to `present.bitset` as described in §4.4 (compaction/shutdown + optional periodic flush).

### 5.2 `has_block(block_number)`

Returns `true` iff:
- shard exists, and
- bitset has bit set for that offset.

### 5.3 Block reads (header/tx_hashes/receipts/etc)

For any per-block read:
1. Compute shard, check bitset; if not present → return `None`.
2. If present: read from `sorted/` directly.

### 5.4 Range reads

Range reads iterate shards overlapped by the range and:
- use the bitset to skip absent blocks,
- use sorted segments for throughput.

### 5.5 `eth_blockNumber` semantics

Per current decision: return **max present block** (`max_present_block`).

Compatibility note:
- This differs from the original PRD “highest fully indexed contiguous head” interpretation.
- If we need strict indexer-compat semantics later, we can add an alternative derived metric (e.g. “highest contiguous present from some configured start”).

### 5.6 `eth_getLogs` semantics under sharded coverage (all-or-nothing)

To prevent indexers from silently skipping logs when there are holes:

- `eth_getLogs(fromBlock=X, toBlock=Y, ...)` MUST be **all-or-nothing** with respect to block availability.
- If **any** block in `[X..=Y]` is not present in storage (bit unset / shard missing), the RPC MUST return a **JSON-RPC error** (not partial results, not empty results).

Implementation rule (conceptual):
1. Determine the block range `[X..=Y]` after parsing params.
2. Verify availability for every block in the range using shard bitsets.
3. If a missing block is found, return error with:
   - a stable “server error” code (`-32001`), and
   - message indicating the range is not fully available, and
   - optional data including the first missing block number (recommended).

This ensures indexers can handle gaps explicitly (split ranges / retry later) rather than advancing checkpoints incorrectly.

---

## 6) Write semantics (ingest)

### 6.1 Fast-sync (ranged) ingestion writes

Fast-sync may fetch blocks out-of-order within a shard.

Invariant:
- The scheduler/queue builder should **not** schedule blocks that are already present (bit set).
- Therefore, “incoming duplicate blocks” should be extremely rare and treated as a no-op.

For each incoming `BlockBundle`:
1. Compute shard.
2. If the presence bit is already set → **ignore** (idempotent no-op).
   - Do not append to `state/staging.wal`.
   - Rationale: avoids mutating shards unnecessarily (including sealed shards) and prevents WAL bloat.
3. Append record to `state/staging.wal`.
4. Flip bit in `present.bitset` from 0→1 and increment `present_count`.
5. Update `max_present_block` in `meta.json` if this block is greater.
6. Update `shard.json` (`present_count`, `complete`, `sorted=false`).

### 6.2 Live-follow ingestion writes

Live-follow appends blocks in-order.

Writes go directly to `sorted/` (no staging), by appending rows at the current `tail_block + 1` position for that shard:
1. Compute shard.
2. Append rows to each `sorted/` segment at `block_number` (monotonic within shard).
3. Set presence bit, increment `present_count`.
4. Update `tail_block` (to the new block number), keep `sorted=true`.

Atomicity expectations (recommended design):
- A crash must not result in the **persisted** presence map (`present.bitset` on disk) claiming a block is present if the block data is not recoverable from either:
  - canonical `sorted/` files, or
  - `state/staging.wal`.
- Use a standard **WAL + checkpoint** pattern:
  1) append the record to `state/staging.wal` first
  2) update the **in-memory** bitset immediately after the append succeeds
  3) flush `present.bitset` + `shard.json` as a checkpoint **only after** the corresponding WAL bytes are durable (group commit is fine; do not `fsync` per block)
- `state/staging.wal` records should be self-delimiting so startup can detect incomplete writes:
  - length-prefix is required (already specified)
  - a per-record checksum (e.g. CRC32) is recommended so that on startup we can stop at the first corrupted/truncated tail record and ignore the partial tail safely.
- Startup recovery (before building queues / resuming fast-sync):
  - if `state/staging.wal` exists, replay it to rebuild the in-memory presence bitset (and `present_count`) on top of the last persisted checkpoint.
  - this avoids refetching duplicates (network is the bottleneck; replay is local IO).
- If `state/staging.wal` is irreparably corrupted, the last-resort fallback is to discard it and refetch missing blocks.

---

## 7) Compaction (rewrite/sort) semantics

### 7.1 When to compact

Compaction is the operation that produces/refreshes `sorted/` and clears `state/`.

Triggers:
- **Shard queue drained for this run** (primary): we know we will not append more blocks for that shard in this run.
- **End-of-run / graceful shutdown**: compact any shard with `sorted=false`.
- **Shard rollover for live follow**: when the canonical head moves into the next shard, compact the previous shard.
- Optional future trigger: if `state/staging.wal` grows beyond a size threshold.

Follow/RPC gating:
- Before starting **live-follow + RPC**, we must compact any shard with staged data (`sorted=false` / `state/staging.wal` exists).

### 7.2 What compaction does

Input:
- Existing canonical `sorted/` (if present)
- New staged `state/staging.wal`

Output:
- A fresh canonical `sorted/` written in shard order up to `tail_block` (see below), with empty rows for missing offsets within that range.
- `shard.json` updated to `sorted=true` (and `complete` if applicable).
- `state/staging.wal` cleared/rotated.

Algorithm (conceptual):
1. Build a map `offset -> BlockBundle` from `state/staging.wal` (last write wins).
2. Define `tail_block` for the shard:
   - `tail_block = max(shard_start + max_set_bit_offset, existing_sorted_tail_block)` (if any data exists),
   - This avoids writing trailing empty rows for future blocks and keeps the shard appendable for follow mode.
3. For each offset 0..=(tail_block - shard_start):
   - if bitset is 1:
     - choose staged bundle if present, else read from old `sorted/`
       - if both exist, staged wins (this is how we “overwrite” without in-place mutation)
   - if bitset is 0:
     - write empty row
4. Write rows for `[shard_start..=tail_block]` to new `sorted/` segments.
4. Atomic swap:
   - write to `sorted.tmp/`, then rename into place (directory swap).
5. Clear `state/`.
6. Set `shard.json.tail_block = tail_block`.

### 7.4 Sealing + hashing

After a shard is compacted, it may become **sealed** (stable), and we compute its `content_hash`:

- **Fast-sync**:
  - At the end of ranged fast-sync, compact all shards that have staged data.
  - Then mark all shards `sealed=true` and compute `content_hash` (except the current head shard if you immediately continue writing into it).
- **Live-follow**:
  - The active head shard is `sealed=false` (it changes as we append new blocks).
  - When follow advances into the next shard, compact the previous shard, mark it `sealed=true`, and compute `content_hash`.

Hashing is done as specified in §4.7.

### 7.3 Concurrency

Compaction is IO-heavy; run it with bounded concurrency (e.g. 1–2 shards at a time).

---

## 8) Reorg / rollback semantics (storage-level)

v0.1 semantics remain: **delete-on-rollback**.

Storage v2 implementation strategy:
- For each rolled-back block number:
  - perform **tail-prune** on the affected shard segments if the rollback removes the current tail (follow-mode expected case)
  - clear presence bits for all removed blocks and decrement `present_count` accordingly
  - update `tail_block` to the new tail

If rollback crosses shard boundaries:
- delete/clear any shards that are entirely above the rollback point
- prune the shard that contains the rollback point to exactly that block

We should not require an “unsorted” state for rollback; shards remain readable via `sorted/` and presence bits.

Correctness requirement:
- Cleared bit MUST prevent RPC from returning stale canonical data even if it remains on disk prior to compaction.

---

## 9) Scheduler / pipeline implications (post-storage, but must be planned now)

### 9.1 Build the initial work queue by shard

Instead of a single global `pending` heap:
- Build per-shard queues for the target range.
- Prefilter each shard queue by `present.bitset` so we never schedule already-present blocks.
- Optionally (recommended), re-check `present.bitset` right before dispatching a block/batch to a peer to avoid rare duplicates due to concurrent writers.

### 9.2 Detect “shard done for this run”

Per shard:
- queue empty AND inflight empty ⇒ shard done ⇒ trigger compaction.

### 9.3 Rare/gap handling becomes shard-local

Replace “failed” with “rare”:
- rare blocks are tried in parallel with normal work (budgeted)
- rare blocks use fanout across peers
- empty response handling:
  - empty batch with size > 1 should penalize peer but should not permanently burn blocks

### 9.4 Backpressure / lookahead

With shards, global “contiguous prefix watermark” backpressure is no longer the controlling factor.

Long-term:
- cap by **buffered bytes** or “max buffered bundles”, not by `max_lookahead_blocks`.

Short-term (transition while sharding lands):
- increase default `fast_sync_max_lookahead_blocks` to **50,000** as agreed.

---

## 10) Configuration changes (v2)

Add/adjust:
- `--shard-size` (default 10_000)
- storage schema bump: `schema_version = 2` (new data dir required)
- keep existing RPC limits

---

## 11) Test plan (minimum)

Unit tests:
- shard alignment math (block → shard_start + offset)
- bitset flip + `present_count` correctness
- unsorted log append + rebuild-from-log
- compaction produces `sorted/` with correct rows and empties

Integration tests:
- write partial shard (middle range) → compact → reads work
- later backfill earlier part of same shard → compact again → reads show union
- reorg rollback clears bits and hides data immediately

---

## 12) Open questions

None for v1 (all prior open questions resolved by this spec + your answers).

