# src

## Purpose
Defines the public "network surface" for reth: traits and common types that higher-level components use to interact with the networking subsystem (peer management, event streams, block download clients, discovery events), plus a noop implementation and testing helpers.

## Contents (one hop)
### Subdirectories
- [x] `test_utils/` - testing helpers for interacting with peers management in integration tests.

## Files (detailed)

### `lib.rs`
- **Role**: Main API surface: defines core network traits (info, peer management, event listeners, downloaders) and shared structs/enums for peer/session/network status.
- **Key items**: `FullNetwork`, `NetworkInfo`, `PeersInfo`, `Peers`, `PeerInfo`, `Direction`, `NetworkStatus`, `PeerId`
- **Interactions**: Implemented by `reth-network`'s concrete network manager; consumed by node wiring, RPC, and sync/pipeline orchestration.

### `downloaders.rs`
- **Role**: Trait for obtaining a `BlockClient` used to fetch blocks from peers (abstracts downloader/client wiring).
- **Key items**: `BlockDownloaderProvider`, `fetch_client()`

### `events.rs`
- **Role**: Event types and listener traits for peer lifecycle, discovery, and per-peer request routing (request/response channels for headers/bodies/txs/receipts).
- **Key items**: `PeerEvent`, `SessionInfo`, `NetworkEvent`, `PeerEventStream`, `NetworkEventListenerProvider`, `DiscoveryEvent`, `DiscoveredEvent`, `PeerRequest`
- **Knobs / invariants**: Many request variants are version-sensitive at the wire level (e.g. receipts eth/69 vs eth/70), but are exposed here as typed request/response channels.

### `error.rs`
- **Role**: Common error type for network API calls (primarily channel closure/oneshot failures).
- **Key items**: `NetworkError::ChannelClosed`

### `noop.rs`
- **Role**: `NoopNetwork` implementation that satisfies the main network traits while doing nothing, useful for tests and wiring generic components.
- **Key items**: `NoopNetwork`, `with_chain_id()`, impls for `NetworkInfo`, `Peers`, `BlockDownloaderProvider`, `NetworkEventListenerProvider`

## Key APIs (no snippets)
- **Traits**: `FullNetwork`, `NetworkInfo`, `Peers`, `NetworkEventListenerProvider`, `BlockDownloaderProvider`
- **Types**: `PeerInfo`, `NetworkStatus`, `PeerEvent` / `NetworkEvent`
