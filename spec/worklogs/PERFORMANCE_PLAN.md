# Performance plan (stateless-history-node)

## Goals
- Make performance bottlenecks **obvious** during ingest (network vs CPU vs DB).
- Ensure the ingest benchmark is **representative of real ingestion**, but exits after the requested range.
- Produce **durable artifacts** per run:
  - a compact **summary JSON** with config + results
  - an optional **timeline/trace** (Chrome trace / JSONL) for “how it behaves” analysis in real environments
- Improve operator UX (e.g. **human-readable ETA**).
- Prefer **tool-based profiling** (Reth-style) where it provides better insights than custom code.

## What “good performance data” should answer
- **Is it generally efficient?**
  - What is the sustained throughput (blocks/s, logs/s, MiB/s downloaded)?
  - How stable is performance over time (tail latencies, variability)?
- **Where is the bottleneck?**
  - Fetch vs processing vs DB writes vs queueing/backpressure.
  - Per-request latency distributions (p50/p95/p99) to identify tail issues.
- **Which knobs matter?**
  - chunk size, max inflight, per-batch timeout, worker count, allocator, build profile, `target-cpu=native`.

---

## How benchmark ingest works today (baseline)

### Entry path
- `node/src/main.rs`:
  - `--benchmark ingest`:
    - opens storage (`Storage::open`)
    - starts P2P (`connect_mainnet_peers`)
    - resolves `head_at_startup`
    - computes a fixed range via `compute_target_range(start_block, end_block, head, rollback_window)`
    - runs `sync::historical::run_ingest_pipeline(...)` with `bench=Some(IngestBenchStats)`
    - prints a JSON summary to stdout and exits
    - does **not** start RPC and does **not** enter live follow

### Shared “real ingest” code
- Real node execution (`node/src/main.rs`) runs `sync::historical::run_follow_loop(...)`.
- `run_follow_loop` calls `run_ingest_pipeline` for:
  - initial backfill / catch-up (`start..=target_head`)
  - repeated catch-up while following

### What differs vs follow mode
- Follow mode sets `head_cap_override=Some(target_head)` which enables “near-tip retry forever” behavior inside `run_ingest_pipeline`.
- Benchmark ingest runs a fixed historical range (typically older than the rollback window), with normal retry semantics.

### Existing ingest benchmark metrics (already present)
`IngestBenchStats` already tracks:
- **Fetch**: total time, batches, blocks, failures, bytes (headers/bodies/receipts/logs), avg throughput (MiB/s)
- **Process**: total time, failures, avg time per block, and breakdown (hashing/tx/log building/etc)
- **DB write**: total time, batches, blocks, avg time per block/batch, and encoded byte totals
- **Peers**: peers_used and peer_failures_total

---

## Observability strategy (modes)

### Always-on (production-safe)
- **Structured logs** at INFO/WARN for lifecycle events.
- **Lightweight counters** (already present in `IngestBenchStats` for benchmarks; optionally promote a subset to always-on metrics later).
- Keep overhead minimal; no per-block JSON serialization.

### Benchmark summary (default for `--benchmark ingest`)
- One **summary JSON** at end of run (and on Ctrl-C).
- Optional: additional “top peer” diagnostics snapshot at end.

### Benchmark timeline / “flight recorder” (optional)
Two equivalent approaches:
- **Chrome trace (recommended)**: span-based timeline output (excellent for bottleneck attribution).
- **JSONL events**: very verbose event stream for later offline analysis.

For benchmarks, we can **buffer in memory and dump on exit** to avoid disk IO jitter.
For long-running/production runs, prefer a **non-blocking writer** (ring buffer + worker) to avoid stalls.

### Tool-based profiling (optional; defer if not needed)
- **CPU profiling**: external `samply` or Instruments (macOS) for hotspot identification.
- **`tracing_samply`**: a tracing layer to label profiles with spans (Reth supports this).
- **Async runtime diagnosis**: `tokio-console` if progress stalls are mysterious (task-level view).
- **Allocator experiments**: `jemalloc` feature and (later) `jemalloc-prof` for heap profiling.

## What to add (metrics + artifacts)

### 1) Benchmark run artifacts
Add a benchmark output mode that writes:

- **Summary JSON file** (always, when enabled):
  - `meta`: timestamp, run_id, benchmark_name, mode, version, debug/release flag
  - `argv`: full CLI args
  - `env`: OS, CPU count, and build metadata (profile, features; optionally git sha if available)
  - `config`: relevant `NodeConfig` + derived/effective values (range, worker count, effective caps)
  - `range`: start/end, head_at_startup, safe_head used, rollback_window_applied
  - `results`: existing `IngestBenchSummary`
  - `peer_health_top`: top N peers by quality/throughput + bans + last errors
  - `peer_health_worst`: worst N peers by failures/latency (helps explain bottlenecks)
  - `notes`: free-form notes field (optional) to annotate runs (“release+native”, “jemalloc”, etc.)

- **Optional timeline artifacts** (only when enabled):
  - **Chrome trace** file (span timeline)
  - **JSONL event log** (one event per line)
  - Optional: “dropped events” counter if buffering is bounded

Filename should include:
- benchmark name/mode
- human-readable timestamp
- range + key knobs (chunk size, inflight, timeout, build profile/allocator if available)

Example filename shape:
`benchmarks/ingest-bench__20260121T154210Z__range-10000000-10009999__chunk16__inflight15__timeout5000.json`

### 2) Make bottlenecks visible: additional metrics worth adding
Keep totals/averages (already there), but add **distributions** and **breakdowns** that explain efficiency and bottlenecks:

- **Fetch breakdown**
  - Separate timings for:
    - headers request
    - bodies request
    - receipts request
  - Track p50/p95/p99 (either by storing samples in benchmark mode, or reservoir sampling).
  - Track request counts per stage (helps detect “too many round trips” vs “slow network”).

- **Queueing / backpressure**
  - Time spent:
    - waiting for an available peer (ready queue)
    - waiting for the inflight semaphore
    - blocked by per-peer timeouts
  - Track channel backpressure (optional): queue depths, send delays (Reth uses metered mpsc wrappers).

- **DB writer visibility**
  - Batch flush duration p50/p95/p99 (not just avg).
  - Optional: buffer depth / how often flush is triggered by size vs by end-of-stream.
  - Optional: bytes written per flush and effective write throughput (MiB/s).

- **Per-peer performance**
  - Per-peer: batches assigned, blocks fetched, failures/partials, avg fetch latency, throughput.
  - Output only top N (or N worst) to keep summary compact.

### 3) Event/timeline logging design
Goal: detailed causality without killing performance, so we can analyze “how it works in the real environment”.

Options:
- **Chrome trace output (recommended)**:
  - Span-based timeline you can open in Perfetto / `chrome://tracing`.
  - Best at explaining “where time goes” across threads/tasks.
- **JSONL event log (max fidelity)**:
  - Extremely verbose event stream for offline computation of any metric.
  - For benchmarks, prefer **in-memory buffering + dump-on-exit**.
  - For long runs, prefer **non-blocking writer** and optionally rotate.

To protect real-world performance:
- event logging should be **off by default**
- consider enabling it automatically only for `--benchmark ingest` runs.

---

## UX tweak: human-readable ETA
Replace `"{:.0}s"` eta formatting with a compact formatter:
- `3d 5h`
- `19h 23m`
- `15m 30s`
- `53s`

Apply consistently to:
- main ingest progress bar ETA
- escalation progress ETA

---

## Measurement plan (10k blocks)
Run the same 10k range and compare summary JSONs:

0) (Optional) Repeat each run 3× and compare medians (reduces noise).
1) Debug build (correctness + baseline “how slow is debug”)
2) Release build (baseline)
3) Release + native CPU (`RUSTFLAGS='-C target-cpu=native'`)
4) Release + jemalloc feature (if added)
5) (Later) Release + profiling layers (`tracing_samply`, Chrome trace) to attribute bottlenecks

Keep constant:
- range (e.g. `start=10_000_000`, `end=10_009_999`)
- `fast_sync_chunk_size`, `fast_sync_max_inflight`, `fast_sync_batch_timeout_ms`
- P2P caps / peer cache

## “Real environment” run
After 10k microbench comparisons, do a longer run (30–60 min) to validate:
- stability of throughput over time
- tail latency behavior under sustained load
- memory growth/fragmentation signals (allocator effects)

---

## Implementation steps (checklist)
- [ ] Add benchmark output config (directory + naming + JSON summary file)
- [ ] Extend summary JSON with `argv`, `env` (build/profile/features), derived values, and peer-health snapshots (top + worst)
- [ ] Add fetch timing breakdown + percentiles (benchmark-only sampling)
- [ ] Add optional Chrome trace output (span timeline) for benchmarks
- [ ] Add optional JSONL event logger (benchmark in-memory; long-run non-blocking) and emit key events
- [ ] (Optional) Metered channels / backpressure counters (Reth-style) for internal pipeline visibility
- [ ] Human-readable ETA formatter and wire it into progress UIs
- [ ] Run the 4 comparison runs and capture artifacts under `benchmarks/`
- [ ] (Optional) Add `tracing_samply` integration and document how to profile CPU hotspots

