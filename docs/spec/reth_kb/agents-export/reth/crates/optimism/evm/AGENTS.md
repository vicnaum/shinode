# evm

## Purpose
`reth-optimism-evm` crate: OP-specific EVM configuration, execution helpers, and block assembly.

## Contents (one hop)
### Subdirectories
- [x] `src/` - EVM config, L1 info parsing, receipt building, and block assembly.

### Files
- `Cargo.toml` - Manifest for OP EVM and execution dependencies.
  - **Key items**: deps `reth-evm`, `op-alloy-evm`, `reth-optimism-consensus`; features `portable`, `rpc`
