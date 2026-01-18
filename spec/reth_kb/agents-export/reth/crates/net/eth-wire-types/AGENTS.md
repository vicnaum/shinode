# eth-wire-types

## Purpose
`reth-eth-wire-types` crate: strongly typed payloads and helpers for the devp2p `eth` wire protocol (eth/66-eth/70) plus related message families (broadcasts, capabilities, disconnect reasons, and SNAP), with version-aware decoding/encoding and pluggable "network primitives" abstractions.

## Contents (one hop)
### Subdirectories
- [x] `src/` - message payload types, IDs, and versioned encoding/decoding for `eth`/SNAP.

### Files
- `Cargo.toml` - crate manifest and feature flags.
  - **Key items**: features `std` (default), `serde`, `arbitrary`; deps `reth-ethereum-primitives`, `reth-primitives-traits`, `alloy-rlp`, `alloy-consensus`, `alloy-eips`

## Key APIs (no snippets)
- `EthMessage`, `EthMessageID`, `ProtocolMessage`
- `EthVersion`, `ProtocolVersion`
- `NetworkPrimitives` / `EthNetworkPrimitives`
