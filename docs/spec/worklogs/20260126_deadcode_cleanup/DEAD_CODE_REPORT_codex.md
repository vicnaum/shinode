# Dead/Unused/Legacy Code Report (Node Repo)

Scope: focused on the Rust node implementation under `node/` (plus a few repo-level notes).
Method: `rg` scans for `#[allow(dead_code)]`, then spot-checking actual call sites/usages.

## High-Confidence Dead Code (No Call Sites / Only Referenced by Itself)

- `node/src/metrics.rs:7` `lag_to_head()` + `node/src/metrics.rs:15` `rate_per_sec()`
  - Both are `pub` and have unit tests, but are not referenced anywhere in the crate.

- `node/src/main.rs:2257` `total_blocks_to_head()`
  - Unused helper (range size to head).

- Probe-mode leftovers (probe still exists, but these specific items are not used by it):
  - `node/src/sync/historical/types.rs:13` `ProcessingMode` (unused enum)
  - `node/src/sync/historical/types.rs:65` `FetchedBlock` (unused struct)
  - `node/src/sync/historical/process.rs:21` `process_probe()` (unused function)

- Unused P2P selector scaffolding (looks like early testing hooks):
  - `node/src/p2p/mod.rs:40` `SelectorPeerId`
  - `node/src/p2p/mod.rs:44` `PeerSelector`
  - `node/src/p2p/mod.rs:52` `RoundRobinPeerSelector` (+ `impl` block at `node/src/p2p/mod.rs:63`)

- DB writer message variant that is never sent:
  - `node/src/sync/historical/db_writer.rs:151` `DbWriterMessage::Flush`

## Likely Dead / Debug-Only Helpers (Suppressed via `#[allow(dead_code)]`)

These are not referenced by the current fast-sync/follow pipelines, and appear to be kept for
future optimizations or debugging tooling.

- WAL reader that loads full payloads (current code prefers WAL indexing / selective reads):
  - `node/src/storage/sharded/wal.rs:44` `read_records()`

- WAL index fields currently unused:
  - `node/src/storage/sharded/wal.rs:19` `WalIndexEntry.record_offset`
  - `node/src/storage/sharded/wal.rs:22` `WalIndexEntry.payload_len`

- Storage compaction helpers / verification tooling that isn’t called:
  - `node/src/storage/sharded/mod.rs:218` `SegmentWriter::append_rows_no_commit()`
  - `node/src/storage/sharded/mod.rs:224` `SegmentWriter::commit()`
  - `node/src/storage/sharded/mod.rs:997` `Storage::compact_all_dirty()`
  - `node/src/storage/sharded/mod.rs:1711` `records_to_map()`
  - `node/src/storage/sharded/mod.rs:1734` `WalPayloadCrcReader` (+ impls)
  - `node/src/storage/sharded/mod.rs:1781` `read_wal_bundle_record_at()`

- Storage meta mutator that appears unused (only internal reads are used):
  - `node/src/storage/sharded/mod.rs:462` `Storage::set_last_indexed_block()`

- Scheduler internals that are unused after recent follow-mode refactors:
  - `node/src/sync/historical/scheduler.rs:22` `SchedulerConfig.max_lookahead_blocks`
  - `node/src/sync/historical/scheduler.rs:46` `PeerHealthConfig.partial_threshold_multiplier`
  - `node/src/sync/historical/scheduler.rs:48` `PeerHealthConfig.partial_ban_duration`
  - `node/src/sync/historical/scheduler.rs:115` `PeerHealth::ban_remaining()`
  - `node/src/sync/historical/scheduler.rs:456` `PeerWorkScheduler.low_watermark`
  - `node/src/sync/historical/scheduler.rs:838` `PeerWorkScheduler::completed_count()`
  - `node/src/sync/historical/scheduler.rs:878` `PeerWorkScheduler::failed_count()`

- Stats helpers that aren’t called (all data is read via `summary()` instead):
  - `node/src/sync/historical/stats.rs:86` `ProbeStats::receipts_total()`
  - `node/src/sync/historical/stats.rs:91` `ProbeStats::elapsed_ms()`
  - `node/src/sync/historical/stats.rs:461` `IngestBenchStats::logs_total()`
  - `node/src/sync/historical/stats.rs:823` `DbWriteByteTotals::add()`

## Stubs / Legacy-Looking Code (Compiled, But Not Meaningfully Implemented)

These match “not implemented yet” features in README/SPEC/PRD, or are placeholders that likely
need either full implementation or removal to reduce confusion.

- Withdrawals are explicitly not stored (and `withdrawals` are always null in RPC), but storage
  exposes a stub API:
  - `node/src/storage/sharded/mod.rs:1188` `Storage::block_withdrawals()` returns `Ok(None)`

- Logs persistence and log indexes are not implemented (logs are derived from receipts), but there
  are stub entrypoints:
  - `node/src/storage/sharded/mod.rs:1234` `Storage::block_logs()` returns `Ok(None)`
  - `node/src/storage/sharded/mod.rs:1278` `Storage::block_logs_range()` returns empty
  - `node/src/storage/sharded/mod.rs:1286` `Storage::log_index_by_address_range()` returns empty
  - `node/src/storage/sharded/mod.rs:1295` `Storage::log_index_by_topic0_range()` returns empty

- Legacy eth protocol compatibility paths (may be necessary, but worth calling out):
  - `node/src/p2p/mod.rs:915`/`:926` fall back to `request_receipts_legacy()` /
    `request_receipt_counts_legacy()` for non-eth/69,70 peers.

## “Repeating” / Refactor Candidates (Not Dead, But Looks Duplicative)

- Repeated “bitset check + open segment readers + read row” patterns across storage accessors:
  - `node/src/storage/sharded/mod.rs` (e.g. `block_header`, `block_tx_hashes`, `block_receipts`, etc.)
  - Suggestion: a small internal helper that:
    1) resolves shard + checks bitset
    2) loads `shard_segment_readers`
    3) runs a closure to read from the chosen segment
    would remove a lot of repeated boilerplate and reduce bug surface.

- Multiple WAL parsing strategies co-exist:
  - Full scan+load (`read_records`) vs index-based reads + optional CRC tooling.
  - Suggestion: keep exactly one “production path” and move the rest under a clearly labeled
    `debug/` module or behind a feature flag to reduce maintenance overhead.

## Suspicious / Possibly Superfluous `#[allow(dead_code)]`

These types currently appear to be used by ingest/RPC, so the `#[allow(dead_code)]` attributes look
like historical carry-over rather than actively needed.

- `node/src/storage/mod.rs`: many `Stored*` types are `#[allow(dead_code)]` but referenced by
  `process_ingest`, RPC formatting, and/or storage accessors.
- `node/src/storage/sharded/mod.rs:330` `DirtyShardInfo.sorted` / `DirtyShardInfo.complete`
  are still suppressed but no longer referenced (could be removed or logged for debugging).

## Repo-Level Note

- There are multiple “projects” in this repo (`node/`, `harness/`, `rindexer/`, `reth/`) with
  overlapping concerns. That’s intentional per `SPEC.md` / `PRD.md`, but it makes “dead code”
  judgments ambiguous unless we define whether we mean:
  - “unused by the node binary at runtime” vs
  - “unused by any tool in this repo” vs
  - “not required by the v0.2 product scope.”

