# Temporary plan (bench + fast-sync perf)

This file captures **agreed next changes** and the supporting instrumentation needed to validate them.

## 1) Benchmark: `--benchmark-min-peers` (default 10)

### Goal
Benchmarks should **not start fast-sync ingest/probe until we have enough usable peers**, otherwise the first ~minutes run under-parallelized and distort results.

### Proposed behavior
- Add CLI flag: `--benchmark-min-peers <N>`
- **Default**:
  - If `--benchmark=ingest|probe`: default `N=10`
  - If `--benchmark=disabled`: ignore (or treat as `N=MIN_PEER_START`)
- Before starting the benchmark pipeline, wait until `PeerPool.len() >= N` (with periodic progress logging).

### Where
- `node/src/cli/mod.rs`
  - Add field `benchmark_min_peers: Option<u32>` (or `u64`), clap arg `--benchmark-min-peers`.
- `node/src/main.rs` (or `node/src/sync/historical/mod.rs` at the benchmark entrypoint)
  - If benchmark enabled: wait for `pool.len() >= benchmark_min_peers_effective`.
  - Optional: add a timeout (e.g. 30–60s) and warn if not met, but still proceed.

## 2) Peer warmup throughput: avoid blocking on head-probe

### Motivation
`spawn_peer_watcher` currently **awaits** `request_head_number()` inside the network event loop. If a peer is slow/unresponsive, this can block processing of subsequent `ActivePeerSession` events, slowing pool growth.

### Proposed change
- On `NetworkEvent::ActivePeerSession`:
  - Add peer to pool immediately with `head_number=0`.
  - Spawn a task (bounded by a semaphore) to probe the head number and then update the pool entry.
- Keep `discover_head_p2p` behavior (it already does “don’t trust head_number” logic for follow).

### Where
- `node/src/p2p/mod.rs`
  - Refactor `spawn_peer_watcher` to not block on `request_head_number`.
  - Add a small concurrency limit for head-probes (e.g. 16–32).
  - Consider a dedicated shorter timeout for head-probes vs regular request timeout.

## 3) Fast-sync DB writer: batch WAL writes (follow stays per-block)

### Requirements
- **Fast-sync only**: batch writes to WAL are OK and don’t need ordering.
- **Follow mode**: must remain per-block (12s cadence; correctness/simplicity preferred).

### Proposed approach
1. In `run_db_writer` (fast-sync mode only):
   - Buffer incoming `BlockBundle`s up to `config.batch_blocks` (existing `--db-write-batch-blocks`, default 512).
   - Flush the buffer when:
     - `buffer.len() >= batch_blocks`, or
     - `DbWriterMessage::Flush|Finalize`, or
     - optional time-based flush (`config.flush_interval`).
2. Add an efficient batch WAL append in storage:
   - Write multiple records with **one file open** + **one flush** per shard per batch.
   - Keep out-of-order correctness (WAL compaction already maps by block number).

### Where
- `node/src/sync/historical/db_writer.rs`
  - Implement buffering only when `mode == DbWriteMode::FastSync`.
  - Update bench/event reporting to record batch sizes (`blocks=N`) instead of always `1`.
- `node/src/storage/sharded/wal.rs`
  - Add `append_records(path, records: &[(block_number, payload)])`.
- `node/src/storage/sharded/mod.rs`
  - Add `write_block_bundles_wal(&self, bundles: &[BlockBundle]) -> Result<usize>` that:
    - groups bundles by shard,
    - locks each shard once,
    - appends all WAL records in one go,
    - updates bitsets/meta once per batch.

### Validation
- Re-run the 100k benchmark with same fetch params and compare:
  - `results.db.flush.*` (should drop meaningfully)
  - overall `performance.elapsed_ms`
  - fetch parallelism and peer ramp (should not regress)

## 4) Metrics: compaction/sealing (to validate “batch write == compaction/seal”)

### Gap
We currently **do not measure** `compact_shard`, `compact_all_dirty`, or `seal_completed_shards` durations directly in bench stats/events.

### Proposed instrumentation
- Add new bench events + stats counters:
  - `CompactionStart/End { shard_start, duration_ms }`
  - `CompactAllDirtyStart/End { duration_ms }`
  - `SealCompletedStart/End { duration_ms }`

### Where
- `node/src/sync/historical/stats.rs`: extend `BenchEvent`
- `node/src/sync/historical/db_writer.rs`: record around compaction/seal calls

