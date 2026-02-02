# sync

## Purpose

Sync state machine and progress tracking. Defines shared types (`BlockPayload`, `SyncStatus`, `FinalizePhase`), provides atomic progress counters (`SyncProgressStats`) with coverage visualization, and re-exports the historical sync orchestration from the `historical/` submodule.

## Contents (one hop)

### Subdirectories
- [x] `historical/` - High-performance pipeline for backfilling blocks from P2P peers. Implements fast-sync and follow-mode with AIMD scheduling, parallel fetch/process/write, escalation retries, reorg detection, and bench event logging. See `historical/AGENTS.md`.

### Files
- `mod.rs` - Core sync types and progress tracking (~570 lines). Defines the sync state machine, atomic progress counters, and coverage tracker for TUI visualization.
  - **Key items**: `BlockPayload`, `SyncStatus` (LookingForPeers, Fetching, Finalizing, UpToDate, Following), `FinalizePhase` (Compacting, Sealing), `SyncProgressStats`, `SyncProgressSnapshot`, `CoverageTracker`, `ProgressReporter` trait, `format_eta_seconds`
  - **Key items** (re-exports): `run_follow_loop`, `PeerHealthTracker`, `BenchEvent`, `BenchEventLogger`, `IngestBenchStats`, `IngestBenchSummary`, `SummaryInput`

## Key APIs (no snippets)

- **Types**: `BlockPayload`, `SyncStatus`, `FinalizePhase`, `SyncProgressStats`, `SyncProgressSnapshot`, `CoverageTracker`
- **Traits**: `ProgressReporter` (set_length, inc)
- **Functions**: `format_eta_seconds()`, `SyncStatus::as_str()`, `SyncStatus::display_name()`, `SyncProgressStats::snapshot()`, `update_peak_speed_max()`, 50+ atomic setters/incrementers
- **Re-exports**: `run_follow_loop`, `PeerHealthTracker`, `BenchEvent`, `BenchEventLogger`, `IngestBenchStats`, `IngestBenchSummary`, `SummaryInput`
- **Progress fields**: `processed`, `queue`, `inflight`, `escalation`, `compactions_done`, `compactions_total`, `sealings_done`, `sealings_total`, `peers_active`, `peers_total`, `peers_stale`, `status`, `head_block`, `head_seen`, `fetch_complete`, `finalize_phase`, `start_block`, `peak_speed`, `last_block_received_ms`, `rpc_active`, `rpc_total_requests`, `rpc_get_logs`, `rpc_get_block`, `rpc_errors`, `db_blocks`, `db_transactions`, `db_logs`, `db_shards`, `db_shards_compacted`, `storage_bytes_headers`, `storage_bytes_transactions`, `storage_bytes_receipts`, `storage_bytes_total`, `coverage`
- **Escalation field**: `escalation` tracks blocks in the priority retry queue (blocks that exceeded N attempts and are being retried indefinitely with shard-aware prioritization)

## Relationships

- **Depends on**: `p2p::NetworkPeer` (for `BlockPayload`), `reth_ethereum_primitives` (Header, BlockBody, Receipt)
- **Used by**: `run/sync_runner.rs` (sync orchestration), `ui/` (progress snapshots for display), `rpc/` (request counters), `logging/` (bench stats for reports)
- **Key field**: `fetch_complete` signals when all fetch tasks are done (used by UI to transition from syncing to finalizing even if `processed < total_len` due to processing failures)
- **Data/control flow**:
  1. `SyncProgressStats` provides atomic counters updated by all sync pipeline stages
  2. `SyncProgressSnapshot` taken every 100ms by UI updater for rendering
  3. `CoverageTracker` maintains per-bucket coverage percentages for TUI blocks map
  4. `SyncStatus` drives phase transitions in both TUI and progress bar modes
  5. `historical/` module implements the actual fetch/process/write pipeline

## Notable features

- `FinalizePhase` enum for compacting vs sealing sub-phases (distinct from main compaction counters)
- `sealings_done`/`sealings_total` for separate sealing phase tracking
- `peers_stale` for cooling-down peer count
- `last_block_received_ms` for "Xs ago" TUI display in follow mode
- `rpc_active`, `rpc_total_requests`, `rpc_get_logs`, `rpc_get_block`, `rpc_errors` for RPC activity
- `db_blocks`, `db_transactions`, `db_logs`, `db_shards`, `db_shards_compacted` for DB statistics display
- `storage_bytes_headers`, `storage_bytes_transactions`, `storage_bytes_receipts`, `storage_bytes_total` for per-segment size
- `CoverageTracker` and `coverage` field for blocks map visualization (bucket-based)
- `peak_speed` and `update_peak_speed_max()` for peak throughput tracking
- `SyncStatus::Following` (value 4) added to status enum
