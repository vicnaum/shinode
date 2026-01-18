# consensus

## Purpose
`reth-ethereum-consensus` crate: Ethereum beacon consensus validator implementation (header/body pre-execution validation + post-execution receipt/gas/requests validation), parameterized by an Ethereum chainspec/hardfork schedule.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EthBeaconConsensus` and post-execution validation helpers.

### Files
- `Cargo.toml` - crate manifest (depends on consensus traits, chainspec/hardforks, execution types).

## Key APIs (no snippets)
- `EthBeaconConsensus`
- `validate_block_post_execution()`
