# network-types

## Purpose
`reth-network-types` crate: commonly used networking types shared across reth (peer model and configuration, reputation/backoff, session limits/timeouts), intended to keep higher-level networking components decoupled from implementation details.

## Contents (one hop)
### Subdirectories
- [x] `src/` - peer/session/backoff types and configuration.

### Files
- `Cargo.toml` - crate manifest (optional `serde` support for config types).
  - **Key items**: feature `serde`; deps `reth-network-peers`, `reth-net-banlist`, `alloy-eip2124`

## Key APIs (no snippets)
- `Peer`, `PeerAddr`, `PeerKind`
- `PeersConfig`, `ConnectionsConfig`
- `SessionsConfig`, `SessionLimits`
- `Reputation`, `ReputationChangeKind`, `BackoffKind`
