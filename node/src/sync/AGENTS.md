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
  - **Key items**: `BlockPayload`, `SyncStatus`, `format_eta_seconds()`, `SyncProgressStats`, `SyncProgressSnapshot`

## Key APIs (no snippets)
- **Types / Traits**: `BlockPayload`, `ProgressReporter`, `SyncStatus`, `SyncProgressStats`, `SyncProgressSnapshot`
- **Functions**: `format_eta_seconds()`, `SyncStatus::display_name()`
- **Progress fields**: `processed`, `queue`, `inflight`, `escalation`, `compactions_done`, `compactions_total`, `peers_active`, `peers_total`, `status`, `head_block`, `head_seen`, `fetch_complete`
- **Escalation field**: `escalation` tracks blocks in the priority retry queue (blocks that exceeded N attempts and are being retried indefinitely with shard-aware prioritization)

## Relationships
- **Used by**: `node/src/main.rs` for progress rendering and follow-mode status; `node/src/ui` for progress bar state; `node/src/sync/historical` updates `SyncProgressStats` during fetch/process/db phases.
- **Key field**: `fetch_complete` signals when all fetch tasks are done (used by UI to transition from syncing to finalizing even if `processed < total_len` due to processing failures).
