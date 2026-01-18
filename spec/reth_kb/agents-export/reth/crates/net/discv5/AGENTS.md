# discv5

## Purpose
`reth-discv5` crate: discovery v5 integration for reth, wrapping `sigp/discv5` with reth-friendly configuration, ENR construction (fork/network-stack keys), discovered-peer filtering, and metrics to feed upstream networking with reachable `NodeRecord`s.

## Contents (one hop)
### Subdirectories
- [x] `src/` - discv5 wrapper, config, ENR conversions, filters, and metrics.

### Files
- `Cargo.toml` - crate manifest.
  - **Key items**: `reth-network-peers` (secp256k1), `discv5` (libp2p), `reth-metrics`, `reth-chainspec`
- `README.md` - crate documentation/overview.
  - **Key items**: n/a

## Key APIs (no snippets)
- `Discv5`
- `Config` / `ConfigBuilder`
- `DiscoveredPeer`

## Relationships
- **Depends on**: `reth-network-peers` (peer IDs + `NodeRecord`), `reth-ethereum-forks` (fork IDs), `reth-chainspec` (network selection), `reth-metrics` (metrics plumbing), `discv5` (protocol implementation).
- **Used by**: higher-level network/discovery orchestration in `reth/crates/net/network*` (via the `Discv5` wrapper and its event processing).
