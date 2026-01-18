# src

## Purpose
Implements `reth-ethereum-forks`: Ethereum fork/hardfork types and utilities used across reth, including ordered hardfork schedules (`ChainHardforks`), fork-id/filter helpers, and human-readable hardfork schedule formatting.

## Contents (one hop)
### Subdirectories
- [x] `hardforks/` - `Hardforks` trait + `ChainHardforks` ordered list and dev-mode hardfork schedule.

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports core hardfork/forkid types from `alloy-hardforks` and `alloy-eip2124`, and exposes local helpers (`DisplayHardforks`, `ChainHardforks`, etc.).
- **Key items**: re-exports `ForkId` types, `DisplayHardforks`, `DEV_HARDFORKS` (via module re-export), and fork condition/types.
- **Interactions**: consumed by chainspec and execution components to gate behaviour by fork activation.

#### `display.rs`
- **Role**: Pretty-print utilities for a hardfork schedule: formats pre-merge / merge / post-merge forks with activation conditions and optional metadata.
- **Key items**: `DisplayHardforks`, `DisplayHardforks::new()`, `DisplayHardforks::with_meta()`

## Key APIs (no snippets)
- `Hardforks` (trait), `ChainHardforks`
- `DisplayHardforks`
- `DEV_HARDFORKS`

## Relationships
- **Builds on**: `alloy-hardforks` and EIP-2124 fork-id primitives.
