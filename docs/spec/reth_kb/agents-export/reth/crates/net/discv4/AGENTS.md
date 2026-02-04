# discv4

## Purpose
`reth-discv4` crate: Discovery v4 implementation for Ethereum devp2p networking-UDP-based peer discovery with a Kademlia-like routing table, node bonding via PING/PONG, recursive `FINDNODE` lookups, and optional external/public IP detection for NAT scenarios.

## Contents (one hop)
### Subdirectories
- [x] `src/` - core discv4 protocol, config, wire encoding/decoding, and test utilities.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-network-peers`, `reth-net-nat`, `discv5` kbucket types; optional serde and test-utils feature).
- `README.md` - crate overview/documentation.

## Key APIs (no snippets)
- `Discv4`, `Discv4Service`
- `Discv4Config`, `Discv4ConfigBuilder`
