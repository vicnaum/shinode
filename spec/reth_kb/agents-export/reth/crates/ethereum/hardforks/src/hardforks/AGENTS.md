# hardforks

## Purpose
Hardfork schedule containers and traits for `reth-ethereum-forks`: defines how to represent an ordered set of hardforks, query activation conditions, and derive fork-id/filter information.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - core hardfork set API: `Hardforks` trait and `ChainHardforks` ordered list with insertion/ordering helpers.
  - **Key items**: `Hardforks` (trait), `ChainHardforks`, `ChainHardforks::new()`, `insert()`, `forks_iter()`, `fork_id()`, `fork_filter()`
- `dev.rs` - dev-mode hardfork schedule: activates essentially all Ethereum hardforks at block/timestamp 0 (with Merge TTD = 0).
  - **Key items**: `DEV_HARDFORKS`
