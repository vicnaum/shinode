# execution-types

## Purpose
`reth-execution-types` crate: shared types for representing EVM block execution results and outcomes (per-block `BlockExecutionResult`/`BlockExecutionOutput`, multi-block `ExecutionOutcome`, and `Chain` containers used by fork/tree logic).

## Contents (one hop)
### Subdirectories
- [x] `src/` - execution output/outcome types and the `Chain` container.

### Files
- `Cargo.toml` - crate manifest (ties together `revm` bundle state, trie helpers, and primitive types; optional serde/bincode compatibility).

## Key APIs (no snippets)
- `BlockExecutionResult`, `BlockExecutionOutput`
- `ExecutionOutcome`
- `Chain`
