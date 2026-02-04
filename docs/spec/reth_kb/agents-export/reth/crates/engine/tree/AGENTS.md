# tree

## Purpose
`reth-engine-tree` crate root: packages the engine-tree implementation that keeps the node in sync during live operation (Engine API), can trigger backfill pipeline runs for large gaps, and persists executed data off the critical path.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - benchmarks for engine-tree hot paths (channels, state-root task).
- [x] `docs/` - design docs and diagrams (Mermaid sources + rendered images).
- [x] `src/` - implementation of engine-tree (handlers, orchestrator, backfill, download, persistence, in-memory tree).
- [x] `tests/` - integration/e2e tests for engine-tree behavior.
- [x] `test-data/` - (skip: test fixtures only; contains `.rlp` sample payloads)

### Files
- `Cargo.toml` - crate manifest for `reth-engine-tree` (features, deps, benches/tests registration).

## Key APIs (no snippets)
- **Primary modules**: `src/lib.rs` (overview), `src/engine.rs` (Engine API handler), `src/tree/` (tree core), `src/persistence.rs` (background persistence), `src/backfill.rs` (pipeline sync)

## Relationships
- **Used by**: `reth-engine-service` (wires engine-tree into a pollable service for the node).
- **Depends on**: `reth-engine-primitives` (message/event/config types), `reth-provider` (DB/provider), `reth-network-p2p` (block downloads), `reth-stages-api` (pipeline control flow).
