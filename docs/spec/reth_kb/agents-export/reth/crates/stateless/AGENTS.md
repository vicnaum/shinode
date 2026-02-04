# stateless

## Purpose
`reth-stateless` crate: stateless block execution and validation using witness data.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Stateless validation pipeline, trie utilities, and witness-backed DB.

### Files
- `Cargo.toml` - Manifest for stateless validation dependencies and crypto backends.
  - **Key items**: features `k256`, `secp256k1`
