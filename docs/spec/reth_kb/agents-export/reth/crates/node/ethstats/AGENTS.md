# ethstats

## Purpose
`reth-node-ethstats` crate: EthStats client service for reporting node and chain stats over WebSocket.

## Contents (one hop)
### Subdirectories
- [x] `src/` - EthStats service, protocol message types, and websocket helpers.

### Files
- `Cargo.toml` - Manifest for EthStats networking and serialization dependencies.
  - **Key items**: deps `reth-network-api`, `reth-chain-state`, `tokio-tungstenite`, `serde`
