# src

## Purpose
Core implementation of the `reth-engine-tree` crate: orchestrates live Engine API handling, backfill (pipeline) sync, on-demand block downloads, in-memory chain tracking, and background persistence.

## Contents (one hop)
### Subdirectories
- [x] `tree/` - Engine API execution/validation core: in-memory `TreeState`, payload validation, state-root pipeline, buffering, caching, and engine metrics.

### Files
- `backfill.rs` - backfill sync types (`BackfillSyncState`, `BackfillAction`, `BackfillEvent`) and `PipelineSync` wrapper for running the stages pipeline.
- `chain.rs` - `ChainOrchestrator` state machine that coordinates a `ChainHandler` with a `BackfillSync` implementation.
- `download.rs` - `BlockDownloader` trait and `BasicBlockDownloader` implementation for fetching full blocks/ranges on demand.
- `engine.rs` - Engine API-facing handler types: `EngineHandler`, `EngineRequestHandler`, `EngineApiRequestHandler`, request/event enums, and download requests.
- `lib.rs` - crate entrypoint and module wiring; documents design goals (in-memory critical path, persistence off-thread).
- `metrics.rs` - metrics structs for downloader and persistence services.
- `persistence.rs` - persistence service (`PersistenceService`, `PersistenceAction`, `PersistenceHandle`) that writes blocks/removals to DB/static files and triggers pruning.
- `test_utils.rs` - helpers for engine-tree tests (test pipeline builder, test header insertion).

## Key APIs (no snippets)
- **Types**: `ChainOrchestrator`, `EngineHandler`, `EngineApiRequestHandler`, `PipelineSync`, `BasicBlockDownloader`, `PersistenceService`, `PersistenceHandle`
- **Traits**: `ChainHandler`, `BackfillSync`, `BlockDownloader`, `EngineRequestHandler`
- **Enums**: `BackfillAction`, `BackfillEvent`, `BackfillSyncState`, `DownloadRequest`, `DownloadOutcome`, `EngineApiKind`

## Relationships
- **Used by**: `reth/crates/engine/service/src/service.rs` (wraps the engine-tree orchestrator as a stream service).
- **Depends on**: `reth-engine-primitives` (Engine API message/event/config), `reth-stages-api` (pipeline targets/control flow), `reth-network-p2p` (block client), `reth-provider` (DB/provider factories), `reth-prune` (pruning during persistence).
- **Data/control flow**:
  - Incoming Engine API messages -> `engine.rs` handler -> `tree/` processing
  - Missing blocks -> `download.rs` downloader -> blocks returned to handler
  - Large gaps -> `backfill.rs` pipeline sync -> `chain.rs` orchestrator coordinates exclusive DB access
  - Persisted outputs -> `persistence.rs` background thread -> pruning + sync height metrics

## Notes
- The `chain.rs` orchestrator documents DB write-lock invariants: backfill/persistence must not deadlock with live handler writes.
