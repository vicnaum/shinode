## Harness vs node probe/ingest parity notes

This doc is a running checklist of **known differences** between `harness/` and the node’s historical pipeline (`node/src/sync/historical/`), to avoid “we forgot why” regressions.

### What now matches Harness (important)
- **Network dial config defaults**
  - `max_outbound=400`
  - `max_concurrent_dials=100`
  - `refill_interval_ms=500`
  - Node: `node/src/p2p/mod.rs` constants and `PeersConfig` wiring.
  - Harness: `harness/src/main.rs` `CliConfig` defaults and `PeersConfig` wiring.
- **Request timeout**
  - `request_timeout_ms=4000` (4s)
  - Node: `REQUEST_TIMEOUT` in `node/src/p2p/mod.rs`
  - Harness: `request_timeout_ms` default in `harness/src/main.rs`
- **Work sizing defaults**
  - `blocks_per_assignment=32`
  - `receipts_per_request=16`
- **Per-block handling of partial results**
  - Harness requeues **only missing blocks**, not the whole batch.
  - Node probe now matches:
    - headers: map-by-number, requeue missing
    - receipts: count-by-index, requeue missing
- **Progress output cadence**
  - 10 updates/sec (`100ms`) and `ProgressDrawTarget::stderr_with_hz(10)`

### Remaining known differences
- **No per-peer overload in node probe (current)**
  - The `spawn_peer_feeder` only seeds **new** peers once; it does not re-add existing peers on its 200ms tick.
  - A peer is re-queued only after its fetch task completes.
  - This means **one in-flight batch per peer**, same as Harness. The “feeder spamming” hypothesis does not apply to the current code.
- **Escalation pass UX**
  - Node now retries failed blocks in an escalation queue, but does **not** yet expose a separate escalation progress bar or per‑block “tried across peers” counters.
- **Probe accounting depth**
  - Harness logs detailed per-request/per-block JSONL events (requests/probes/window_stats).
  - Node probe logs summary JSON and minimal warnings.
- **Data moved through the pipeline**
### Recent fixes prompted by perf review
- **JoinSet cleanup in probe loop**
  - Node now drains completed fetch tasks via `try_join_next()` during scheduling.
  - Prevents accumulation of completed task handles for long runs.
- **Warmup delay**
  - Added `WARMUP_SECS=3` in probe loop to match Harness behavior.
- **Header hash clone removed**
  - Probe now moves headers into `seal_slow` instead of cloning.
- **Escalation queue**
  - Added a failed‑block escalation queue that retries across peers after the main queue drains.
- **Escalation progress bar**
  - Added a secondary escalation bar (red) that tracks failed blocks retried across peers.
  - Harness probe computes **receipt counts only** and never materializes/stores full receipts.
  - Node probe now matches (uses `request_receipt_counts`), but ingest still materializes full receipts (by design).

### Why inflight can “drop”
Even with many connected peers, `inflight` can drop temporarily due to:
- **Batch fragmentation from missing blocks**
  - The scheduler enforces consecutive batches; if a missing block is requeued, the next batch often becomes size 1–few until the hole is filled.
  - This reduces effective `inflight` and makes it fluctuate.
- **Peer head gating / bans**
  - A peer can temporarily be unable to accept the current smallest pending block (head < needed) or be banned after consecutive failures.
  - That peer cycles in the ready queue without contributing.

### Key file map
- Harness:
  - `harness/src/main.rs`:
    - `spawn_progress_bar` (10Hz)
    - `pop_next_batch_for_head` (consecutive scheduling)
    - `probe_block_batch` (headers+hash+receipt-counts; per-block requeue on missing)
- Node:
  - `node/src/sync/historical/mod.rs`:
    - `spawn_probe_progress_bar`
    - `spawn_peer_feeder` (dynamic peer injection)
    - `run_benchmark_probe` scheduler loop
  - `node/src/sync/historical/fetch.rs`:
    - `fetch_probe_batch` (headers+hash+receipt-counts; per-block requeue on missing)
  - `node/src/p2p/mod.rs`:
    - `request_headers_batch`
    - `request_receipt_counts`

