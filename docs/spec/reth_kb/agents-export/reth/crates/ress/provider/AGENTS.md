# provider

## Purpose
`reth-ress-provider` crate: reth-backed provider for serving RESS protocol requests.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RESS provider implementation and pending-state tracking.

### Files
- `Cargo.toml` - Manifest for RESS provider dependencies.
  - **Key items**: deps `reth-ress-protocol`, `reth-revm`, `reth-trie`, `reth-storage-api`
