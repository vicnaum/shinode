# it

## Purpose
Integration tests for network behavior: peer connections, request/response flows, session negotiation, startup/discovery edge cases, and transaction gossip/fetching.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness that wires all integration test modules.
- **Key items**: module list for `connect`, `requests`, `session`, `startup`, `transaction_hash_fetching`, `txgossip`, `multiplex`, `big_pooled_txs_req`

### `big_pooled_txs_req.rs`
- **Role**: Validates large `GetPooledTransactions` request handling by fetching ~2000 txs from a peer and verifying returned hashes.
- **Key items**: `test_large_tx_req`, `GetPooledTransactions`, `PooledTransactions`, `Testnet`
- **Interactions**: Uses `TransactionsManager` + request handlers on a two-peer testnet.

### `connect.rs`
- **Role**: Connection and peer-management integration tests (establishment, already-connected handling, peer lookups, and discovery/listener edge cases).
- **Key items**: `test_establish_connections`, `test_already_connected`, `test_get_peer`, `NetworkConfigBuilder`, `Discv4Config`
- **Interactions**: Uses `Testnet`, `NetworkManager`, `NetworkEventStream`, and `PeersConfig` to validate connection lifecycle.

### `multiplex.rs`
- **Role**: RLPx subprotocol multiplexing tests using a custom ping/pong protocol handler.
- **Key items**: `PingPongProtoMessage`, `PingPongProtoHandler`, `ProtocolHandler`, `ConnectionHandler`
- **Interactions**: Exercises `ProtocolConnection` and capability negotiation alongside the core network.

### `requests.rs`
- **Role**: Verifies header/body request flows via `FetchClient` and network request handlers.
- **Key items**: `test_get_body`, `test_get_body_range`, `test_get_header`, `test_get_header_range`
- **Interactions**: Uses `HeadersClient`, `BodiesClient`, `Testnet`, and `MockEthProvider`.

### `session.rs`
- **Role**: Session negotiation tests for eth protocol version selection and capability mismatches.
- **Key items**: `test_session_established_with_highest_version`, `test_capability_version_mismatch`, `test_eth69_peers_can_connect`
- **Interactions**: Uses `PeerConfig::with_protocols` and `NetworkEventStream`.

### `startup.rs`
- **Role**: Startup/config edge-case tests (addr-in-use errors, discovery conflicts, NAT address handling).
- **Key items**: `test_listener_addr_in_use`, `test_discovery_addr_in_use`, `test_discv5_and_discv4_same_socket_fails`
- **Interactions**: Validates `NetworkConfigBuilder`, `Discovery`, and `NetworkManager` error paths.

### `transaction_hash_fetching.rs`
- **Role**: End-to-end tx hash fetching test across peers (ignored by default).
- **Key items**: `transaction_hash_fetching` (#[ignore]), `TransactionsManagerConfig`, `Testnet`
- **Interactions**: Uses pools and pending transaction listeners to confirm fetch completion.

### `txgossip.rs`
- **Role**: Transaction gossip and policy tests (propagation modes and ingress policies).
- **Key items**: `test_tx_gossip`, `test_tx_propagation_policy_trusted_only`, `test_tx_ingress_policy_trusted_only`
- **Interactions**: Exercises `TransactionsManagerConfig`, `TransactionPropagationKind`, `TransactionIngressPolicy`, and peer trust changes.

## End-to-end flow (high level)
- Build a `Testnet` with mock providers and pools, then connect peers.
- Drive requests or broadcasts through network handles and wait for events.
- Assert negotiation behavior, error handling, or policy effects based on the scenario.

## Key APIs (no snippets)
- `Testnet`, `NetworkEventStream`
- `FetchClient`, `HeadersClient`, `BodiesClient`
- `TransactionsManagerConfig`, `TransactionPropagationKind`, `TransactionIngressPolicy`
