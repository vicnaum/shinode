# errors

## Purpose
`reth-errors` crate: high-level error types and re-exports for reth, consolidating consensus/execution/storage failures into a single `RethError` for ergonomic error handling across the codebase.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `RethError` definition and re-export surface.

### Files
- `Cargo.toml` - crate manifest (ties together consensus/execution/storage error crates).
  - **Key items**: dependencies on `reth-consensus`, `reth-execution-errors`, `reth-storage-errors`

## Key APIs (no snippets)
- **Type**: `RethError`
- **Alias**: `RethResult<T>`
