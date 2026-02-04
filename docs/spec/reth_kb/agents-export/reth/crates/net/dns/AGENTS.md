# dns

## Purpose
`reth-dns-discovery` crate: EIP-1459 DNS-based node discovery-resolves and verifies ENR tree records via DNS TXT lookups, rate-limits queries, incrementally syncs trees, and emits discovered peers as `NodeRecord` updates.

## Contents (one hop)
### Subdirectories
- [x] `src/` - DNS tree parsing/verification, query pool + resolver abstraction, and discovery service.

### Files
- `Cargo.toml` - crate manifest (optional `serde` support).
  - **Key items**: feature `serde`; deps `hickory-resolver`, `reth-network-peers`, `reth-tokio-util` (ratelimit)

## Key APIs (no snippets)
- `DnsDiscoveryService`, `DnsDiscoveryHandle`
- `DnsNodeRecordUpdate`
- `DnsDiscoveryConfig`
