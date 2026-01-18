# test_utils

## Purpose
Network test helpers: utilities for free ports/peer IDs, an in-process testnet harness, and helpers for transaction manager tests.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test-utils module wiring and re-exports for network tests.
- **Key items**: `Testnet`, `Peer`, `PeerConfig`, `PeerHandle`, `NetworkEventStream`, `unused_tcp_udp()`, `new_tx_manager()`

### `init.rs`
- **Role**: Low-level helpers to obtain unused TCP/UDP ports/addresses and convert ENR to `PeerId`.
- **Key items**: `enr_to_peer_id()`, `unused_port()`, `unused_tcp_addr()`, `unused_udp_addr()`, `unused_tcp_and_udp_port()`, `unused_tcp_udp()`
- **Knobs / invariants**: Port helpers do not reserve ports; they only check availability at call time.

### `testnet.rs`
- **Role**: Test network harness that spins up multiple `NetworkManager` peers with optional transaction pools and request handlers.
- **Key items**: `Testnet`, `TestnetHandle`, `Peer`, `PeerConfig`, `PeerHandle`, `NetworkEventStream`
- **Interactions**: Builds `NetworkManager`, `TransactionsManager`, and `EthRequestHandler`; uses event streams to await session establishment.
- **Knobs / invariants**: Convenience builders for eth pool configuration and transaction propagation policy.

### `transactions.rs`
- **Role**: Transaction-manager test helpers for constructing a manager, inserting hashes into fetch state, and creating mock session metadata.
- **Key items**: `new_tx_manager()`, `buffer_hash_to_tx_fetcher()`, `new_mock_session()`
- **Interactions**: Works with `TransactionFetcher`, `TransactionsManager`, and `PeerMetadata` to simulate network behavior.

## End-to-end flow (high level)
- Use `PeerConfig`/`Testnet::create*` to spin up a multi-peer test network with optional pools.
- Call `TestnetHandle::connect_peers()` and await events via `NetworkEventStream`.
- For transaction-specific tests, build a `TransactionsManager` via `new_tx_manager()` and simulate inflight hashes with `buffer_hash_to_tx_fetcher()`.

## Key APIs (no snippets)
- `Testnet`, `Peer`, `PeerHandle`, `NetworkEventStream`
- `new_tx_manager()`, `new_mock_session()`
- `unused_tcp_udp()`, `enr_to_peer_id()`
