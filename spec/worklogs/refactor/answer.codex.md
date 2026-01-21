[1 tool called]

Below is a detailed, concrete spec/plan that addresses each of your six points. I kept it specific to the current `node/` layout and the harness behavior you described.

## 1) Changes needed to make node behave like harness
- Replace range‑based scheduling with a peer‑driven scheduler like harness: per‑peer “ready” queue, head‑aware batches, and requeue peers immediately after a batch completes.
- Decouple fetching from processing; fetching should only do headers+receipts (plus hash once), and hand off results to a processing queue.
- Add a true probe/benchmark processing mode that skips chain insert, bloom computation, and **all DB writes**, matching harness’s minimal CPU/IO.
- Move retry logic to the **block** level (not chunk/range); add escalation across peers and peer bans similar to harness.
- Respect per‑peer head in scheduling; never request blocks above that peer’s head (harness does this, node currently does not).
- Compute header hash once and carry it with the fetched payload; eliminate duplicate hashing in fetch + process.

## 2) Problems in the current node pipeline
- **In‑order processing stall** (`next_expected` gating) blocks all later chunks when one chunk is slow or fails.
- **Single‑peer per range** with sequential retries/backoff (250ms), causing a slow peer to delay the whole chunk.
- **Duplicate header hashing** in `p2p` fetch and again in receipts‑only processing.
- **Receipts‑only still does heavy work**: chain insertion, bloom computation, and DB commits.
- **Peer head not enforced** during requests, causing wasted timeouts on peers behind the requested range.
- **Chunk‑level retries** re‑fetch the entire chunk even if only a subset of blocks failed.

## 3) Refactor plan to fully isolate and rewrite historical fetching
### Phase A: Architecture split and contracts
- Add a new `node/src/historical/` (or `node/src/sync/historical.rs`) module that owns historical sync only; keep existing `sync` for active mode initially.
- Define explicit types for `FetchRequest`, `FetchedBlock`, `ProcessInput`, `ProcessResult`, and timing stats in `node/src/historical/types.rs`.
- Introduce traits `BlockFetcher`, `BlockProcessor`, and `DataSink` so fetch, process, and persistence are swappable and testable.
- Add a `BenchmarkMode` enum to config and route historical sync through the new pipeline when benchmark is enabled.
- Add a `MetricsRecorder` interface (or struct) that tracks request latency, receipt counts, and peer success/failure.

### Phase B: Fetch pipeline redesign (harness‑like)
- Implement a `FetchScheduler` that pulls ready peers from a channel and assigns sequential block batches with `blocks_per_assignment`.
- Implement `FetchWorker` tasks that do headers + receipts per batch; chunk receipts by `receipts_per_request`; compute hash once and attach to payload.
- Add per‑block attempt tracking with an escalation queue; after N attempts, send a block to a different peer; ban peers with repeated failures.
- Push `FetchedBlock` objects to a bounded `process_tx` channel, and push timing stats to `MetricsRecorder`.
- Update `PeerPool` to expose peer head; scheduler must filter out blocks above peer head.

### Phase C: Processing pipeline redesign (independent and parallel)
- Implement a `ProcessWorker` pool that consumes `FetchedBlock`s from the process queue and applies a `ProcessingMode`.
- For historical/probe mode, allow **out‑of‑order** processing and skip chain insert, bloom, and DB writes entirely.
- For full historical mode, keep chain insertion and DB writes but decouple from fetch; use a small reordering buffer only if needed for monotonic commits.
- Provide `DataSink` implementations: `NoopSink` (probe/benchmark), `DbSink` (current storage), and optional `MetricsSink`.
- Add optional CPU parallelism for heavy steps in full mode using `spawn_blocking` or a Rayon pool; keep probe mode lightweight.

### Phase D: Benchmark/probe mode introduction
- Replace `--debug-receipts-only` with `--benchmark <type>`; define `BenchmarkMode::{Harness, Probe}` (or keep `Probe` as alias).
- `benchmark=harness` should fetch **headers + receipts only**, compute hash once, skip chain insert, skip bloom, skip DB, and output a harness‑style summary.
- Record and print: total blocks, elapsed time, blocks/sec, peers used, per‑peer success rate, p50/p95 header/receipt latency, failure reasons.
- Ensure benchmark mode does not touch `last_indexed_block` or write any storage tables.
- Leave “full processing with per‑stage timings” as a later extension to `benchmark=full` (planned but not implemented now).

### Phase E: Tests and validation
- Unit tests for the scheduler: head‑aware batching, escalation behavior, per‑block retry, peer ban recovery.
- Unit tests for probe processing: verify no DB calls and no chain mutations.
- Integration test for `--benchmark harness`: run with mocked peers and assert the summary output schema.
- Regression test that benchmark mode leaves DB untouched and does not update meta keys.
- Small perf test in CI or locally with mocked peers to ensure throughput > baseline.

## 4) Fetch vs process separation (spec sketch)
Flow: `RangePlanner -> FetchScheduler -> FetchWorkers -> ProcessQueue -> ProcessWorkers -> DataSink`

Component responsibilities:
- **RangePlanner**: emits block numbers for the historical window; no DB dependencies.
- **FetchScheduler**: assigns batches to ready peers, tracks attempts, and pushes fetch tasks.
- **FetchWorker**: requests headers and receipts, computes block hash once, returns `FetchedBlock`.
- **ProcessWorker**: applies `ProcessingMode` (probe vs full) on each `FetchedBlock`, optionally parallel.
- **DataSink**: persists results for full mode or discards them for probe/benchmark.
- **MetricsRecorder**: aggregates request timings, throughput, and peer health stats.

Queueing/backpressure & ordering:
- Bounded `fetch_tx` to limit per‑peer inflight batches and avoid memory growth.
- Bounded `process_tx` so heavy processing throttles fetchers naturally.
- Probe mode allows out‑of‑order; full mode may use a small reorder buffer or a commit stage if needed.
- Track queue depths and inflight counts as stats, similar to harness’s windowed metrics.
- Optional CPU offload for heavy steps in full mode; probe mode stays async only.

## 5) `--benchmark` CLI spec (replacing `--debug-receipts-only`)
- Add `--benchmark <type>` in `node/src/cli/mod.rs`, with types `probe` and `harness`.
- Map `benchmark=harness` to the new historical pipeline with `ProcessingMode::Probe` and `DataSink::Noop`.
- Remove or deprecate `--debug-receipts-only` in config; keep a temporary alias if you want short‑term compatibility.
- Update `main.rs` to route benchmark runs directly to the historical fetch pipeline (skip `IngestRunner`).
- Output a single benchmark summary at the end; no DB writes or progress‑bar updates.

---

Next step on my side would be implementing Phase A + B in this worktree (new historical module + fetch scheduler/worker path) and wiring a minimal `benchmark=harness` that prints summary stats and skips DB/processing.