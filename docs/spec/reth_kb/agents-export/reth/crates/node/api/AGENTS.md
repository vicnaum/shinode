# api

## Purpose
`reth-node-api` crate: shared traits and abstractions for configuring node components, payload builders, and add-on services.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Node API traits and re-exports for engine/payload/EVM primitives.

### Files
- `Cargo.toml` - Manifest defining dependencies for node component traits and add-ons.
  - **Key items**: deps `reth-node-types`, `reth-network-api`, `reth-transaction-pool`, `reth-engine-primitives`
