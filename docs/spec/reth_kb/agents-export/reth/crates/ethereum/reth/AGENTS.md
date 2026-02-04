# reth

## Purpose
`reth-ethereum` crate: feature-gated meta crate that bundles and re-exports Ethereum-specific reth primitives and (optionally) node subsystems (consensus/evm/network/provider/rpc/etc.) behind a single dependency.

## Contents (one hop)
### Subdirectories
- [x] `src/` - re-export surface.

### Files
- `Cargo.toml` - crate manifest with feature flags (`full`, `node`, `rpc`, `consensus`, `evm`, `provider`, `network`, `cli`, etc.) controlling which subsystems are re-exported.

## Key APIs (no snippets)
- Re-export modules: `primitives`, `chainspec`, `cli`, `consensus`, `evm`, `network`, `provider`, `storage`, `node`, `engine`, `trie`, `rpc`
