# rpc

## Purpose
`reth-rpc` crate: concrete RPC server implementations for all namespaces over JSON-RPC.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC handlers for `eth`, `debug`, `trace`, `admin`, `net`, `txpool`, etc.

### Files
- `Cargo.toml` - Manifest for RPC implementations and feature flags.
  - **Key items**: feature `js-tracer`, dependencies on `reth-rpc-eth-api`, `jsonrpsee`
