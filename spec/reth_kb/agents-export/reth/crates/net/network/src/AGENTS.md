# src

## Purpose
Implements the `reth-network` crate's core P2P stack: discovery, peer/session management, network state, request routing, transaction gossip, and the public network handle API.

## Contents (one hop)
### Subdirectories
- [x] `fetch/` - Request coordination and `FetchClient` for headers/bodies downloads.
- [x] `session/` - RLPx/ETH session lifecycle, handshake, and active session state machines.
- [x] `test_utils/` - Testnet harness and helpers for network/tx tests.
- [x] `transactions/` - Transaction gossip, fetcher, policies, and pool integration.

### Files
- `budget.rs` - Polling budgets and macros for draining nested streams with bounded work.
  - **Key items**: `DEFAULT_BUDGET_TRY_DRAIN_STREAM`, `DEFAULT_BUDGET_TRY_DRAIN_SWARM`, `DEFAULT_BUDGET_TRY_DRAIN_NETWORK_HANDLE_CHANNEL`, `poll_nested_stream_with_budget!`, `metered_poll_nested_stream_with_budget!`
- `builder.rs` - Network builder wiring for request handlers and transactions manager.
  - **Key items**: `NetworkBuilder`, `ETH_REQUEST_CHANNEL_CAPACITY`, `request_handler()`, `transactions_with_policy()`, `transactions_with_policies()`
- `cache.rs` - Lightweight LRU cache wrappers used across network subsystems.
  - **Key items**: `LruCache`, `LruMap`
- `config.rs` - Network configuration and builder API for discovery, sessions, and boot nodes.
  - **Key items**: `NetworkConfig`, `NetworkConfigBuilder`, `rng_secret_key()`, `NetworkMode`, `start_network()`
- `discovery.rs` - Discovery orchestration for discv4/discv5/DNS and event fan-out.
  - **Key items**: `Discovery`, `DEFAULT_MAX_CAPACITY_DISCOVERED_PEERS_CACHE`, `add_listener()`, `update_fork_id()`, `ban()`
  - **Interactions**: Drives `reth-discv4`, `reth-discv5`, and DNS discovery streams into `DiscoveryEvent`s.
- `error.rs` - Network error taxonomy and session error classification/backoff rules.
  - **Key items**: `NetworkError`, `ServiceKind`, `SessionError`, `from_io_error()`
- `eth_requests.rs` - ETH request handler for headers/bodies/receipts with size limits.
  - **Key items**: `EthRequestHandler`, `IncomingEthRequest`, `MAX_HEADERS_SERVE`, `MAX_BODIES_SERVE`, `SOFT_RESPONSE_LIMIT`
  - **Interactions**: Uses `BlockReader`/`HeaderProvider` to serve requests and records `EthRequestHandlerMetrics`.
- `flattened_response.rs` - Helper to flatten oneshot receiver errors for async responses.
  - **Key items**: `FlattenedResponse`
- `import.rs` - Block import abstraction for `NewBlock`/`NewBlockHashes` announcements.
  - **Key items**: `BlockImport`, `NewBlockEvent`, `BlockImportEvent`, `BlockValidation`, `ProofOfStakeBlockImport`
- `lib.rs` - Crate entrypoint and public re-exports for networking types and handles.
  - **Key items**: module exports, `NetworkManager`, `NetworkHandle`, `NetworkConfig`, `FetchClient`
- `listener.rs` - TCP connection listener and event type used by the swarm.
  - **Key items**: `ConnectionListener`, `ListenerEvent`
- `manager.rs` - Top-level network event loop that ties together swarm, handlers, and discovery.
  - **Key items**: `NetworkManager`, `with_transactions()`, `set_eth_request_handler()`, `add_rlpx_sub_protocol()`
  - **Interactions**: Drives `Swarm` and routes `NetworkHandle` commands to sessions, tx manager, and request handler.
- `message.rs` - Internal message wrappers and response plumbing for ETH requests.
  - **Key items**: `PeerMessage`, `BlockRequest`, `PeerResponse`, `PeerResponseResult`, `NewBlockMessage`
- `metrics.rs` - Metrics types for network, sessions, transactions, fetcher, and request handling.
  - **Key items**: `NetworkMetrics`, `SessionManagerMetrics`, `TransactionsManagerMetrics`, `TransactionFetcherMetrics`, `EthRequestHandlerMetrics`, `TxTypesCounter`
- `network.rs` - Shareable network handle API and protocol registry hooks.
  - **Key items**: `NetworkHandle`, `NetworkProtocols`, `send_request()`, `announce_block()`, `shutdown()`
- `peers.rs` - Peer manager with reputation/backoff, ban lists, and trusted peer resolution.
  - **Key items**: `PeersManager`, `PeerAction`, `ConnectionInfo`, `InboundConnectionError`
- `protocol.rs` - Interfaces for custom RLPx subprotocols and connection handlers.
  - **Key items**: `ProtocolHandler`, `ConnectionHandler`, `RlpxSubProtocol`, `RlpxSubProtocols`, `OnNotSupported`
- `required_block_filter.rs` - Optional filter task to ban peers missing required block hashes.
  - **Key items**: `RequiredBlockFilter`
- `state.rs` - Network state machine coordinating discovery, peers, and fetcher actions.
  - **Key items**: `NetworkState`, `StateAction`, `BlockNumReader`
- `swarm.rs` - Connectivity layer combining listener, session manager, and network state.
  - **Key items**: `Swarm`, `SwarmEvent`, `NetworkConnectionState`
- `trusted_peers_resolver.rs` - Periodic DNS resolution for trusted peers.
  - **Key items**: `TrustedPeersResolver`

## Key APIs (no snippets)
- `NetworkManager`, `NetworkHandle`, `NetworkConfig`
- `PeersManager`, `SessionManager`
- `Discovery`, `FetchClient`, `EthRequestHandler`

## Relationships
- **Depends on**: `reth-eth-wire` (protocols/handshakes), `reth-discv4`/`reth-discv5`/`reth-dns-discovery` (discovery), `reth-network-p2p` (download traits), `reth-network-types` (peer/session config), `reth-transaction-pool` (tx propagation).
- **Used by**: Node builder and CLI commands to configure and run the network stack; tests in `tests/it` and benches in `benches`.
