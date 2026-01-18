# primitives

## Purpose
`reth-engine-primitives` crate root: shared types and traits that define reth's Engine API message/event surface, forkchoice tracking, engine-tree configuration, and extension hooks.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Engine API messages (`BeaconEngineMessage`), handles/futures, events, forkchoice tracking, `TreeConfig`, and invalid-block hook traits.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-primitives` (features and dependencies).

## Key APIs (no snippets)
- **Exports**: `BeaconEngineMessage`, `ConsensusEngineHandle`, `ConsensusEngineEvent`, `ForkchoiceStateTracker`, `ForkchoiceStatus`, `TreeConfig`, `EngineTypes`

## Relationships
- **Used by**: engine crates under `reth/crates/engine/` (especially `tree/` and `service/`).
