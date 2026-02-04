# service

## Purpose
`reth-engine-service` crate root: provides a top-level "engine service" abstraction that composes the engine-tree components into a stream/service suitable for integration into the node runtime.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EngineService` implementation and crate wiring.

### Files
- `Cargo.toml` - crate manifest for `reth-engine-service`.

## Key APIs (no snippets)
- **Exports**: `EngineService`, `EngineMessageStream`

## Relationships
- **Builds on**: `reth-engine-tree` (core engine logic) and `reth-engine-primitives` (events/messages/config surface).
