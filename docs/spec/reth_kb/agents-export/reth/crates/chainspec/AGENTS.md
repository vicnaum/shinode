# chainspec

## Purpose
`reth-chainspec` crate: defines the spec of an Ethereum network (chain identity, genesis, and hardfork schedule) and exposes shared chain-spec query traits used across reth.

## Contents (one hop)
### Subdirectories
- [x] `src/` - ChainSpec types/builders, chain presets, and `EthChainSpec` trait.
- [x] `res/` - (skip: static assets) Genesis JSON inputs used to build built-in chain specs.

### Files
- `Cargo.toml` - crate manifest (features like `std`, `arbitrary`, `test-utils`).

## Key APIs (no snippets)
- **Traits**: `EthChainSpec`, `ChainSpecProvider`
- **Types**: `ChainSpec`, `ChainSpecBuilder`, `ChainInfo`
- **Presets**: `MAINNET`, `SEPOLIA`, `HOLESKY`, `HOODI`, `DEV`

## Relationships
- **Used by**: consensus/validation, engine, EVM execution configuration, network status/handshakes, and RPC where fork/chain params are required.
