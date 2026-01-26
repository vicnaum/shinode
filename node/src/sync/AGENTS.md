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
  - **Key items**: `BlockPayload`, `SyncStatus`, `format_eta_seconds()`, `SyncProgressStats`

## Key APIs (no snippets)
- **Types / Traits**: `BlockPayload`, `ProgressReporter`, `SyncStatus`, `SyncProgressStats`, `SyncProgressSnapshot`
- **Functions**: `format_eta_seconds()`

## Relationships
- **Used by**: `node/src/main.rs` for progress rendering and follow-mode status; `node/src/sync/historical` updates `SyncProgressStats` during fetch/process/db phases.
