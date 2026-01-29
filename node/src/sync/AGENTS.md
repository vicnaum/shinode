# sync

## Purpose
Sync and ingest orchestration primitives shared by historical backfill and follow mode. Defines the
block payload model, progress reporting hooks, and atomic progress counters used by the UI and
benchmark event streams.

## Contents (one hop)
### Subdirectories
- [x] `historical/` - Historical pipeline (ingest/follow), peer scheduler + health tracking, and benchmark stats/event logging.

### Files
- `mod.rs` - Core sync types and progress tracking utilities.
  - **Key items**: `BlockPayload`, `ProgressReporter`, `SyncStatus`, `FinalizePhase`, `CoverageTracker`, `SyncProgressStats`, `SyncProgressSnapshot`, `format_eta_seconds()`

## Key APIs (no snippets)
- **Types / Traits**: `BlockPayload`, `ProgressReporter`, `SyncStatus`, `FinalizePhase`, `CoverageTracker`, `SyncProgressStats`, `SyncProgressSnapshot`
- **Functions**: `format_eta_seconds()`, `SyncStatus::as_str()`, `SyncStatus::display_name()`
- **Progress fields**: `processed`, `queue`, `inflight`, `escalation`, `compactions_done`, `compactions_total`, `sealings_done`, `sealings_total`, `peers_active`, `peers_total`, `peers_stale`, `status`, `head_block`, `head_seen`, `fetch_complete`, `finalize_phase`, `start_block`, `peak_speed`, `last_block_received_ms`, `rpc_active`, `rpc_total_requests`, `rpc_get_logs`, `rpc_get_block`, `rpc_errors`, `db_blocks`, `db_transactions`, `db_logs`, `db_shards`, `db_shards_compacted`, `storage_bytes_*`, `coverage`
- **Escalation field**: `escalation` tracks blocks in the priority retry queue (blocks that exceeded N attempts and are being retried indefinitely with shard-aware prioritization)

## Relationships
- **Used by**: `node/src/main.rs` for progress rendering and follow-mode status; `node/src/ui` for progress bar state and TUI stats panels; `node/src/sync/historical` updates `SyncProgressStats` during fetch/process/db phases; `node/src/rpc` for request counters.
- **Key field**: `fetch_complete` signals when all fetch tasks are done (used by UI to transition from syncing to finalizing even if `processed < total_len` due to processing failures).

## New in this version
- `FinalizePhase` enum for compacting vs sealing sub-phases (distinct from main compaction counters)
- `sealings_done`/`sealings_total` for separate sealing phase tracking
- `peers_stale` for cooling-down peer count
- `last_block_received_ms` for "Xs ago" TUI display in follow mode
- `rpc_active`, `rpc_total_requests`, `rpc_get_logs`, `rpc_get_block`, `rpc_errors` for RPC activity
- `db_blocks`, `db_transactions`, `db_logs`, `db_shards`, `db_shards_compacted` for DB statistics display
- `storage_bytes_headers`, `storage_bytes_transactions`, `storage_bytes_receipts` for per-segment size
- `CoverageTracker` and `coverage` field for blocks map visualization (bucket-based)
- `peak_speed` and `update_peak_speed_max()` for peak throughput tracking
- `SyncStatus::Following` (value 4) added to status enum
