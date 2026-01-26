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
  - **Key items**: `DbWriteConfig`, `DbWriteMode`, `DbWriterMessage`, `DbWriterFinalizeStats`, `run_db_writer()`
- `fetch.rs` - Ingest fetch wrappers around the P2P layer (consecutive batch enforcement, ingest payload fetch).
  - **Key items**: `fetch_ingest_batch()`, `FetchIngestOutcome`, `ensure_consecutive()`
- `follow.rs` - Live follow loop: head discovery, near-tip backoff, reorg detection, rollback, and incremental ingest.
  - **Key items**: `run_follow_loop()`, `SyncStatus::UpToDate`
- `mod.rs` - Pipeline orchestration for ingest (fetch loop, processing workers, DB writer wiring, progress bars).
  - **Key items**: `run_ingest_pipeline()`, `MissingBlocks`, `TailIngestConfig`, `IngestFinalizeStats`, `IngestPipelineOutcome`, `build_peer_health_tracker()`
- `process.rs` - Processing stage: turns fetched payloads into `BlockBundle`s (tx hash/signing hash, size/log counts) and records timing.
  - **Key items**: `process_ingest()`, `KeccakBuf`, `tx_hash_fast()`, `signing_hash_fast()`, `ProcessTiming`
- `reorg.rs` - Reorg detection and common-ancestor search used by follow mode.
  - **Key items**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`
- `scheduler.rs` - Peer-driven scheduler (pending/inflight/failed/escalation) and peer health tracking (AIMD, bans, quality scoring).
  - **Key items**: `SchedulerConfig`, `PeerWorkScheduler`, `PeerHealthTracker`, `PeerHealthDump`, `FetchMode`, `enqueue_range()`
- `stats.rs` - Ingest stats aggregation and JSONL bench event writer.
  - **Key items**: `IngestBenchStats`, `IngestBenchSummary`, `BenchEvent`, `BenchEventLogger`, `add_blocks_total()`, `ProcessTiming`
- `types.rs` - Shared types for historical pipeline stages (batches, fetch modes).
  - **Key items**: `FetchBatch`, `FetchMode`

## Key APIs (no snippets)
- **Pipeline**: `run_ingest_pipeline()`, `run_follow_loop()`, `MissingBlocks`, `TailIngestConfig`
- **Scheduling**: `PeerWorkScheduler`, `PeerHealthTracker`
- **Stats**: `IngestBenchStats`, `BenchEventLogger`

## Relationships
- **Depends on**: `node/src/p2p` (network fetch), `node/src/storage` (writes and reads), `node/src/sync` (shared progress counters/status).
- **Used by**: `node/src/main.rs` (follow mode entrypoint).

## Files (detailed)

### `mod.rs`
- **Role**: Orchestrates the ingest pipeline by wiring together the scheduler, fetch tasks, processing workers, DB writer, and progress/event reporting. Supports optional tail range ingestion for moving-safe-head fast-sync.
- **Key items**: `run_ingest_pipeline()`, `MissingBlocks`, `TailIngestConfig` (fields: `stop_when_caught_up`, `head_offset`), `IngestFinalizeStats`, `IngestPipelineOutcome`
- **Interactions**:
  - Uses `fetch::fetch_ingest_batch()` to talk to the P2P layer.
  - Uses `process::process_ingest()` to build `BlockBundle`s and sends them to `db_writer`.
  - Uses `scheduler::{PeerWorkScheduler, PeerHealthTracker}` to assign work and adapt per-peer batch limits.
  - When `TailIngestConfig` is provided, enqueues appended ranges via `PeerWorkScheduler::enqueue_range()` and updates progress totals; head offset controls safe-head vs head tracking.
- **Knobs / invariants**:
  - Concurrency is bounded by `fast_sync_max_inflight`; per-batch timeout is `fast_sync_batch_timeout_ms`.
  - In follow mode, retries are unbounded (`max_attempts_per_block = u32::MAX`) to tolerate propagation lag near the tip.
  - In follow mode, scheduling caps batches by the global observed head (from head tracking), not per-peer `head_number` which can go stale.
  - In follow mode, "missing blocks" responses (including empty batches) are treated as partials to avoid banning peers for near-tip propagation lag.
  - Tail ingestion (when enabled) tracks `head_seen_rx` and stops scheduling once the safe head is caught up (or continues in follow epochs).

### `scheduler.rs`
- **Role**: Maintains the global work queue and per-peer health/quality model so scheduling adapts to real-world peer behavior.
- **Key items**: `SchedulerConfig`, `PeerWorkScheduler::next_batch_for_peer()`, `PeerWorkScheduler::enqueue_range()`, `PeerHealthTracker::record_success()`, `PeerHealthDump`
- **Interactions**: Called from `mod.rs` for batch assignment, requeue, and peer health updates; consulted for "best peer" selection via quality scores.
- **Knobs / invariants**: AIMD batch limit clamps between `aimd_min_batch` and `aimd_max_batch`; bans trigger after `peer_failure_threshold`.

### `fetch.rs`
- **Role**: Wraps P2P requests into stage-friendly outcomes for ingest mode.
- **Key items**: `FetchIngestOutcome`, `fetch_ingest_batch()`, `ensure_consecutive()`
- **Interactions**: Uses `p2p::fetch_payloads_for_peer()` for ingest.
- **Knobs / invariants**: Batches must be consecutive.

### `process.rs`
- **Role**: Converts fetched payloads into storage-ready `BlockBundle`s and records processing timings for benchmarks.
- **Key items**: `process_ingest()`, `KeccakBuf`, `KECCAK_SCRATCH_LEN`, `block_rlp_size()`, `ProcessTiming`
- **Interactions**: Feeds `BlockBundle`s to `db_writer` via `DbWriterMessage::Block`; updates `IngestBenchStats` when enabled.
- **Knobs / invariants**: Requires tx count to match receipts count; logs are counted from receipts but stored logs are currently empty (`StoredLogs { logs: Vec::new() }`).

### `db_writer.rs`
- **Role**: Applies `BlockBundle` writes to storage and manages compaction/sealing so reads work after ingest completes.
- **Key items**: `run_db_writer()`, `flush_fast_sync_buffer()`, `DbWriteMode::{FastSync, Follow}`, `DbWriterFinalizeStats`, `BenchEvent::DbFlushStart/End`, `BenchEvent::CompactionStart/End`
- **Interactions**: Calls `Storage::write_block_bundles_wal()` (fast sync) or `Storage::write_block_bundle_follow()` (follow). Follow writes are gated by an in-memory reorder buffer to enforce in-order appends. Returns finalize timing stats used by `main.rs` logging.
- **Knobs / invariants**: Compaction is serialized with a `Semaphore(1)` to cap peak IO/memory; finalize logs dirty shard/WAL sizes and runs `compact_all_dirty()` as a safety net.

### `follow.rs`
- **Role**: Runs an infinite loop to keep the store close to the network head, including reorg detection and bounded rollback.
- **Key items**: `run_follow_loop()`, `FOLLOW_POLL_MS`, `FOLLOW_NEAR_TIP_BLOCKS`, `HEAD_PROBE_PEERS`, `REORG_PROBE_PEERS`
- **Interactions**: Uses `discover_head_p2p()` to observe head; spawns per-epoch head/tail trackers to extend the ingest range as the head moves; uses `reorg::{preflight_reorg, find_common_ancestor}` and calls `Storage::rollback_to()` when needed; uses `run_ingest_pipeline()` to fill missing blocks in follow mode.
- **Knobs / invariants**: When caught up, follow can report `SyncStatus::UpToDate` (outer follow loop: nothing to ingest) or `SyncStatus::Following` (long-lived ingest epoch: caught up and waiting). An optional one-shot "synced" signal fires on the first UpToDate/Following edge (used to start RPC). Reorg rollback is capped by `NodeConfig.rollback_window`.

### `reorg.rs`
- **Role**: Detects reorgs by comparing stored tip hashes against network headers and finds the latest common ancestor for rollback.
- **Key items**: `ReorgCheck`, `preflight_reorg()`, `find_common_ancestor()`, `find_common_ancestor_number()`
- **Interactions**: Called by `follow.rs` prior to ingesting new blocks.
- **Knobs / invariants**: Uses a small number of peers (`probe_peers`) and falls back to `Inconclusive` when headers are unavailable.

### `stats.rs`
- **Role**: Aggregates ingest performance metrics and writes JSON summaries and JSONL event streams.
- **Key items**: `IngestBenchStats`, `IngestBenchSummary`, `BenchEvent`, `BenchEventLogger`, `IngestBenchStats::add_blocks_total()`, `ProcessTiming`
- **Interactions**: Updated by fetch/process/db stages; `BenchEventLogger` is used by `mod.rs` and `main.rs` to emit structured time-series events.
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
