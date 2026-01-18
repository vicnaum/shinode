# evm

## Purpose
`reth-evm` crate: core EVM abstraction for reth-defines `ConfigureEvm` and execution/building traits used by nodes and payload builders to execute blocks and build new blocks, with optional Engine API helpers and metrics.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `ConfigureEvm`, execution/building helpers, engine extensions, and metrics.

### Files
- `Cargo.toml` - crate manifest (depends on `revm` + `alloy_evm` and reth execution error/type crates; optional metrics support).

## Key APIs (no snippets)
- `ConfigureEvm`
- `Executor`
- `ConfigureEngineEvm` (feature `std`)
