# types

## Purpose
`reth-exex-types` crate: shared types for ExEx usage (notifications, ExEx head/progress, and aggregated finished height for pruning), with optional serde + bincode-compat wrappers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `ExExNotification`, `ExExHead`, `FinishedExExHeight`.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-chain-state` and `reth-execution-types` for `CanonStateNotification`/`Chain` integration).

## Key APIs (no snippets)
- `ExExNotification`
- `ExExHead`
- `FinishedExExHeight`
