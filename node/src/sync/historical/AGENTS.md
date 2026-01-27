# historical

## Purpose
Historical sync pipeline used for (1) batched ingest backfill into the sharded storage backend
(including optional tail range ingestion for a moving safe head), and (2) follow mode with reorg
handling. Includes peer scheduling, peer-health tracking (AIMD + bans), DB write/compaction, and
benchmark stats/event logging.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `db_writer.rs` - Batched DB writer that buffers `BlockBundle`s, writes WAL/follow segments, and triggers compaction + sealing.
  - **Key items**: `DbWriteConfig`, `DbWriteMode`, `DbWriterMessage`, `DbWriterFinalizeStats`, `DbWriterParams`, `FlushContext`, `run_db_writer()`, `flush_fast_sync_buffer()`, `finalize_fast_sync()`
- `fetch.rs` - Ingest fetch wrappers around the P2P layer (consecutive batch enforcement, ingest payload fetch).
  - **Key items**: `fetch_ingest_batch()`, `FetchIngestOutcome`, `ensure_consecutive()`
- `fetch_task.rs` - Fetch task execution for the ingest pipeline; encapsulates per-batch fetch logic with timeout/error handling.
  - **Key items**: `FetchTaskContext`, `FetchTaskParams`, `run_fetch_task()`, `handle_fetch_timeout()`, `handle_fetch_success()`, `handle_fetch_error()`, `requeue_blocks()`
- `follow.rs` - Live follow loop: head discovery, near-tip backoff, reorg detection, rollback, and incremental ingest.
  - **Key items**: `run_follow_loop()`, `run_head_tracker()`, `run_tail_scheduler()`, `run_reorg_detector()`, `spawn_synced_watcher()`, `SyncStatus::UpToDate`
- `mod.rs` - Pipeline orchestration for ingest (fetch loop, processing workers, DB writer wiring, progress bars).
  - **Key items**: `run_ingest_pipeline()`, `MissingBlocks`, `TailIngestConfig`, `IngestFinalizeStats`, `IngestPipelineOutcome`, `IngestPipelineConfig`, `IngestPipelineTrackers`, `build_peer_health_tracker()`, `run_processor_worker()`, `run_peer_events_tracker()`
- `process.rs` - Processing stage: turns fetched payloads into `BlockBundle`s (tx hashes, size, receipts) and records timing.
  - **Key items**: `process_ingest()`, `KeccakBuf`, `tx_hash_fast()`, `ProcessTiming`
- `reorg.rs` - Reorg detection and common-ancestor search used by follow mode.
  - **Key items**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`
- `scheduler.rs` - Peer-driven scheduler (pending/inflight/escalation) and peer health tracking (AIMD, bans, quality scoring).
  - **Key items**: `SchedulerConfig`, `PeerWorkScheduler`, `PeerHealthTracker`, `PeerHealthConfig`, `PeerQuality`, `PeerHealthDump`, `EscalationState`, `FetchMode`, `enqueue_range()`
- `stats.rs` - Ingest stats aggregation and JSONL bench event writer.
  - **Key items**: `IngestBenchStats`, `IngestBenchSummary`, `SummaryInput`, `BenchEvent`, `BenchEventLogger`, `add_blocks_total()`, `ProcessTiming`
- `types.rs` - Shared types for historical pipeline stages (batches, fetch modes).
  - **Key items**: `FetchBatch`, `FetchMode`

## Key APIs (no snippets)
- **Pipeline**: `run_ingest_pipeline()`, `run_follow_loop()`, `MissingBlocks`, `TailIngestConfig`, `IngestPipelineConfig`, `IngestPipelineTrackers`
- **Fetch tasks**: `FetchTaskContext`, `FetchTaskParams`, `run_fetch_task()`
- **Scheduling**: `PeerWorkScheduler`, `PeerHealthTracker`, `PeerHealthConfig`, `PeerQuality`, `PeerHealthDump`
- **Stats**: `IngestBenchStats`, `SummaryInput`, `BenchEventLogger`

## Relationships
- **Depends on**: `node/src/p2p` (network fetch), `node/src/storage` (writes and reads), `node/src/sync` (shared progress counters/status).
- **Used by**: `node/src/main.rs` (follow mode entrypoint).

## Files (detailed)

### `mod.rs`
- **Role**: Orchestrates the ingest pipeline by wiring together the scheduler, fetch tasks, processing workers, DB writer, and progress/event reporting. Supports optional tail range ingestion for moving-safe-head fast-sync.
- **Key items**: `run_ingest_pipeline()`, `MissingBlocks`, `TailIngestConfig` (fields: `stop_when_caught_up`, `head_offset`), `IngestFinalizeStats`, `IngestPipelineOutcome`, `IngestPipelineConfig`, `IngestPipelineTrackers`, `run_processor_worker()`, `run_peer_events_tracker()`
- **Interactions**:
  - Uses `fetch_task::{FetchTaskContext, FetchTaskParams, run_fetch_task()}` for per-batch P2P fetching (extracted from the main loop).
  - Uses `process::process_ingest()` to build `BlockBundle`s and sends them to `db_writer`.
  - Uses `scheduler::{PeerWorkScheduler, PeerHealthTracker}` to assign work and adapt per-peer batch limits.
  - When `TailIngestConfig` is provided, enqueues appended ranges via `PeerWorkScheduler::enqueue_range()` and updates progress totals; head offset controls safe-head vs head tracking.
- **Knobs / invariants**:
  - Concurrency is bounded by `fast_sync_max_inflight`; per-batch timeout is `fast_sync_batch_timeout_ms`.
  - In follow mode, retries are unbounded (`max_attempts_per_block = u32::MAX`) to tolerate propagation lag near the tip.
  - In follow mode, scheduling caps batches by the global observed head (from head tracking), not per-peer `head_number` which can go stale.
  - In follow mode, "missing blocks" responses (including empty batches) are treated as partials to avoid banning peers for near-tip propagation lag.
  - Tail ingestion (when enabled) tracks `head_seen_rx` and stops scheduling once the safe head is caught up (or continues in follow epochs).

### `fetch_task.rs`
- **Role**: Encapsulates fetch task execution for a single batch of blocks from a peer. Handles timeouts, success/failure outcomes, stats recording, and requeueing.
- **Key items**: `FetchTaskContext` (shared context with scheduler, peer_health, channels, stats, events), `FetchTaskParams` (per-task parameters: peer, blocks, mode, batch_limit, permit), `run_fetch_task()`, `handle_fetch_timeout()`, `handle_fetch_success()`, `handle_fetch_error()`, `requeue_blocks()`
- **Interactions**: Called from `mod.rs` fetch loop; uses `fetch::fetch_ingest_batch()` for P2P; updates `PeerHealthTracker` and `PeerWorkScheduler` on outcomes; sends payloads to processing workers via `fetched_tx`.
- **Knobs / invariants**: Timeout is configured via `fetch_timeout` in context; escalation vs normal mode affects requeue behavior.

### `scheduler.rs`
- **Role**: Maintains the global work queue, priority escalation queue, and per-peer health/quality model so scheduling adapts to real-world peer behavior.
- **Key items**: `SchedulerConfig`, `PeerWorkScheduler::next_batch_for_peer()`, `PeerWorkScheduler::enqueue_range()`, `PeerHealthTracker::record_success()`, `PeerHealthConfig` (pub), `PeerQuality` (pub), `PeerHealthDump` (pub), `EscalationState`, `completed_count()`, `escalation_len()`
- **Interactions**: Called from `mod.rs` and `fetch_task.rs` for batch assignment, requeue, and peer health updates; consulted for "best peer" selection via quality scores.
- **Escalation queue**: Blocks that fail N attempts (default: 5) are promoted to a priority escalation queue. Escalation blocks are checked FIRST before the normal queue, ensuring difficult blocks don't starve. Uses shard-aware prioritization (shards with fewer missing blocks get priority for faster compaction) and peer cooldown tracking (30-second cooldown before same peer retries same block). Blocks in escalation retry indefinitely until fetched.
- **Knobs / invariants**: AIMD batch limit clamps between `aimd_min_batch` and `aimd_max_batch`; bans trigger after `peer_failure_threshold`; `ESCALATION_THRESHOLD` (default: 5) controls promotion; `ESCALATION_PEER_COOLDOWN` (30s) prevents hammering same peer.

### `fetch.rs`
- **Role**: Wraps P2P requests into stage-friendly outcomes for ingest mode.
- **Key items**: `FetchIngestOutcome`, `fetch_ingest_batch()`, `ensure_consecutive()`
- **Interactions**: Uses `p2p::fetch_payloads_for_peer()` for ingest.
- **Knobs / invariants**: Batches must be consecutive.

### `process.rs`
- **Role**: Converts fetched payloads into storage-ready `BlockBundle`s and records processing timings for benchmarks.
- **Key items**: `process_ingest()`, `KeccakBuf`, `KECCAK_SCRATCH_LEN`, `tx_hash_fast()`, `block_rlp_size()`, `ProcessTiming`
- **Interactions**: Feeds `BlockBundle`s to `db_writer` via `DbWriterMessage::Block`; updates `IngestBenchStats` when enabled.
- **Knobs / invariants**: Requires tx count to match receipts count; logs are counted from receipts (and derived at query time).

### `db_writer.rs`
- **Role**: Applies `BlockBundle` writes to storage and manages compaction/sealing so reads work after ingest completes.
- **Key items**: `run_db_writer()`, `DbWriterParams` (full config for DB writer task), `FlushContext` (shared context for flush operations), `flush_fast_sync_buffer()`, `finalize_fast_sync()`, `DbWriteMode::{FastSync, Follow}`, `DbWriterMessage::{Block, Finalize}`, `DbWriterFinalizeStats`, `BenchEvent::DbFlushStart/End`, `BenchEvent::CompactionStart/End`
- **Interactions**: Calls `Storage::write_block_bundles_wal()` (fast sync) or `Storage::write_block_bundle_follow()` (follow). Follow writes are gated by an in-memory reorder buffer to enforce in-order appends. Returns finalize timing stats used by `main.rs` logging.
- **Knobs / invariants**: Compaction is serialized with a `Semaphore(1)` to cap peak IO/memory; finalize logs dirty shard/WAL sizes and runs `compact_all_dirty()` as a safety net.

### `follow.rs`
- **Role**: Runs an infinite loop to keep the store close to the network head, including reorg detection and bounded rollback.
- **Key items**: `run_follow_loop()`, `run_head_tracker()`, `run_tail_scheduler()`, `run_reorg_detector()`, `spawn_synced_watcher()`, `FOLLOW_POLL_MS`, `FOLLOW_NEAR_TIP_BLOCKS`, `HEAD_PROBE_PEERS`, `REORG_PROBE_PEERS`
- **Interactions**: Uses `discover_head_p2p()` to observe head; spawns per-epoch head/tail trackers (`run_head_tracker()`, `run_tail_scheduler()`) to extend the ingest range as the head moves; spawns `run_reorg_detector()` to watch for reorganizations; uses `reorg::{preflight_reorg, find_common_ancestor}` and calls `Storage::rollback_to()` when needed; uses `run_ingest_pipeline()` to fill missing blocks in follow mode.
- **Knobs / invariants**: When caught up, follow can report `SyncStatus::UpToDate` (outer follow loop: nothing to ingest) or `SyncStatus::Following` (long-lived ingest epoch: caught up and waiting). An optional one-shot "synced" signal fires on the first UpToDate/Following edge (used to start RPC via `spawn_synced_watcher()`). Reorg rollback is capped by `NodeConfig.rollback_window`.

### `reorg.rs`
- **Role**: Detects reorgs by comparing stored tip hashes against network headers and finds the latest common ancestor for rollback.
- **Key items**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`, `find_common_ancestor_number()`
- **Interactions**: Called by `follow.rs` prior to ingesting new blocks.
- **Knobs / invariants**: Uses a small number of peers (`probe_peers`) and falls back to `Inconclusive` when headers are unavailable.

### `stats.rs`
- **Role**: Aggregates ingest performance metrics and writes JSON summaries and JSONL event streams.
- **Key items**: `IngestBenchStats`, `IngestBenchSummary`, `SummaryInput` (input struct for `summary()` method), `BenchEvent`, `BenchEventLogger`, `DbWriteByteTotals`, `IngestBenchStats::add_blocks_total()`, `ProcessTiming`
- **Interactions**: Updated by fetch/process/db stages; `BenchEventLogger` is used by `mod.rs` and `main.rs` to emit structured time-series events. Uses `metrics::{rate_per_sec, percentile_triplet}` for calculations. Uses `parking_lot::Mutex` instead of `std::sync::Mutex` for performance.
- **Knobs / invariants**: Sample vectors are capped (`SAMPLE_LIMIT`) to bound memory use during long runs; totals can grow when tail ranges are appended.

### `types.rs`
- **Role**: Shared stage types for historical pipeline stages.
- **Key items**: `FetchBatch`, `FetchMode`
- **Interactions**: Imported by scheduler/fetch modules.

## End-to-end flow (high level)
- Build `PeerHealthTracker` (AIMD + bans) from `NodeConfig` fast-sync parameters.
- Initialize `PeerWorkScheduler` with the missing block set and per-block attempt tracking.
- Spawn `spawn_peer_feeder()` to supply newly connected peers into a ready queue.
- Optionally spawn a tail-ingest task to append new safe-head ranges into the scheduler.
- Fetch loop: pick the best available peer (quality score), assign a consecutive batch, and fetch payloads with a hard timeout.
- On success: send payloads to processing workers, mark blocks completed, and record peer success/partials.
- Processing workers: `process_ingest()` builds `BlockBundle`s and sends them to the DB writer channel.
- DB writer: buffer + `write_block_bundles_wal()` (fast sync) or direct follow writes, then compact dirty shards and seal completed shards.
- Follow mode: periodically observe head, preflight reorgs, rollback to common ancestor when needed, and re-run ingest for missing blocks near the tip.
