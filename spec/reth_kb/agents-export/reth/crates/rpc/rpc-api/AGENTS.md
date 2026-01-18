# rpc-api

## Purpose
`reth-rpc-api` crate: JSON-RPC interface definitions and trait exports for all namespaces.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC trait definitions for admin, engine, eth, trace, etc.

### Files
- `Cargo.toml` - Manifest for RPC interface traits and client feature.
  - **Key items**: feature `client`, `jsonrpsee` macros, `reth-rpc-eth-api`
