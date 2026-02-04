# src

## Purpose
Shared "plumbing" types used across reth networking: peer model/state/configuration (addresses, kinds, reputation, backoff, bans) and session manager configuration/limits.

## Contents (one hop)
### Subdirectories
- [x] `peers/` - peer address/kind/state, reputation model, and peering configuration.
- [x] `session/` - session manager configuration: timeouts, buffers, and limits.

### Files
- `lib.rs` - crate entrypoint: exports the common peer/session types and config from submodules.
  - **Key items**: `Peer`, `PeerAddr`, `PeerKind`, `Reputation`, `PeersConfig`, `SessionsConfig`, `BackoffKind`
- `backoff.rs` - backoff severity classification used by peering logic.
  - **Key items**: `BackoffKind::{Low,Medium,High}`, `BackoffKind::is_severe()`

## Key APIs (no snippets)
- `Peer`, `PeerAddr`, `PeerKind`, `PeerConnectionState`
- `PeersConfig`, `SessionsConfig`
- `ReputationChangeKind`, `BackoffKind`
