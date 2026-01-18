# revm

## Purpose
`reth-revm` crate: reth-specific adapters and helpers around `revm` execution.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Database adapters, cached reads, cancellation markers, and witness helpers.

### Files
- `Cargo.toml` - Manifest for revm utilities and feature flags.
  - **Key items**: features `witness`, `test-utils`, `serde`, `optional-checks`, `memory_limit`
