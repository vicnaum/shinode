# peers

## Purpose
`reth-network-peers` crate: peer/node record types and utilities, including bootnode lists, `NodeRecord` (enode) parsing/display, `TrustedPeer` with optional DNS resolution, and flexible `AnyNode` parsing for APIs.

## Contents (one hop)
### Subdirectories
- [x] `src/` - peer ID/node record/trusted peer types plus bootnode lists.

### Files
- `Cargo.toml` - crate manifest (feature-gated secp256k1 + async DNS lookup support).

## Key APIs (no snippets)
- `NodeRecord`
- `TrustedPeer`
- `AnyNode`
