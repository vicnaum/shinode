# builder

## Purpose
`reth-node-builder` crate: declarative, type-safe node builder with component wiring, launch orchestration, and RPC add-ons.

## Contents (one hop)
### Subdirectories
- [x] `docs/` - (skip: diagrams/docs only) mermaid diagram for builder flow.
- [x] `src/` - Builder core, component traits, launchers, and RPC wiring.

### Files
- `Cargo.toml` - Manifest with node builder dependencies and feature flags.
  - **Key items**: features `js-tracer`, `test-utils`, `op`; deps `reth-network`, `reth-rpc`, `reth-engine-tree`
- `README.md` - Short crate description and example pointer.

## Key APIs (no snippets)
- `NodeBuilder`, `NodeComponentsBuilder`
- `EngineNodeLauncher`, `DebugNodeLauncher`
