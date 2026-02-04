# evm

## Purpose
`reth-evm-ethereum` crate: Ethereum-specific EVM configuration for reth, bridging `reth-evm` execution traits to `alloy_evm`/`revm` Ethereum execution contexts and hardfork specs, plus block assembly and receipt building.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EthEvmConfig`, block assembler, receipt builder, revm spec helpers, and test-only mocks.
- [x] `tests/` - integration-style execution tests for fork-specific behaviours.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-evm`, `revm`, `reth-chainspec`, and Ethereum forks/primitives).

## Key APIs (no snippets)
- `EthEvmConfig`
- `EthBlockAssembler`
- `RethReceiptBuilder`
