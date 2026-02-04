# network-api

## Purpose
`reth-network-api` crate: interfaces and common types that decouple reth's networking implementation from its consumers (node wiring, RPC, sync), including peer management traits, event stream types, downloader/client providers, and a noop implementation.

## Contents (one hop)
### Subdirectories
- [x] `src/` - network traits, events, error types, noop impl, and test utilities.

### Files
- `Cargo.toml` - crate manifest (optional `serde` support for public types).
  - **Key items**: feature `serde`; deps `reth-network-types`, `reth-network-p2p`, `reth-eth-wire-types`

## Key APIs (no snippets)
- `FullNetwork`, `NetworkInfo`, `Peers`
- `NetworkEventListenerProvider`, `PeerEventStream`
- `BlockDownloaderProvider`
