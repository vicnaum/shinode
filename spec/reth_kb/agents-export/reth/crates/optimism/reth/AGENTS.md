# reth

## Purpose
`reth-op` meta crate: feature-gated re-exports for OP reth stacks (consensus, node, RPC, storage, etc.).

## Contents (one hop)
### Subdirectories
- [x] `src/` - Feature-gated re-export surface.

### Files
- `Cargo.toml` - Manifest defining feature flags for OP reth re-exports.
  - **Key items**: features `full`, `node`, `rpc`, `consensus`, `evm`, `provider`, `cli`
