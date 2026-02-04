# engine

## Purpose
Top-level engine subsystem directory: contains the crates that implement reth's Engine API handling, live-sync "engine tree", orchestration/persistence wiring, and supporting utilities for testing/debugging.

## Contents (one hop)
### Subdirectories
- [x] `invalid-block-hooks/` - Debugging hooks for invalid blocks (e.g., witness generation via `InvalidBlockHook`).
- [x] `local/` - Local/dev-chain components that generate Engine API traffic (`LocalMiner`, payload attributes builder).
- [x] `primitives/` - Shared Engine API types/traits/events/config (`BeaconEngineMessage`, `ConsensusEngineEvent`, `TreeConfig`).
- [x] `service/` - `EngineService` wiring that composes engine-tree + downloader + pipeline backfill into a pollable service.
- [x] `tree/` - Core `reth-engine-tree` implementation (Engine API request handling, in-memory tree state, backfill, downloads, persistence).
- [x] `util/` - Stream utilities around engine message streams (store/replay/skip/reorg simulation).

### Files
- (none)

## Key APIs (no snippets)
- **Main building blocks**:
  - `tree/` - `EngineApiTreeHandler`, `BasicEngineValidator`, `ChainOrchestrator`, `PipelineSync`, `PersistenceHandle`
  - `service/` - `EngineService`
  - `primitives/` - `BeaconEngineMessage`, `ConsensusEngineHandle`, `ConsensusEngineEvent`, `TreeConfig`

## Relationships
- **Integrated by**: node launch/wiring layers outside this subtree (Engine API RPC ingress -> engine message stream -> engine service/tree).
- **Design intent**: keep Engine API responsiveness high by minimizing DB writes on the critical path and delegating persistence/pruning to background workers.
