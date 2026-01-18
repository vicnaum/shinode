# hardforks

## Purpose
`reth-ethereum-forks` crate: Ethereum fork/hardfork types and utilities used across reth (fork-id types, fork activation conditions, ordered hardfork schedules, and display helpers).

## Contents (one hop)
### Subdirectories
- [x] `src/` - fork types + schedules (`ChainHardforks`) and display helpers.

### Files
- `Cargo.toml` - crate manifest (re-exports alloy hardfork + EIP-2124 forkid types; optional `arbitrary`/`serde`).

## Key APIs (no snippets)
- `ChainHardforks`, `Hardforks`
- `DisplayHardforks`
- `DEV_HARDFORKS`
