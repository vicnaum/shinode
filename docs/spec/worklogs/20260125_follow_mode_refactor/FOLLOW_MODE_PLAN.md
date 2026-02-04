# Fast-Sync -> Follow Mode Plan (Option C)

Goal: keep the benchmark ingest pipeline as the default backfill mechanism (WAL + compaction).

- If `--end-block` is set: sync the fixed range and quit (no follow, no head tracking).
- If `--end-block` is NOT set: fast-sync to a *moving safe head* (head - rollback window),
  finalize/compact, then switch to follow mode (sorted writes + reorg rollback) and keep running.

No concurrent fast-sync + follow: fast-sync runs to a catch-up point, finalizes/compacts, then
follow starts.

## Implementation Checklist (WIP)

Status below reflects what is implemented in code on this branch (including local, uncommitted
changes), relative to this document.

### Mode Selection / High-Level Flow

- [X] If `--end-block` is set: sync fixed range and quit (no follow, no head tracking).
- [X] If `--end-block` is NOT set: fast-sync to moving safe head, finalize/compact, then switch to follow.

### Head Tracking (Only When `--end-block` Is NOT Set)

- [X] Track `head_seen` from `PeerPool.snapshot()` (`max(peer.head_number)`) and persist via `Storage::set_head_seen`.
- [X] Tail scheduling: extend the fast-sync target by enqueueing newly-reachable safe-head ranges.
- [X] Head tracker persists across fast-sync -> follow transition.

### Phase 1: Fast-Sync (WAL + Compaction)

- [X] Efficient resume: scan initial range and skip already-present blocks (shard meta fast-path; bitsets when needed).
- [X] Streaming scheduler cursor (`next_to_schedule` / `target_end`) for the whole run (missing ranges are enqueued without materializing per-block lists).
- [X] Catch-up detection: when scheduler drained and we have scheduled up to the current safe head, freeze scheduling and start finalization.
- [X] Finalization: flush writer, compact dirty shards, seal completed shards.
- [X] Compaction fan-out limited to 1 at a time.
- [X] Finalization visibility: log dirty shards + total WAL size before `compact_all_dirty()`.

### Phase 2: Follow Mode (Sorted Writer)

- [X] Follow uses authoritative head discovery (`discover_head_p2p`) rather than trusting `peer.head_number`.
- [X] RPC starts only once, when follow reaches `SyncStatus::UpToDate` for the first time.
- [X] Reorg handling: preflight -> find common ancestor -> `Storage::rollback_to` -> resync forward.
- [X] Required reorder buffer for strict in-order writes (prevents out-of-order "expected block X, got Y" crashes).
- [X] Avoid repeated short `run_ingest_pipeline()` epochs that each "finalize" (follow keeps a long-running ingest epoch with tail scheduling).

### Shutdown Behavior

- [X] First Ctrl-C requests graceful shutdown (drain + finalize). Second Ctrl-C forces exit.
- [ ] Ctrl-C during fast-sync should never enter follow mode (intended, but observed follow starting in at least one run; needs investigation).

### Observability / Logging (Add Before Further Refactors)

Goal: make it obvious (from console logs) *why* we are in a given phase and what we are waiting on.

- [X] Fast-sync -> follow boundary log (single, structured `info!`):
  - Fields: `follow_after`, `shutdown_requested`, `switch_tip`, `last_indexed`, `head_seen`,
    `safe_head`, `dirty_shards`, `total_wal_mib`.
  - Include per-step timings for finalize: `drain_workers_ms`, `flush_db_writer_ms`,
    `compact_all_dirty_ms`, `seal_completed_ms`.
- [X] Tail scheduling log (structured `debug!` or `info!` when `-v`):
  - On each appended range: `range_start`, `range_end`, `blocks_added`, `safe_head`,
    updated `total_blocks` / progress length.
- [X] Follow epoch log (structured `debug!` or `info!` when `-v`):
  - On each follow "epoch" (each `run_ingest_pipeline()` call): `range_start`, `range_end`,
    `missing_blocks`, `last_indexed`, `observed_head`.
  - Log `UpToDate` edge transition once (to explain RPC start trigger).
- [X] (After reorder buffer exists) reorder buffer gauges:
  - Periodic sample: `expected`, `buffer_len`, `min_key`, `max_key`, `max_gap`.

## Key Constraints / Motivations

- "Safe head runs away": if initial sync takes days, the safe head can advance by 10-20k blocks.
  We must keep extending the fast-sync target while running (without buffering huge payloads).
- Follow mode is different from fast-sync:
  - fast-sync: writes WAL (unsorted), then compacts to sorted
  - follow: writes sorted directly (no compaction), but must enforce strict in-order writes
- Reuse fetch/process code as much as possible; avoid two copies of similar logic.
- RPC should start only in follow mode, and only once (triggered when we become synced the first time).

## Proposed Architecture

### Head Tracking (Shared Service; only when `--end-block` is NOT set)

We want to keep `safe_head` moving during long fast-sync runs without adding extra network load.

Implementation notes (best-effort, low overhead):
- Prefer using the head signal we already have from peers:
  - When peers connect, their `Status` includes a best hash which we resolve to a `head_number`.
  - `PeerPool` maintains `peer.head_number`; this is cheap to read (`pool.snapshot()`).
- Update `head_seen` periodically (e.g. every 1s) from `max(peer.head_number)` and persist via `Storage::set_head_seen`.
- Compute `safe_head = head_seen.saturating_sub(rollback_window)`; `safe_head` is monotonic.
- In follow mode (near tip), continue using the existing verified head probing (`discover_head_p2p`)
  rather than trusting `peer.head_number` as authoritative.

Head tracking is a persistent service: it continues running across the fast-sync -> follow transition.
During fast-sync finalization we may "freeze" the target end, but we do not need to stop the tracker.

### Phase 1: Fast-Sync (WAL + Compaction) With a Moving Safe Head

Applies only when `--end-block` is NOT set.

Goal: continuously backfill until we catch up to the moving `safe_head`, while keeping memory
bounded by existing inflight/buffer caps.

1) Streaming scheduler (not precomputed list)
- Replace "Vec of blocks to fetch" with:
  - `next_to_schedule: u64` cursor
  - `target_end: u64` updated from head tracker (`safe_head`)
- Scheduler loop:
  - While `next_to_schedule <= target_end`:
    - Skip blocks already present (meta fast-path; fall back to bitsets only if needed)
    - Enqueue missing blocks
    - Increment `next_to_schedule`

2) Efficient resume behavior
- Resume (restart) is handled by skipping already-present blocks in the initial `[start_block..=target_end]`.
  - For already-downloaded shards, use shard meta ("complete/sorted/sealed/etc") as fast-path.
  - Only consult bitsets when shard meta cannot conclusively answer.
- When `target_end` grows during the same run:
  - We generally do NOT need to scan the newly appended range for "already present" blocks.
  - Rationale: the scheduler cursor is the only producer of work; if it never scheduled those blocks,
    they can't exist locally yet.
  - (If we later add speculative prefetch beyond `target_end`, revisit this assumption.)

3) Catch-up (quiescence) detection
- Consider fast-sync caught up when:
  - `next_to_schedule > target_end`
  - AND pipeline queue empty
  - AND inflight == 0

4) Fast-sync finalization boundary
- Freeze `switch_tip = target_end` (or "last processed/written" as appropriate).
- Stop extending the ingest target during finalization (ignore head tracker updates for scheduling),
  but keep head tracking running so follow starts with a fresh view of the network.

5) Fast-sync finalization actions
- Drain remaining writers/flushes.
- Compact everything needed up to `switch_tip`.
  - If `switch_tip` is mid-shard, we should compact the contiguous prefix so follow can append
    safely to sorted. (Avoid "start follow at shard boundary" unless we accept extra lag.)

### Phase 2: Follow Mode (Sorted Writer + Reorder Buffer)

Only if `--end-block` is NOT set.

1) Follow start point
- `follow_start = switch_tip + 1` (or `last_written + 1`), assuming no gaps below.

2) RPC start policy (decided)
- Do NOT start RPC during fast-sync.
- Start RPC when follow reaches "synced" for the first time (equivalent to `SyncStatus::UpToDate`),
  and ensure it is started only once (trigger/edge, not a loop).

3) Reuse fetch/process code
- Fetching + processing can be reused from fast-sync.
- The main change is the DB write mode:
  - Follow writes sorted segments directly.
  - Follow must guarantee strict in-order writes across all segments.

4) Required: in-memory reorder buffer for out-of-order arrivals
- Maintain:
  - `expected = next_block_to_write`
  - `buffer: BTreeMap<u64, BlockBundle>`
- On block arrival:
  - Insert into buffer.
  - While buffer has `expected`: pop + write, then increment `expected`.
Boundedness:
- No separate arbitrary cap is required if we ensure follow scheduling itself is bounded
  (by inflight/lookahead/max-buffered-blocks) and stays near tip.
- The reorder buffer must not grow unbounded: it should be implicitly bounded by the follow
  pipeline's scheduling window (out-of-order arrivals cannot exceed what we scheduled).

5) Reorg policy (decided)
- Follow writes to the real latest head (no artificial safe-head lag).
- On reorg:
  - Detect via `preflight_reorg()`.
  - Find common ancestor via `find_common_ancestor()`.
  - Roll back storage via `Storage::rollback_to(ancestor)` (truncate sorted tail), then re-sync forward.
- Reorg rollback depth is capped by `NodeConfig.rollback_window`; deeper reorgs stop follow with an error.

## Fixed Decisions / Assumptions

- Shards are fixed-size (10,000 blocks).
- Before switching to follow, we compact any dirty shards (including partial shards) so follow can append.
- Follow writes to the actual head and handles reorgs by rollback + rewrite (no "safe head only" writes).
- RPC starts only once, when follow reaches `UpToDate` (synced) for the first time.

## Alternative Considered (Not Primary)

Looped fast-sync epochs:
- Run fast-sync `[start_block..=safe_head]`, finalize/compact, then re-check head and repeat.
- Downside: can devolve into tiny ranges near tip (1-3 blocks) that cause repeated partial-shard compactions.
- Preferred: moving-safe-head fast-sync with a shared head tracker, then a single clean switch to follow.
