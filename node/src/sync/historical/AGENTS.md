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
  - **Key items**: `DbWriteConfig`, `DbWriteMode`, `DbWriterMessage`, `DbWriterFinalizeStats`, `DbWriterParams`, `FlushContext`, `ShardRemainingTracker`, `run_db_writer()`, `flush_fast_sync_buffer()`, `finalize_fast_sync()`, `db_bytes_for_bundles()`
- `fetch.rs` - Ingest fetch wrappers around the P2P layer (consecutive batch enforcement, ingest payload fetch).
  - **Key items**: `fetch_ingest_batch()`, `FetchIngestOutcome`, `ensure_consecutive()`
- `fetch_task.rs` - Fetch task execution for the ingest pipeline; encapsulates per-batch fetch logic with error handling.
  - **Key items**: `FetchTaskContext`, `FetchTaskParams`, `run_fetch_task()`, `handle_fetch_success()`, `handle_fetch_error()`, `requeue_blocks()`, `ActiveFetchTaskGuard`
- `follow.rs` - Live follow loop: head discovery, near-tip backoff, reorg detection, rollback, and incremental ingest.
  - **Key items**: `run_follow_loop()`, `run_head_tracker()`, `run_tail_scheduler()`, `run_reorg_detector()`, `spawn_synced_watcher()`, `HeadTrackerContext`, `ReorgDetectorContext`, `FOLLOW_POLL_MS`, `FOLLOW_NEAR_TIP_BLOCKS`, `HEAD_PROBE_PEERS`, `REORG_PROBE_PEERS`
- `mod.rs` - Pipeline orchestration for ingest (fetch loop, processing workers, DB writer wiring, progress bars).
  - **Key items**: `run_ingest_pipeline()`, `MissingBlocks`, `TailIngestConfig`, `IngestFinalizeStats`, `IngestPipelineOutcome`, `IngestPipelineConfig`, `IngestPipelineTrackers`, `build_peer_health_tracker()`, `run_processor_worker()`, `run_peer_events_tracker()`, `pick_best_ready_peer_index()`, `spawn_peer_feeder()`, `ProcessorContext`
- `process.rs` - Processing stage: turns fetched payloads into `BlockBundle`s (tx hashes, size, receipts) and records timing.
  - **Key items**: `process_ingest()`, `KeccakBuf`, `tx_hash_fast()`, `signing_hash_fast()`, `ProcessTiming`, `KECCAK_SCRATCH_LEN`
- `reorg.rs` - Reorg detection and common-ancestor search used by follow mode.
  - **Key items**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`, `find_common_ancestor_number()`, `fetch_single_header()`
- `scheduler.rs` - Peer-driven scheduler (pending/inflight/escalation) and peer health tracking (AIMD, bans, quality scoring).
  - **Key items**: `SchedulerConfig`, `PeerWorkScheduler`, `PeerHealthTracker`, `PeerHealthConfig`, `PeerQuality`, `PeerHealthDump`, `EscalationState`, `FetchMode`, `enqueue_range()`, `record_success()`, `record_partial()`, `record_failure()`, `note_error()`, `is_peer_cooling_down()`, `snapshot()`, `quality()`
- `stats.rs` - Ingest stats aggregation and JSONL bench event writer.
  - **Key items**: `IngestBenchStats`, `IngestBenchSummary`, `SummaryInput`, `BenchEvent`, `BenchEventLogger`, `ProcessTiming`, `RangeSummary`, `PeerSummary`, `FetchByteTotals`, `DbWriteByteTotals`
- `types.rs` - Shared types for historical pipeline stages (batches, fetch modes).
  - **Key items**: `FetchBatch`, `FetchMode`

## Key APIs (no snippets)
- **Pipeline**: `run_ingest_pipeline()`, `run_follow_loop()`, `MissingBlocks`, `TailIngestConfig`, `IngestPipelineConfig`, `IngestPipelineTrackers`, `IngestPipelineOutcome`
- **Fetch tasks**: `FetchTaskContext`, `FetchTaskParams`, `run_fetch_task()`, `handle_fetch_success()`, `handle_fetch_error()`, `ActiveFetchTaskGuard`
- **Scheduling**: `PeerWorkScheduler`, `PeerHealthTracker`, `PeerHealthConfig`, `PeerQuality`, `PeerHealthDump`, `SchedulerConfig`
- **Stats**: `IngestBenchStats`, `SummaryInput`, `BenchEventLogger`, `ProcessTiming`, `BenchEvent`
- **Reorg**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`

## Relationships
- **Depends on**: `node/src/p2p` (network fetch), `node/src/storage` (writes and reads), `node/src/sync` (shared progress counters/status).
- **Used by**: `node/src/main.rs` (follow mode entrypoint).

## Files (detailed)

### `mod.rs`
- **Role**: Orchestrates the ingest pipeline by wiring together the scheduler, fetch tasks, processing workers, DB writer, and progress/event reporting. Supports optional tail range ingestion for moving-safe-head fast-sync with automatic per-shard compaction triggering.
- **Key items**: `run_ingest_pipeline()`, `MissingBlocks`, `TailIngestConfig` (fields: `stop_when_caught_up`, `head_offset`), `IngestFinalizeStats`, `IngestPipelineOutcome`, `IngestPipelineConfig`, `IngestPipelineTrackers`, `run_processor_worker()`, `run_peer_events_tracker()`, `ProcessorContext`, `pick_best_ready_peer_index()`, `spawn_peer_feeder()`, `PEER_FEEDER_CURSOR`
- **Interactions**:
  - Uses `fetch_task::{FetchTaskContext, FetchTaskParams, run_fetch_task()}` for per-batch P2P fetching.
  - Uses `process::process_ingest()` to build `BlockBundle`s and sends them to `db_writer`.
  - Uses `scheduler::{PeerWorkScheduler, PeerHealthTracker}` to assign work and adapt per-peer batch limits.
  - Spawns scheduler gauge task to emit `BenchEvent::SchedulerGaugeSample` every 10 seconds.
  - When `TailIngestConfig` is provided, enqueues appended ranges via `PeerWorkScheduler::enqueue_range()` and updates progress totals; head offset controls safe-head vs head tracking.
  - Peer feeder rotates peer list for fairness using atomic `PEER_FEEDER_CURSOR`.
  - Detects stalls (30s with no progress) and logs detailed peer health dump.
- **Knobs / invariants**:
  - Concurrency is bounded by `fast_sync_max_inflight`.
  - In follow mode, retries are unbounded to tolerate propagation lag near the tip.
  - In follow mode, scheduling caps batches by the global observed head (from head tracking), not per-peer `head_number`.
  - In follow mode, "missing blocks" responses (including empty batches) are treated as partials to avoid banning peers for near-tip propagation lag.
  - Tail ingestion (when enabled) tracks `head_seen_rx` and stops scheduling once the safe head is caught up.
  - **Stale-peer handling**: Peers cooling down are recycled with a 500ms delay. Peers >10k blocks behind are dropped entirely. Peer `head_number` is refreshed from the pool at all receive sites.
  - **Per-shard compaction**: `remaining_per_shard` tracker ensures shards compact as soon as all their blocks are written; compactions gated by `Semaphore(1)`.

### `fetch_task.rs`
- **Role**: Encapsulates fetch task execution for a single batch of blocks from a peer. Handles timeouts, success/failure outcomes, stats recording, and requeueing. Tracks active task count via `ActiveFetchTaskGuard`.
- **Key items**: `FetchTaskContext` (shared context with scheduler, peer_health, channels, stats, events, `active_fetch_tasks` counter, `db_mode`), `FetchTaskParams` (per-task parameters: peer, blocks, mode, batch_limit, permit), `run_fetch_task()`, `handle_fetch_success()`, `handle_fetch_error()`, `requeue_blocks()`, `ActiveFetchTaskGuard`
- **Interactions**: Called from `mod.rs` fetch loop; uses `fetch::fetch_ingest_batch()` for P2P; updates `PeerHealthTracker` and `PeerWorkScheduler` on outcomes; sends payloads to processing workers via `fetched_tx`; records coverage and last-block-received timestamp for TUI display.
- **Knobs / invariants**: Escalation vs normal mode affects requeue behavior; active task count auto-decremented on drop via guard.

### `scheduler.rs`
- **Role**: Maintains the global work queue, priority escalation queue, and per-peer health/quality model so scheduling adapts to real-world peer behavior. Escalation queue uses shard-aware prioritization.
- **Key items**: `SchedulerConfig`, `PeerWorkScheduler::next_batch_for_peer()`, `PeerWorkScheduler::enqueue_range()`, `PeerHealthTracker::record_success()`, `PeerHealthTracker::record_partial()`, `PeerHealthTracker::record_failure()`, `PeerHealthConfig` (pub), `PeerQuality` (pub), `PeerHealthDump` (pub), `EscalationState`, `completed_count()`, `escalation_len()`, `quality()`, `snapshot()`
- **Interactions**: Called from `mod.rs` and `fetch_task.rs` for batch assignment, requeue, and peer health updates; consulted for "best peer" selection via quality scores.
- **Escalation queue**: Blocks that fail N attempts (default: 5) are promoted to a priority escalation queue checked FIRST. Uses shard-aware prioritization (shards with fewer missing blocks get priority for faster compaction) and per-block per-peer cooldown tracking (30s `ESCALATION_PEER_COOLDOWN`). Blocks retry indefinitely until fetched.
- **Knobs / invariants**: AIMD batch limit clamps between `aimd_min_batch` (1) and `aimd_max_batch`; partial responses trigger 0.7x reduction, failures 0.5x; bans trigger after `peer_failure_threshold`; `ESCALATION_THRESHOLD` (default: 5) controls promotion; `ESCALATION_PEER_COOLDOWN` (30s) prevents hammering same peer.

### `fetch.rs`
- **Role**: Wraps P2P requests into stage-friendly outcomes for ingest mode.
- **Key items**: `FetchIngestOutcome`, `fetch_ingest_batch()`, `ensure_consecutive()`
- **Interactions**: Uses `p2p::fetch_payloads_for_peer()` for ingest.
- **Knobs / invariants**: Batches must be consecutive; returns empty if input is empty.

### `process.rs`
- **Role**: Converts fetched payloads into storage-ready `BlockBundle`s and records processing timings for benchmarks. Uses inline Keccak scratch buffer for zero-copy tx hashing.
- **Key items**: `process_ingest()`, `KeccakBuf`, `KECCAK_SCRATCH_LEN` (4096), `tx_hash_fast()`, `signing_hash_fast()`, `block_rlp_size()`, `ProcessTiming`
- **Interactions**: Feeds `BlockBundle`s to `db_writer` via `DbWriterMessage::Block`; updates `IngestBenchStats` when enabled; emits `BenchEvent::ProcessStart/End`.
- **Knobs / invariants**: Requires tx count to match receipts count; logs are counted from receipts and derived at query time.

### `db_writer.rs`
- **Role**: Applies `BlockBundle` writes to storage and manages compaction/sealing so reads work after ingest completes. Supports both fast-sync (WAL-based batched) and follow-mode (reorder buffer) writes.
- **Key items**: `run_db_writer()`, `DbWriterParams`, `FlushContext`, `flush_fast_sync_buffer()`, `finalize_fast_sync()`, `DbWriteMode::{FastSync, Follow}`, `DbWriterMessage::{Block, Finalize}`, `DbWriterFinalizeStats`, `DbWriteConfig`, `ShardRemainingTracker`, `db_bytes_for_bundles()`
- **Interactions**: Calls `Storage::write_block_bundles_wal()` (fast sync) or `Storage::write_block_bundle_follow()` (follow). Follow writes gated by `BTreeMap<u64, BlockBundle>` reorder buffer. Returns finalize timing stats. Spawns per-shard compaction tasks as shards complete.
- **Knobs / invariants**: Compaction serialized with `Semaphore(1)`; finalize has two sub-phases (Compacting, Sealing) with separate progress counters; WAL writes accumulate `total_logs` in storage.

### `follow.rs`
- **Role**: Runs an infinite loop to keep the store close to the network head, including reorg detection and bounded rollback.
- **Key items**: `run_follow_loop()`, `run_head_tracker()`, `run_tail_scheduler()`, `run_reorg_detector()`, `spawn_synced_watcher()`, `HeadTrackerContext`, `ReorgDetectorContext`, `FOLLOW_POLL_MS`, `FOLLOW_NEAR_TIP_BLOCKS`, `HEAD_PROBE_PEERS`, `REORG_PROBE_PEERS`
- **Interactions**: Uses `discover_head_p2p()` to observe head; spawns per-epoch head/tail trackers and reorg detector; uses `reorg::{preflight_reorg, find_common_ancestor}` and calls `Storage::rollback_to()` when needed; uses `run_ingest_pipeline()` to fill missing blocks in follow mode.
- **Knobs / invariants**: When caught up, follow can report `SyncStatus::UpToDate` (outer follow loop: nothing to ingest) or `SyncStatus::Following` (long-lived ingest epoch: caught up and waiting). An optional one-shot "synced" signal fires on the first UpToDate/Following edge (used to start RPC via `spawn_synced_watcher()`). Reorg rollback is capped by `NodeConfig.rollback_window`.

### `reorg.rs`
- **Role**: Detects reorgs by comparing stored tip hashes against network headers and finds the latest common ancestor for rollback.
- **Key items**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`, `find_common_ancestor_number()`, `fetch_single_header()`
- **Interactions**: Called by `follow.rs` prior to ingesting new blocks.
- **Knobs / invariants**: Uses a small number of peers (`probe_peers`) and falls back to `Inconclusive` when headers are unavailable.

### `stats.rs`
- **Role**: Aggregates ingest performance metrics and writes JSON summaries and JSONL event streams.
- **Key items**: `IngestBenchStats`, `IngestBenchSummary`, `SummaryInput`, `BenchEvent`, `BenchEventLogger`, `ProcessTiming`, `RangeSummary`, `PeerSummary`, `FetchByteTotals`, `DbWriteByteTotals`
- **Interactions**: Updated by fetch/process/db stages; `BenchEventLogger` is used by `mod.rs` and `main.rs` to emit structured time-series events. Uses `metrics::{rate_per_sec, percentile_triplet}` for calculations. Uses `parking_lot::Mutex` for performance.
- **Knobs / invariants**: Sample vectors are capped (`SAMPLE_LIMIT` = 100k) to bound memory use during long runs; totals can grow when tail ranges are appended; gauge samples recorded every 10 seconds by scheduler and DB writer tasks.

### `types.rs`
- **Role**: Shared stage types for historical pipeline stages.
- **Key items**: `FetchBatch`, `FetchMode` (Normal vs Escalation)
- **Interactions**: Imported by scheduler/fetch modules.

## End-to-end flow (high level)
- Build `PeerHealthTracker` (AIMD + bans) from `NodeConfig` fast-sync parameters.
- Initialize `PeerWorkScheduler` with the missing block set and per-block attempt tracking.
- Spawn `spawn_peer_feeder()` to supply newly connected peers into a ready queue with rotation fairness.
- Spawn scheduler gauge task to emit metrics every 10 seconds.
- Optionally spawn a tail-ingest task to append new safe-head ranges into the scheduler.
- Fetch loop: pick the best available peer (quality score), assign a consecutive batch, and fetch payloads with a hard timeout.
- On success: send payloads to processing workers, mark blocks completed, record coverage/last-block-time, and update peer success.
- Processing workers: `process_ingest()` builds `BlockBundle`s with tx hashes via zero-copy KeccakBuf and sends them to the DB writer channel.
- DB writer: buffer + `write_block_bundles_wal()` (fast sync) or BTreeMap reorder buffer (follow), trigger per-shard compaction when complete, compact dirty shards and seal completed shards in finalize.
- Follow mode: periodically observe head, preflight reorgs, rollback to common ancestor when needed, and re-run ingest for missing blocks near the tip.
