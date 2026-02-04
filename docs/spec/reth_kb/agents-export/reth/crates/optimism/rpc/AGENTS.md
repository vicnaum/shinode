# rpc

## Purpose
`reth-optimism-rpc` crate: OP-specific RPC implementations for engine, eth, miner, sequencer, and debug/witness endpoints.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP RPC servers, errors, sequencer client, historical forwarding, and eth wrappers.

### Files
- `Cargo.toml` - Manifest for OP RPC dependencies and client feature.
  - **Key items**: feature `client`; deps `reth-rpc-eth-api`, `reth-rpc-engine-api`, `reth-optimism-flashblocks`, `op-alloy-rpc-types-engine`
