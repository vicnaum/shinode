Got it. I’ll (a) turn our comparison into a very concrete “diff list” (what to change vs what’s wrong), then (b) propose a detailed refactor spec that cleanly splits **fetching** from **processing/writing**, and (c) define the new `--benchmark` modes (starting with a true harness-like probe that does no DB work). I’ll quickly re-open the current worktree versions of the relevant files to keep names/paths exact.

[5 tools called]

Next I’ll turn the code diff into a concrete refactor spec: what to change to reach harness behavior, what’s wrong today (with exact locations), and then a very detailed step-by-step plan for splitting “fetch” and “process/write” into separate async pipelines and introducing `--benchmark probe` to mimic harness.

[6 tools called]

## 1) What must change in `node/` to match `harness/` behavior

### A. Scheduling / parallelism (make it peer-driven, not chunk-driven)
- **Replace range-chunk concurrency with “ready peer → assign batch → requeue peer”**
  - **Harness**: peers are treated like workers; a peer becomes “ready”, gets a batch, runs it, then is re-queued (`harness/src/main.rs` ready queue + per-peer tasks).
  - **Node today**: a global `FuturesUnordered` spawns “chunk tasks” (`fast_sync_chunk_size`, default 32) and each task picks *one peer* and fetches the whole chunk; processing waits on `next_expected` (`node/src/sync/mod.rs` `run_fast_range`, `run_receipts_only_range`).
- **Add head-aware scheduling**
  - **Harness**: only assigns blocks ≤ peer head, and prefers contiguous sequences (`pop_next_batch_for_head`, `next_batch_for_peer` in `harness/src/main.rs`).
  - **Node today**: picks a peer without checking head suitability for the specific requested range.
- **Use bounded queues + backpressure instead of “buffer until next_expected”**
  - Fetching and processing should run independently with bounded queues; only the final writer may need ordering.

### B. Work-per-block must be a first-class “mode” (probe vs ingest)
- **Introduce a true “probe/benchmark mode” that does *no* ingest work**
  - **Harness** does:
    - request headers (batch)
    - compute header hash (needed to request receipts)
    - request receipts (chunked)
    - count receipts + measure timings
    - log/aggregate stats
  - **Node `--debug-receipts-only` today still does**:
    - `ChainTracker::insert_header(...)` per block
    - compute `logs_bloom(...)` per block
    - write headers+receipts to DB (`write_header_receipts_batch`)
- **Make processing toggles live in the processing pipeline**
  - Fetch stage should always produce the same “fetched record”.
  - Processing stage decides whether to:
    - do nothing (probe)
    - compute bloom (optional)
    - derive logs/indexes (full)
    - write to DB (ingest)

### C. Peer usage + retries must be per-block/batch, not “entire chunk on one peer”
- **Distribute work across peers by scheduling batches to peers (like harness)**
  - **Harness**: multiple peers concurrently each fetch their own batch.
  - **Node today**: `MultiPeerBlockPayloadSource::{blocks_by_number, receipts_only_by_number}` chooses a peer, tries it, then (if fails) tries another peer sequentially with backoff for the same *range*.
- **Retry should requeue blocks and try different peers (escalation)**
  - **Harness**: per-block attempts + escalation tries the same block across different peers.
  - **Node today**: failures are handled at range/chunk granularity, not per block.

### D. Request sizing / batching must match harness (avoid “split-on-partial explosion”)
- **Match harness fixed receipt request size** (default harness `receipts_per_request = 16`)
  - **Harness**: does `hashes.chunks(receipts_per_request)` and issues small receipt requests.
  - **Node today**: `request_receipts_chunked()` starts by requesting receipts for the *entire slice* and only splits if the peer returns partial results (`node/src/p2p/mod.rs`). If peers frequently return partial receipts, this can degrade into many requests (worst-case near per-block).
- **Make request sizes explicit configuration (esp. receipts)**
  - “Dynamic splitting” is a fallback mechanism; for throughput you want a stable, conservative request size that peers reliably answer in full.

### E. Remove duplicate hashing by carrying computed hashes forward
- **Compute header hash once and carry it through**
  - **Node today** computes header hash in fetch phase (to request receipts) and again in processing phase (for chain insertion) (`SealedHeader::seal_slow(...)` appears in both `node/src/p2p/mod.rs` and `node/src/sync/mod.rs`).
  - In a proper architecture, fetch returns `(header, header_hash, receipts, …)` and processing reuses `header_hash`.

---

## 2) Problems in node today (where time is lost)

### A. Duplicate header hashing (avoidable)
- **Problem**: `SealedHeader::seal_slow(header.clone()).hash()` is done:
  - in p2p fetch to build `hashes` (needed for receipts)
  - again in receipts-only processing before `ChainTracker::insert_header(...)`
- **Impact**: extra CPU per block, multiplied by 11k blocks.

### B. In-order processing stall (`next_expected`)
- **Problem**: both `run_fast_range` and `run_receipts_only_range` buffer completed chunks in a `BTreeMap` and only process the chunk starting at `next_expected`.
- **Impact**: one slow chunk stalls *all later chunks*, even if they were already fetched. This is exactly the “pipeline stall” harness avoids by per-peer scheduling of smallest blocks first.

### C. Single-peer per range attempt + sequential retries/backoff
- **Problem**: `MultiPeerBlockPayloadSource::receipts_only_by_number(range)` chooses a peer and fetches the whole chunk from it; on failure, it sleeps 250ms and tries another peer **sequentially**.
- **Impact**: reduces effective multi-peer throughput and makes you sensitive to picking slow peers.

### D. Receipt request strategy can explode requests on partial responses
- **Problem**: `request_receipts_chunked(peer, hashes)` starts by requesting receipts for all `hashes[start..end]`. If the peer returns fewer receipts than requested (common with some clients), it recursively splits the range and retries.
- **Impact**: in the “bad but realistic” case, you can end up issuing many requests per 32-block chunk (vs harness’ fixed 2 requests of size 16).

### E. “Receipts-only” is not “probe”
Even with `--debug-receipts-only`, node still:
- mutates canonical chain state (`ChainTracker`)
- computes bloom (`logs_bloom`)
- writes to DB (transaction + commit)
Those are **explicitly not** done in harness’ hot path, so the modes aren’t comparable.

---

## 3) Plan/spec to fully isolate & rewrite historical fetching

### Target outcome (definition of “done”)
- **Historical sync fetch pipeline is harness-style**: peer-driven scheduling, multi-peer concurrency, conservative fixed request sizing, per-block retries.
- **Fetching and processing/writing are separated**: independent queues, independent concurrency knobs, bounded memory.
- **Benchmark/probe mode exists**: harness-like fetch+count, no chain insert, no bloom, no DB.
- **Architecture is modular**: easy to swap fetch policy, processing policy, and storage policy independently.

### Proposed module split (concrete)
Create a dedicated historical subsystem, e.g.
- `node/src/sync/historical/mod.rs`
- `node/src/sync/historical/scheduler.rs`
- `node/src/sync/historical/fetch.rs`
- `node/src/sync/historical/processing.rs`
- `node/src/sync/historical/writer.rs` (can be a no-op in probe mode)
- `node/src/sync/historical/stats.rs`
- `node/src/sync/historical/types.rs`

### Core types (explicit)
- **Work units**
  - `BlockNumber = u64`
  - `FetchBatch { blocks: Vec<u64> /* consecutive */, mode: FetchMode }`
  - `FetchMode { Normal, Escalation }` (optional but recommended for harness parity)
- **Fetch outputs**
  - `FetchedBlock {`
    - `number: u64`
    - `peer_id: PeerId`
    - `eth_version: EthVersion`
    - `header: Header`
    - `header_hash: B256` *(computed once)*
    - `receipts: Vec<Receipt>` *(or receipt_count in probe mode to reduce memory)*
    - `timing: FetchTiming { headers_ms, receipts_ms }`
    - `}`  
- **Processing outputs**
  - Probe: `ProbeRecord { number, peer_id, headers_ms, receipts_ms, receipt_count }`
  - Full ingest (future): `ProcessedBlock { bundle: BlockBundle | ReceiptBundle, … }`
- **Policies**
  - `ProcessingMode`:
    - `ProbeHarnessLike` (initial)
    - `FullIngest` (later)
    - `ProfiledIngest` (later; per-stage timers)
  - `WriterMode`:
    - `Disabled` (probe)
    - `Enabled` (ingest)

### Runtime pipeline (explicit, async, decoupled)
- **Queue 1: fetch scheduling**
  - Input: target block numbers (range or anchors)
  - Scheduler assigns batches to peers as they become ready.
- **Queue 2: processing**
  - Each fetched block is sent immediately to processing (per-block, not per-chunk).
  - Processing runs with independent concurrency (CPU-bound work can be `spawn_blocking`).
- **Queue 3: writing (optional)**
  - Writer consumes processed outputs and persists (only in ingest modes).
  - Writer may preserve ordering if required, but must not block fetching directly; it only backpressures via bounded channels.

---

## 4) Brainstormed architecture: independent “block fetching” vs “block processing”

### Design principles
- **Fetching is network-bound** → scale by peers + in-flight network requests.
- **Processing is CPU/IO-bound** → scale by CPU threads and DB batch sizing.
- **Never couple fetch progress to “next_expected”** except optionally at the final writer boundary (and only if you truly need strict ordering for checkpoints).

### Concrete orchestration proposal
- **Peer Manager**
  - Maintains `PeerHandle { peer_id, messages, eth_version, head_number, peer_key }`
  - Maintains peer health (fail counters, ban deadlines)
  - Emits “ready peers” into a channel (exactly like harness)
- **Scheduler**
  - Holds a min-heap of pending blocks (`BinaryHeap<Reverse<u64>>`)
  - On `ready_peer`:
    - pick next consecutive batch ≤ peer head (`blocks_per_assignment`, default 32)
    - send `FetchBatch` to that peer worker
- **Peer Worker**
  - Fetch headers once (`GetBlockHeaders` rising)
  - Compute `header_hash` once per header
  - Fetch receipts in *fixed-size chunks* (`receipts_per_request`, default 16)
  - Emit per-block `FetchedBlock` into processing queue
- **Processing Workers**
  - `ProbeHarnessLike`: just count receipts + record timings; drop payload (no DB, no bloom, no chain)
  - `FullIngest` (later): compute bloom/derive logs/etc and build bundles
- **Writer**
  - `Disabled` for probe
  - `Enabled` for ingest:
    - optionally batch commits (e.g., 100 blocks/tx)
    - update checkpoints
    - can enforce ordering here if needed

### Why this fixes the observed slowness
- Removes **in-order stall** from the fetch stage entirely.
- Ensures **true multi-peer** utilization at the scheduler level, not “one peer per chunk”.
- Avoids **receipt request explosion** by using a conservative fixed request size (harness strategy).
- Allows probe mode to skip **all** CPU/DB work, matching harness resource profile.

---

## 5) Replace `--debug-receipts-only` with `--benchmark <type>` (spec)

### CLI changes (`node/src/cli/mod.rs`)
- **Remove**:
  - `--debug-receipts-only`
- **Add**:
  - `--benchmark <mode>` where `<mode>` is a `ValueEnum`
    - `probe` (initial, must exist)
    - `harness` (alias of `probe` initially, or reserved for “exact harness output format”)
- **Add explicit benchmark range selection** (so you can reproduce “11000 blocks” deterministically)
  - Recommended minimal set:
    - `--benchmark-start <u64>` (defaults to `--start-block`)
    - `--benchmark-end <u64>` *(new)*  
      or alternatively `--benchmark-blocks <u64>` *(new; end = start + blocks - 1)*
- **Add benchmark tuning knobs** (default to harness-like values)
  - `--benchmark-blocks-per-assignment` default `32`
  - `--benchmark-receipts-per-request` default `16`
  - `--benchmark-max-peers` optional cap (default: use all connected)
  - `--benchmark-timeout-ms` default `4000` (match existing)

### Runtime behavior changes (`node/src/main.rs`)
- If `--benchmark` is set:
  - **Do not start RPC** (or start but warn + it’s irrelevant); harness does not.
  - **Do not open/write DB** in `probe/harness` mode (or open read-only if needed for head).
  - Run benchmark once, print summary, exit with code 0 on success.

### Output requirements (probe/harness)
- **At end, print a single summary blob** (JSON is ideal) including:
  - blocks requested / succeeded
  - total time, blocks/sec
  - p50/p95 for `headers_ms`, `receipts_ms`
  - peers used (count + top peers by throughput)
  - retry counts, failure reasons (if any)
This matches the “harness measures stats and outputs after finishing” requirement.

---

## 6) VERY specific step-by-step refactor plan (self-contained tasks)

### Phase 0 — Lock down the target behavior (no big code churn yet)
- **Task 0.1**: Define benchmark modes + configs
  - Add `BenchmarkMode` enum to `node/src/cli/mod.rs`
  - Add `BenchmarkConfig` struct (range, batch sizes, timeouts)
- **Task 0.2**: Remove `debug_receipts_only` and update call sites
  - Update `node/src/main.rs` (where it warns on `debug_receipts_only`)
  - Update `node/src/sync/mod.rs` constructor args (`IngestRunner::new(...)`)
  - Update the CLI defaults test in `node/src/cli/mod.rs`

### Phase 1 — Implement a harness-like fetch engine inside node (fetch-only, no processing)
- **Task 1.1**: Introduce `PeerHandle` + “ready peer channel” in node p2p
  - Extend `p2p::connect_mainnet_peers()` (or add a new `connect_mainnet_peers_with_ready_channel()`) to emit peer sessions as `PeerHandle`s on connect.
  - Keep current `PeerPool` intact for other uses; benchmark will use the ready channel.
- **Task 1.2**: Implement `historical::Scheduler`
  - Data structures (mirror harness):
    - min-heap queue of pending blocks
    - queued set
    - attempts map
    - peer health map (failures + ban-until)
    - escalation queue + tried-peers map (optional but recommended)
  - Functions:
    - `pop_next_batch_for_head(head, max_batch) -> Vec<u64>` (consecutive)
    - `next_batch_for_peer(peer_key, head) -> FetchBatch`
    - `on_fetch_success(peer_key, blocks)`
    - `on_fetch_failure(peer_key, blocks, reason)` → requeue per block + ban logic
- **Task 1.3**: Implement `historical::fetch::fetch_headers_and_receipts(peer, batch, cfg)`
  - Must be equivalent to harness:
    - header request: single `GetBlockHeaders` rising for `[start..start+len-1]`
    - compute hashes once
    - receipts: fixed chunk size `receipts_per_request`
  - Important: do **not** use `request_receipts_chunked` “split-on-partial” logic as the primary strategy in benchmark mode; instead request fixed-size chunks (and treat mismatches as failure → retry on another peer, like harness).
- **Task 1.4**: Emit per-block `ProbeRecord` immediately
  - record `headers_ms`, `receipts_ms`, receipt counts, peer id/version
  - store into stats aggregator; do not allocate/store full receipts unnecessarily

### Phase 2 — Wire `--benchmark probe` to the new engine
- **Task 2.1**: In `node/src/main.rs`, branch early if `config.benchmark.is_some()`
  - Start p2p
  - Build range (`start..=end` or `start..start+N-1`)
  - Run benchmark engine
  - Print summary and exit
- **Task 2.2**: Add periodic progress logging
  - Every X seconds print: completed, inflight, blocks/sec, peers used, p50/p95 so far

### Phase 3 — Split full historical ingest into fetch → process → write (post-probe; but architect now)
- **Task 3.1**: Define shared `FetchedBlock` type + channels
  - `fetch_tx: mpsc::Sender<FetchedBlock>`
  - bounded capacity (e.g. 10k blocks) to prevent memory blowups
- **Task 3.2**: Add `processing.rs` worker pool
  - `ProbeHarnessLike`: no-op (already done)
  - `FullIngest`: (later) bloom, signer recovery, derivation, etc
  - CPU work must be able to run via `spawn_blocking` so it doesn’t starve async IO
- **Task 3.3**: Add `writer.rs`
  - For full ingest only:
    - batch commits
    - checkpoint updates
    - optional ordering enforcement
- **Task 3.4**: Replace `run_fast_range` historical path with the new pipeline
  - In `node/src/sync/mod.rs`:
    - if historical_range exists: call `historical::ingest_range_full(...)`
    - keep active head-range on existing sequential path for now

### Phase 4 — Testing plan (must be added as part of refactor, not afterthought)
- **Task 4.1**: Unit tests for scheduler (`node/src/sync/historical/scheduler.rs`)
  - batch selection is consecutive
  - head gating works
  - requeue + attempts increment works
  - escalation does not retry same peer for same block until needed
- **Task 4.2**: Unit tests for receipt request chunking strategy
  - Ensure fixed-size chunking is used in benchmark mode
  - Mock partial responses and confirm it’s treated as failure (so blocks get retried on different peer) rather than recursively splitting forever
- **Task 4.3**: “Mock fetcher” integration test (no network)
  - Define a trait `PeerClient` used by `historical::fetch` so you can inject a deterministic fake peer and validate:
    - concurrency (multiple peers used)
    - retries happen
    - stats are correct

### Phase 5 — Acceptance criteria (so you can verify without guessing)
- **Benchmark parity**
  - Running `node --benchmark probe --benchmark-start X --benchmark-end X+10999` should:
    - use many peers (>= ~10 if available)
    - show throughput in the same order of magnitude as harness (target: ~1000 blocks/s on a healthy peer set)
    - show stable receipt request sizes (16) and low retry rate
- **Isolation**
  - Probe mode must not:
    - write to DB
    - compute bloom
    - insert into `ChainTracker`
    - start RPC (recommended)
- **Maintainability**
  - Fetch strategy (batch sizes, retries, peer selection) is entirely in `sync/historical/*`
  - Processing policy is entirely in `sync/historical/processing.rs`
  - Storage policy is entirely in `sync/historical/writer.rs`

---

If you want the most “time-to-signal” approach: I’d implement **Phase 0 + Phase 1 + Phase 2 first** (benchmark probe only). That will immediately tell us if the slowdown is mostly “node ingest work” vs “node fetch strategy”, because probe mode removes ingest entirely and uses harness scheduling. After that, the full fetch/process split becomes a straightforward extension rather than a risky big-bang rewrite.