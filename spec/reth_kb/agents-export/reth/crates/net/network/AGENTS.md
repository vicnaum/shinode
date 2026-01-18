# network

## Purpose
`reth-network` crate: full P2P networking stack for Ethereum (discovery, sessions, peer management, request routing, transaction gossip, and network APIs).

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Criterion benchmarks for broadcast ingress and transaction hash fetching.
- [x] `docs/` - (skip: diagrams/docs only) mermaid diagrams for fetch client, swarm, and network manager.
- [x] `src/` - Core networking implementation: discovery, sessions, peers, state, request handlers, and transactions.
- [x] `tests/` - Integration tests for connections, requests, sessions, and gossip policies.

### Files
- `Cargo.toml` - Crate manifest with networking, discovery, and transaction-pool dependencies plus test-utils features.
  - **Key items**: features `serde`, `test-utils`; deps `reth-network-p2p`, `reth-discv4`, `reth-discv5`, `reth-dns-discovery`, `reth-eth-wire`, `reth-transaction-pool`
- `README.md` - Short crate-level description.
  - **Key items**: `RETH network implementation`

## Key APIs (no snippets)
- `NetworkManager`, `NetworkHandle`
- `NetworkConfig`, `NetworkBuilder`
- `PeersManager`, `SessionManager`
