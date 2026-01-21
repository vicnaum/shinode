## Historical Sync Refactor Spec (Node vs Harness parity)

### Context / problem statement
We currently have two historical-fetching implementations:
- **`harness/`**: a lightweight multi-peer probe that fetches **headers + receipts** and measures throughput (observed ~1200 blocks/sec).
- **`node/`**: a stateless history node that fetches and ingests data (observed far slower). Even when `--debug-receipts-only` is enabled to “mimic harness”, it still does significant per-block work and DB writes.

This spec defines a refactor to:
- **Match harness-like throughput for historical fetching** by adopting harness’s peer-driven scheduling and receipt-request strategy.
- **Cleanly split “fetch” from “process/write”** so performance issues can be isolated and each stage can be tuned independently.
- Replace `--debug-receipts-only` with a **true benchmark/probe mode** that is actually comparable to harness (no DB, no chain tracking, no bloom, no RPC, exit after range).

---

### Goals (what “done” means)
- **Benchmark parity**: `node --benchmark probe ...` achieves the same *order of magnitude* throughput as `harness/` on the same machine/peer set.
- **True probe behavior**: in benchmark/probe mode:
  - no DB writes
  - no checkpoint updates
  - no chain tracking / reorg logic
  - no bloom computation
  - no signer recovery / heavy derivations
  - **no RPC server**
  - exits when the configured range is fully fetched (no follow mode)
- **Architecture**:
  - aim at the full rewrite of the current ingestion system from scratch (keeping it as backup until done, but not referencing it's methods)
  - fetching and processing are separate modules/stages with explicit contracts
  - the implementation **must** be a pipeline of independent workers connected by **bounded queues/channels** (see “Exact block flow” below)
  - fetch stage computes header hashes **once** and carries them forward
  - receipt requests in probe/historical mode use **fixed-size chunking** (harness style), not “request big and recursively split on partial” as the primary strategy
- **Notes**:
  - Refer to `harness/` for how block/receipt ingestion is done
  - DO NOT reference existing ingestion system (it is slow and should be deprecated in the future)
  - The existing ingestion code may remain as a temporary fallback for safety, but it must not be used as a design reference.

---

### Non-goals (this refactor pass)
- Live “follow” mode (continuous syncing) and reorg handling.
- New RPC methods, WS, trace/stateful calls.
- Full per-stage profiling benchmark (can be a follow-up once probe exists).
- Replacing Reth networking primitives (we reuse existing network plumbing).

---

### Key observed differences (why node is slow today)
These are the root causes this refactor must address:

- **In-order processing stall** (`next_expected` gating):
  - Node buffers fetched chunks and only processes the chunk starting at `next_expected`.
  - A single slow chunk stalls all later chunks, even if already fetched.

- **Single-peer per chunk + sequential retries/backoff**:
  - Node tends to pick one peer for a whole range chunk and retries sequentially with backoff.
  - This prevents effective multi-peer utilization and amplifies slow peers.

- **Receipt request strategy mismatch**:
  - Node can request receipts for many hashes and then split recursively when peers return partial results.
  - In the real world, partial responses can make this degrade into many requests and cause major slowdown.
  - Harness uses **fixed receipt request sizes** (e.g., 16 hashes/request) and treats missing as failure/retry/escalation.

- **“Receipts-only” is not “probe”**:
  - Current node “receipts-only” path still:
    - inserts into chain tracker,
    - computes log bloom,
    - writes to DB
  - Harness does none of that on the hot path.

- **Duplicate header hashing**:
  - Node computes header hashes in fetch (to request receipts) and again in processing.
  - Must compute once and carry forward.

---

### Design overview (target architecture)

#### High-level flow
We create a historical subsystem that can run in either:
- **Benchmark/probe mode**: fetch-only + stats, no DB, no RPC.
- **(Future) historical ingest mode**: same fetch engine, but processing/writing enabled.

Pipeline stages:

1) **Range planner** (produces block numbers)
2) **Peer-driven scheduler** (assigns batches to ready peers, head-aware)
3) **Fetch workers** (headers + receipts using fixed receipt chunking)
4) **Processing workers** (probe: count + stats; ingest: derive + build bundles)
5) **Sink / writer** (probe: noop; ingest: DB)

Key property: **No “next_expected” stall in fetch**. Ordering (if ever needed) is handled only at the “sink/checkpoint” layer, and for historical-only it can be avoided entirely.

#### Exact block flow (queues + workers) — REQUIRED
This is the exact architecture shape the implementation must follow. If the implementation does not match this pipeline (separate stages + queues + worker pools), it is not acceptable.

**Block flow overview:**

```
range -> enumerate block numbers -> scheduler.pending_blocks
  -> (ready peer) -> assign FetchBatch -> fetch task
  -> fetched_blocks_tx -> processing worker pool
  -> processed_blocks_tx -> sink (probe) OR db_writer (ingest)
```

**Step-by-step (what happens to each block):**
- **1) Plan range**
  - Compute the range once (same flags as normal sync).
  - Sample `head_at_startup` once at the start.
  - Enumerate all block numbers in the target range and insert into scheduler pending state.

- **2) Schedule work**
  - Peers are fed through a `ready_peers_rx` channel (harness model).
  - When a peer becomes ready, scheduler selects the next consecutive batch for that peer:
    - up to `blocks_per_assignment` blocks (default 32)
    - only consecutive blocks
    - only blocks `<= peer_head`

- **3) Fetch**
  - A fetch task for `(peer, batch)` does:
    - request headers for the batch
    - compute `header_hash` once per header
    - request receipts using fixed-size chunking (default 16 hashes/request)
  - The fetch task emits **one `FetchedBlock` per block** into `fetched_blocks_tx`.

- **4) Process**
  - A processing worker pool consumes `FetchedBlock` from `fetched_blocks_rx`.
  - Probe mode:
    - counts receipts + records timing + peer attribution
    - emits a minimal record into `processed_blocks_tx` (or directly into a stats sink, but still via a queue)
  - Ingest mode (future):
    - computes bloom/derived logs/indexes/etc
    - emits `ProcessedBlock` into `processed_blocks_tx`.

- **5) Persist (ingest mode only)**
  - A dedicated DB writer consumes `ProcessedBlock` from `processed_blocks_rx`.
  - It buffers processed blocks and writes them in **batched DB transactions** with flush triggers.

**Backpressure (must implement):**
- `fetched_blocks_tx` is **bounded** so processing can throttle fetchers (prevents unbounded memory growth).
- `processed_blocks_tx` is **bounded** so DB writing can throttle processors.
- End-of-run must cleanly drain:
  - stop scheduling new fetches
  - close `fetched_blocks_tx` when fetch tasks complete
  - processors drain and then close `processed_blocks_tx`
  - sink/db_writer drains and flushes (if ingest) then exits

---

### CLI + range semantics (as requested)

#### New flags
- Add `--benchmark <mode>` where `<mode>` includes:
  - `probe` (initial + required)

#### Deprecations
- Deprecate and completely remove `--debug-receipts-only` and all it's leftovers.

#### Range selection (NO new benchmark-specific range flags)
Benchmark uses the **same** range selection as normal sync, with one difference: **it exits when the range is complete** (no head follow).

Range rules:
- If `--start-block` is present:
  - range = `start_block..=head_at_startup` (or `..=end_block` if end is present)
- If `--end-block` is present as well:
  - range = `start_block..=end_block`
- If both are absent:
  - range = `0..=head_at_startup`

Notes:
- `head_at_startup` must be sampled once at the beginning of the benchmark run.
- Benchmark must also respect the existing “historical safety window” semantics (e.g., `rollback_window`) if currently used in node’s historical range selection:
  - If node normally avoids the last N blocks for safety, benchmark should do the same (consistent behavior).

#### RPC behavior in benchmark
- **No RPC server in benchmark**, regardless of `--rpc-bind` or other RPC-related flags.
- It’s acceptable to still parse RPC flags (for config compatibility), but they must be ignored when `--benchmark probe` is active.

---

### New modules / file layout (minimal, focused, testable)
Create a dedicated historical subsystem under sync:

```
node/src/sync/historical/
  mod.rs          # orchestrator entrypoints
  types.rs        # shared types/contracts
  scheduler.rs    # peer-driven scheduler + retry/escalation + peer health
  fetch.rs        # headers+receipts fetching for a peer batch
  process.rs      # probe processor (and later ingest processor)
  sink.rs         # Noop sink (and later Db sink)
  stats.rs        # aggregation + JSON summary output
```

We intentionally do **not** create a separate top-level `benchmark/` module unless needed; benchmark is a mode of historical sync.

---

### Core types and contracts

#### Scheduler inputs/outputs
- `FetchBatch`
  - `blocks: Vec<u64>` (consecutive block numbers)
  - `mode: FetchMode` (optional: `Normal | Escalation`)
- Scheduler must be **head-aware**:
  - it may only hand out blocks `<= peer_head`
  - it should prefer consecutive sequences for each batch

#### Fetch output
- `FetchedBlock` (probe-capable)
  - `number: u64`
  - `peer_id` (and/or peer key)
  - `eth_version`
  - `header`
  - `header_hash` (**computed once**)
  - `receipts` (or in probe mode, optionally `receipt_count` to reduce memory)
  - `timing`:
    - header request time
    - receipts request time
    - total per batch/per block

#### Processing mode
- `ProcessingMode`
  - `Probe`:
    - must not compute bloom
    - must not insert into chain tracker
    - must not write to DB
    - should count receipts and emit stats
  - (future) `Ingest`:
    - can compute bloom, build bundles, write to DB

#### Sink contract
- `DataSink` trait:
  - `NoopSink` for probe (only consumes stats)
  - (future) `DbSink` for ingest (writes bundles)

#### DB writer batching (ingest mode only)
Batching DB writes makes sense and is very likely to speed up ingest mode.

Why it helps:
- MDBX has meaningful overhead per write transaction/commit.
- Writing block-by-block often becomes **commit-bound**.
- Batching multiple blocks into one write transaction reduces the number of commits and amortizes per-tx overhead.

**Required writer behavior (ingest mode):**
- Exactly **one** DB writer task is responsible for MDBX write transactions (single-writer model).
- The writer owns an in-memory buffer of `ProcessedBlock`s and flushes it when:
  - **Size trigger**: `buffer.len() >= db_write_batch_blocks` (recommended default: 32)
  - **Time trigger** (optional but recommended): `db_write_flush_interval_ms` elapses since last flush
  - **Drain trigger**: end-of-range / shutdown (flush remainder even if < 32)
  - **Explicit flush**: orchestrator sends a “flush now” signal at end-of-run

**Correctness constraints (ingest mode):**
- Must flush partial batches at end-of-range.
- Must not stall fetch directly; DB bottlenecks propagate only via bounded `processed_blocks_tx`.

---

### Receipt request policy (must match harness behavior)
This is the most important “make node behave like harness” rule.

In `benchmark=probe`:
- For a batch, compute `hashes` from headers.
- Request receipts in **fixed-size chunks**:
  - `receipts_per_request` default should match harness (`16`) unless overridden by config.
- If a peer responds with:
  - fewer receipts than requested, or
  - receipts that don’t match the requested hashes (ordering/association mismatch),
  then treat it as a failure for the missing subset, requeue those blocks, and record peer failure.

Important: do **not** use “request all hashes then recursively split on partial” as the primary strategy in probe. That strategy can explode requests and destroy throughput when peers commonly return partial results.

---

### Scheduler requirements (peer-driven, harness-like)
The scheduler is the engine that ensures multi-peer utilization.

#### Ready-peer model
- Maintain a channel of **ready peers**.
- When a peer is ready, scheduler assigns it a batch immediately.
- After the peer completes a batch, it is re-queued as ready (unless banned/cooldown).

#### Work selection
- Pending blocks are stored in an ordered structure (e.g. min-heap).
- Batch selection for a peer:
  - takes up to `blocks_per_assignment` blocks (default 32)
  - blocks must be consecutive
  - blocks must be `<= peer_head`

#### Retries and escalation
- Track attempts **per block** (not per chunk).
- On failure for a batch:
  - requeue only the blocks that failed
  - increment per-block attempt counters
  - optionally track which peers already tried each block (to prefer other peers first)
- Stop conditions:
  - either “max attempts per block” (and then mark failed), or
  - “tried on all peers” (if implementing tried-peer tracking)

#### Peer health
- Track consecutive failures per peer.
- Soft-ban a peer after a threshold:
  - `peer_failure_threshold` (e.g. 5)
  - ban duration `peer_ban_secs` (e.g. 120s)
- Banned peers should be re-queued only after ban expires.

---

### Stats and output (similar summary JSON is enough)

#### Required end-of-run JSON summary
Emit a single human-formatted JSON summary object at the end, including at minimum:
- `mode`: `"probe"`
- `range`:
  - `start_block`
  - `end_block`
  - `head_at_startup`
  - `rollback_window_applied` (boolean)
- totals:
  - `blocks_total`
  - `blocks_succeeded`
  - `blocks_failed`
  - `receipts_total`
- performance:
  - `elapsed_ms`
  - `blocks_per_sec_avg`
  - `receipts_per_sec_avg`
- peer stats (at least counts):
  - `peers_used`
  - `peer_failures_total`
  - optional per-peer breakdown (throughput, failures)
- latency summary (recommended):
  - `headers_ms_p50`, `headers_ms_p95`
  - `receipts_ms_p50`, `receipts_ms_p95`
  - `batch_total_ms_p50`, `batch_total_ms_p95`

---

### Implementation plan (checkboxes, execution-ready)

#### Phase 1 — CLI and config plumbing (no behavior change yet)
- [x] **Add `--benchmark` flag** to `node/src/cli/mod.rs`
  - [x] Define `BenchmarkMode` enum with `disabled | probe`
  - [x] Add `benchmark: BenchmarkMode` to config
- [x] **Add `--end-block` flag** (optional)
  - [x] Update config docs/help text to explain range semantics
- [x] **Deprecate and delete `--debug-receipts-only`**
  - [x] Delete the flag from CLI
  - [x] Cleanup all the leftovers from code
- [x] **Ensure benchmark uses same range flags**
  - [x] Implement range computation function used by both normal and benchmark paths:
    - `compute_target_range(start_block_opt, end_block_opt, head_at_startup, rollback_window) -> RangeInclusive<u64>`

#### Phase 2 — Benchmark entrypoint (no RPC, exit after range)
- [x] In `node/src/main.rs`, add early branch:
  - [x] If `benchmark == probe`:
    - [x] do not start RPC server
    - [x] sample `head_at_startup`
    - [x] compute range using existing flags
    - [x] run historical probe runner
    - [x] print final formatted summary JSON
    - [x] exit

#### Phase 3 — Add historical subsystem skeleton
- [x] Create new directory `node/src/sync/historical/`
- [x] Add `types.rs`
  - [x] Define `FetchedBlock`, `FetchBatch`, `BenchmarkConfig`, `ProcessingMode`, and stats types
- [x] Add `stats.rs`
  - [x] Define aggregator that can:
    - [x] record per-batch timings and per-block receipt counts
    - [x] compute rate averages
    - [x] compute simple percentiles (p50/p95) for key timings
    - [x] produce final summary JSON object
- [x] Add `mod.rs`
  - [x] define `run_benchmark_probe(...) -> Result<()>`

#### Phase 4 — Implement scheduler (peer-driven harness model)
- [x] Create `scheduler.rs`
  - [x] Data structures:
    - [x] pending blocks (min-heap or ordered set)
    - [x] per-block attempts counter
    - [x] completed set
    - [x] failed set
    - [x] in-flight bookkeeping (optional but recommended)
    - [x] peer health map (consecutive failures + banned_until)
    - [ ] (optional) tried-peers per block for escalation correctness
  - [x] Implement:
    - [x] `next_batch_for_peer(peer_key, peer_head) -> Vec<u64>` (consecutive, head-aware)
    - [x] `mark_completed(blocks)`
    - [x] `requeue_failed(blocks, reason)`
    - [x] `record_peer_success(peer_key)`
    - [x] `record_peer_failure(peer_key)` + ban logic
    - [x] `is_done()`
- [x] Unit tests for batching, head gating, retries, bans

#### Phase 5 — Implement fetcher (headers + receipts with fixed receipt chunking)
- [x] Create `fetch.rs`
  - [x] Implement `fetch_probe_batch(peer, blocks, receipts_per_request) -> Result<Vec<FetchedBlock>>`
  - [x] Requirements:
    - [x] request headers for the batch in one call (ascending)
    - [x] compute `header_hash` once per header
    - [x] request receipts in chunks of size `receipts_per_request` (default 16)
    - [x] validate count/mapping; treat partial as error for missing subset
    - [x] return per-block and/or per-chunk timing info for stats
  - [x] Eliminate duplicate hashing:
    - [x] ensure processing never recomputes `header_hash` in probe mode

#### Phase 6 — Orchestrate probe run (ready peers + concurrent tasks)
- [x] Create `process.rs`
  - [x] Implement `ProbeProcessor`:
    - [x] counts receipts
    - [x] records timing and peer attribution into stats
    - [x] does not allocate/store large receipt data beyond what’s needed (optional optimization)
- [x] Create `sink.rs`
  - [x] Implement `NoopSink` (probe only)
- [x] Implement the orchestrator loop in `historical/mod.rs`
  - [x] Maintain a ready-peer channel
  - [x] Create the pipeline queues (bounded):
    - [x] `fetched_blocks_tx/rx` (fetch tasks -> processing workers)
    - [x] `processed_blocks_tx/rx` (processing workers -> sink)
  - [x] Spawn the processing worker pool:
    - [x] each worker consumes `FetchedBlock` from `fetched_blocks_rx`
    - [x] outputs `ProbeRecord`/`ProcessedBlock` into `processed_blocks_tx`
  - [x] Spawn the sink consumer:
    - [x] consumes `processed_blocks_rx` and aggregates stats
  - [x] For each ready peer:
    - [x] ask scheduler for next batch
    - [x] spawn a task to fetch the batch from that peer
    - [x] on success:
      - [x] send `FetchedBlock`s into `fetched_blocks_tx`
      - [x] mark completed blocks in scheduler
      - [x] record peer success + timings
    - [x] on failure:
      - [x] requeue failed blocks (per-block attempts, not per-chunk)
      - [x] record peer failure + ban/cooldown if needed
      - [x] record failure reason in stats
    - [x] requeue peer when done (or after cooldown)
  - [x] Exit/drain conditions:
    - [x] when scheduler is done, stop dispatching new fetch work
    - [x] wait for in-flight fetch tasks to finish
    - [x] close `fetched_blocks_tx` so processors can finish draining
    - [x] wait for processors to finish, then close `processed_blocks_tx`
    - [x] wait for sink to finish, then print final formatted summary JSON

#### Phase 7 — Tests (must be part of the refactor, not optional)
- [x] Unit tests for scheduler (`scheduler.rs`)
  - [x] consecutive batch selection
  - [x] head gating (no blocks above peer head)
  - [x] retry increments per-block attempts
  - [x] max attempts marks blocks failed
  - [x] peer ban triggers and expires
- [x] Unit tests for receipt request chunking (`fetch.rs`)
  - [x] uses fixed chunk size
  - [x] partial receipts are treated as failure (requeue), not infinite split behavior
- [x] Probe safety tests
  - [x] probe mode does not call DB write functions
  - [x] probe mode does not update checkpoints

#### Phase 8 — Replace/remove old “debug receipts only” behavior
- [x] Ensure `--debug-receipts-only` is either:
  - [x] fully removed
- [x] Update docs (`README.md`) to:
  - [x] document `--benchmark probe`
  - [x] document new `--end-block` (if added)

#### Phase 9 — (After probe parity) rewrite ingest mode using the same pipeline + batched DB writer
This phase completes the “rewrite from scratch”: the production historical ingest uses the same pipeline structure and the DB-writer batching stage is introduced.

- [x] Add `ProcessingMode::Ingest` plumbing (types + config)
- [x] Implement the DB writer stage (ingest mode):
  - [x] add config knobs:
    - [x] `db_write_batch_blocks` (default 32)
    - [x] `db_write_flush_interval_ms` (optional)
  - [x] implement flush triggers:
    - [x] size trigger (>= 32)
    - [x] time trigger (optional)
    - [x] drain trigger (flush remainder at end-of-range)
    - [x] explicit flush signal
- [x] Implement `DbSink` that converts `ProcessedBlock` into storage writes
- [x] Wire normal historical sync (non-benchmark) to the new pipeline
  - [x] keep legacy path temporarily available as a safety fallback (but do not use it as a reference)
  - [x] ensure the new pipeline contains no `next_expected` / in-order stall gating anywhere

---

### Rollout / migration strategy (safe, incremental)
- Build the new historical pipeline **from scratch** modeled after `harness/` (scheduler + peer-driven workers + queues).
- Benchmark/probe path should be implemented **without destabilizing** the normal ingest path.
- Start by:
  - wiring benchmark probe to the new subsystem,
  - leaving existing `node/src/sync/mod.rs` ingest flow untouched.
- Only after probe achieves parity, migrate historical ingest to the new pipeline (Phase 9) and then deprecate the legacy historical ingestion code.

---

### Acceptance criteria (explicit)
- [x] **Benchmark exits after range** and does not follow head.
- [x] **No RPC in benchmark** (verify by checking it does not bind/listen).
- [x] **No DB side effects**:
  - no writes
  - no checkpoint updates
- [x] **Multi-peer utilization**:
  - uses many peers concurrently (not just one peer at a time)
- [x] **Receipt requests are fixed-size**:
  - confirmed via logs/metrics: `receipts_per_request=16` default
- [ ] **Throughput**:
  - `node --benchmark probe` reaches ≥ 80% of harness throughput on a comparable run.

---

### Risks and mitigations
- **Risk: peers returning partial receipts frequently**
  - Mitigation: treat partial as failure, requeue blocks, prefer different peers; ban flaky peers temporarily.
- **Risk: shared scheduler state contention**
  - Mitigation: keep scheduler critical sections small; avoid long work under locks.
- **Risk: drift between benchmark and ingest paths**
  - Mitigation: define shared `fetch.rs` building blocks and reuse in future ingest pipeline.

