# Q002 - Networking Entry Points & Lifecycle

Status: verified

## Question
Where are the core networking entry points and how do they connect? Provide a map of the P2P stack from configuration to active peer sessions and request handling.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question:
Where are the core networking entry points and how do they connect?
I want a map of the P2P stack from configuration to active peer sessions and request handling.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Identify entry points (builders, managers, handles, public APIs).
3. Provide the lifecycle: config -> start -> discovery -> sessions -> request routing.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Lifecycle flow (step-by-step)
4. Key abstractions/types (with file paths)
5. “Where to look next” (top files to read)
6. Suggested follow-up questions
```

## Gemini Answer
Based on the provided networking context, here is the architecture map of the Reth P2P stack, ranging from configuration to active request handling.

### 1. Summary
*   **Modular Entry Points**: Users configure the stack via `NetworkConfigBuilder` and interact with it post-launch via `NetworkHandle` or `FetchClient`.
*   **Central Hub**: The `NetworkManager` acts as the primary "hub" future, orchestrating discovery, connection swarm, and sub-protocol routing.
*   **Multi-Protocol Discovery**: Discovery is handled concurrently via three protocols: Discv4 (UDP), Discv5 (UDP/ENR), and DNS (EIP-1459).
*   **Connection Layering**: Reth wraps raw TCP in ECIES encryption, then into an RLPx `P2PStream` (handling pings/compression), and finally into an `EthStream` or `EthSnapStream` for sub-protocol logic.
*   **The Swarm**: The `Swarm` struct is the internal engine that manages the `SessionManager` (connection state) and `NetworkState` (peer/data state).
*   **Task Separation**: Sub-protocols like transactions and ETH requests are offloaded to specialized spawned tasks (`TransactionsManager` and `EthRequestHandler`).

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/config.rs` | `NetworkConfigBuilder` | The primary UI for configuring the stack (ports, secret keys, bootnodes, and discovery settings). |
| `crates/net/network/src/manager.rs` | `NetworkManager` | The core "worker" task. It is an endless future that drives the entire P2P stack forward. |
| `crates/net/network/src/network.rs` | `NetworkHandle` | A cloneable, thread-safe handle used by other parts of the node (like the RPC or Pipeline) to send commands or query network status. |
| `crates/net/network/src/fetch/client.rs` | `FetchClient` | The high-level interface used by the sync pipeline to request headers and block bodies. |
| `crates/net/network/src/builder.rs` | `NetworkBuilder` | A helper to assemble the `NetworkManager` with its sub-components like the `TransactionsManager`. |

---

### 3. Lifecycle Flow

#### Step 1: Configuration
The user uses `NetworkConfigBuilder::new(secret_key)` to define parameters. This builder eventually produces a `NetworkConfig` (`crates/net/network/src/config.rs`).

#### Step 2: Starting the Manager
`NetworkManager::new(config)` is called. This initializes the `Swarm`, which includes:
*   **`ConnectionListener`**: Binds to a TCP port for incoming peers (`crates/net/network/src/listener.rs`).
*   **`SessionManager`**: Prepares to handle handshakes (`crates/net/network/src/session/mod.rs`).
*   **`Discovery`**: Starts the UDP discovery services (`crates/net/network/src/discovery.rs`).

#### Step 3: Discovery & Peering
The `Discovery` task finds peer addresses. It sends `DiscoveryEvent::NewNode` to the `NetworkState`. If there is capacity, the `PeersManager` issues a `PeerAction::Connect` (`crates/net/network/src/peers.rs`).

#### Step 4: The Handshake (Unauthed to Authed)
1.  **TCP Connection**: Established via `TcpStream`.
2.  **ECIES Handshake**: Authenticates and encrypts the stream (`crates/net/network/src/session/mod.rs`).
3.  **P2P Hello**: `UnauthedP2PStream` exchanges `Hello` messages and negotiates capabilities (`crates/net/eth-wire/src/p2pstream.rs`).
4.  **ETH Status**: `UnauthedEthStream` exchanges `Status` messages to ensure peers are on the same chain/fork (`crates/net/eth-wire/src/ethstream.rs`).

#### Step 5: Active Session & Multiplexing
Once authed, a session becomes an `ActiveSession`. It uses `RlpxProtocolMultiplexer` to allow multiple protocols (like `eth` and `snap`) to run over the same encrypted connection (`crates/net/eth-wire/src/multiplex.rs`).

#### Step 6: Request Routing
When a message arrives:
*   **Transactions** are sent to the `TransactionsManager` via the `to_transactions_manager` channel.
*   **ETH Data Requests** (GetHeaders, GetBodies) are sent to the `EthRequestHandler` (`crates/net/network/src/eth_requests.rs`).
*   **Inbound Data** for our own requests is routed back to the `FetchClient`'s response channel.

---

### 4. Key Abstractions/Types

*   **`Swarm`** (`crates/net/network/src/swarm.rs`): Orchestrates the interaction between the TCP listener, the sessions, and the internal state.
*   **`EthStream`** (`crates/net/eth-wire/src/ethstream.rs`): A wrapper that transforms RLPx bytes into `EthMessage` types.
*   **`PeerMetadata`** (`crates/net/network/src/test_utils/transactions.rs`): Tracks what data a peer has seen to prevent redundant broadcasts.
*   **`UnifiedStatus`** (`crates/net/eth-wire-types/src/status.rs`): A version-agnostic structure that encapsulates all fields needed for `eth/66` through `eth/70` handshakes.
*   **`BlockRangeInfo`** (`crates/net/network/src/session/types.rs`): Tracks the block range a peer can serve (critical for `eth/69` protocol).

---

### 5. Where to Look Next
1.  **`crates/net/network/src/swarm.rs`**: This is the best place to see the actual logic for how Reth decides to accept a connection or dial a peer.
2.  **`crates/net/network/src/session/active.rs`**: Read this to understand the main loop of a single peer connection, including how timeouts and protocol breaches are handled.
3.  **`crates/net/eth-wire/src/multiplex.rs`**: Read this to understand how Reth runs both the `eth` and `snap` protocols simultaneously.
4.  **`crates/net/network/src/transactions.rs`** (referenced but logic partially outside): To see how transaction gossip and propagation are handled.

---

### 6. Suggested Follow-up Questions
*   "How does the `SessionManager` enforce connection limits for inbound versus outbound peers?"
*   "How are the new `eth/70` partial receipts requested and validated in `eth_requests.rs`?"
*   "What is the role of the `RequiredBlockFilter` in filtering peers for shadowforks?"
*   "How does the `Pinger` state machine in `pinger.rs` trigger a peer disconnect on timeout?"

## Verification
- `NetworkConfigBuilder` exists and is the primary config UI; `NetworkConfig::start_network` spawns the network and eth-request handler tasks via `NetworkManager::builder`.```170:182:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/config.rs
    /// Starts the networking stack given a [`NetworkConfig`] and returns a handle to the network.
    pub async fn start_network(self) -> Result<NetworkHandle<N>, NetworkError> {
        let client = self.client.clone();
        let (handle, network, _txpool, eth) = NetworkManager::builder::<C>(self)
            .await?
            .request_handler::<C>(client)
            .split_with_handle();

        tokio::task::spawn(network);
        tokio::task::spawn(eth);
        Ok(handle)
    }
```
```185:222:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/config.rs
/// Builder for [`NetworkConfig`](struct.NetworkConfig.html).
#[derive(Debug)]
pub struct NetworkConfigBuilder<N: NetworkPrimitives = EthNetworkPrimitives> {
    /// The node's secret key, from which the node's identity is derived.
    secret_key: SecretKey,
    /// How to configure discovery over DNS.
    dns_discovery_config: Option<DnsDiscoveryConfig>,
    /// How to set up discovery version 4.
    discovery_v4_builder: Option<Discv4ConfigBuilder>,
    /// How to set up discovery version 5.
    discovery_v5_builder: Option<reth_discv5::ConfigBuilder>,
    /// All boot nodes to start network discovery with.
    boot_nodes: HashSet<TrustedPeer>,
```
- `NetworkManager` is an endless future and wires up discovery, sessions, state, and swarm.```224:318:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
/// The [`NetworkManager`] is an endless future that needs to be polled in order to advance the
/// state of the entire network.
pub async fn new<C: BlockNumReader + 'static>(
    config: NetworkConfig<C, N>,
) -> Result<Self, NetworkError> {
    ...
    let discovery = Discovery::new(...).await?;
    let sessions = SessionManager::new(...);
    let state = NetworkState::new(...);
    let swarm = Swarm::new(incoming, sessions, state);
    ...
}
```
- Discovery uses discv4/discv5/DNS and emits `DiscoveryEvent`s.```35:67:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/discovery.rs
/// An abstraction over the configured discovery protocol.
pub struct Discovery {
    /// Handler to interact with the Discovery v4 service
    discv4: Option<Discv4>,
    /// Handler to interact with the Discovery v5 service
    discv5: Option<Discv5>,
    /// Updates from the DNS discovery service.
    dns_discovery_updates: Option<ReceiverStream<DnsNodeRecordUpdate>>,
    ...
}
```
- Connection layering is explicit: `EthStream<P2PStream<ECIESStream<TcpStream>>>`, with optional satellite protocols.```18:37:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/conn.rs
/// The type of the underlying peer network connection.
pub type EthPeerConnection<N> = EthStream<P2PStream<ECIESStream<TcpStream>>, N>;
...
pub enum EthRlpxConnection<N: NetworkPrimitives = EthNetworkPrimitives> {
    /// A connection that only supports the ETH protocol.
    EthOnly(Box<EthPeerConnection<N>>),
    /// A connection that supports the ETH protocol and __at least one other__ `RLPx` protocol.
    Satellite(Box<EthSatelliteConnection<N>>),
}
```
- P2P and ETH handshakes are performed by `UnauthedP2PStream` and `UnauthedEthStream`.```63:103:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/p2pstream.rs
/// An un-authenticated [`P2PStream`]. This is consumed and returns a [`P2PStream`] after the
/// `Hello` handshake is completed.
pub struct UnauthedP2PStream<S> {
...
    pub async fn handshake(
        mut self,
        hello: HelloMessageWithProtocols,
    ) -> Result<(P2PStream<S>, HelloMessage), P2PStreamError> {
```
```35:73:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/ethstream.rs
/// An un-authenticated [`EthStream`]. This is consumed and returns a [`EthStream`] after the
/// `Status` handshake is completed.
pub struct UnauthedEthStream<S> {
...
    pub async fn handshake<N: NetworkPrimitives>(
        self,
        status: UnifiedStatus,
        fork_filter: ForkFilter,
    ) -> Result<(EthStream<S, N>, UnifiedStatus), EthStreamError> {
```
- The RLPx protocol multiplexer exists and is named `RlpxProtocolMultiplexer`.```35:52:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/multiplex.rs
/// A Stream and Sink type that wraps a raw rlpx stream [`P2PStream`] and handles message ID
/// multiplexing.
pub struct RlpxProtocolMultiplexer<St> {
    inner: MultiplexInner<St>,
}
```
- `Swarm` is the core state machine for listener/session/state wiring.```25:57:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/swarm.rs
/// A swarm emits [`SwarmEvent`]s when polled.
#[derive(Debug)]
#[must_use = "Swarm does nothing unless polled"]
pub(crate) struct Swarm<N: NetworkPrimitives = EthNetworkPrimitives> {
    /// Listens for new incoming connections.
    incoming: ConnectionListener,
    /// All sessions.
    sessions: SessionManager<N>,
    /// Tracks the entire state of the network and handles events received from the sessions.
    state: NetworkState<N>,
}
```
- `FetchClient` is a front-end API that sends download requests over a channel.```27:43:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/fetch/client.rs
/// Front-end API for fetching data from the network.
#[derive(Debug, Clone)]
pub struct FetchClient<N: NetworkPrimitives = EthNetworkPrimitives> {
    /// Sender half of the request channel.
    pub(crate) request_tx: UnboundedSender<DownloadRequest<N>>,
```
- Discovery → state → peer actions: discovery events are queued as `StateAction::DiscoveredNode`, and peer actions can produce `PeerAction::Connect`.```325:347:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/state.rs
fn on_discovery_event(&mut self, event: DiscoveryEvent) {
    match event {
        DiscoveryEvent::NewNode(DiscoveredEvent::EventQueued { peer_id, addr, fork_id }) => {
            self.queued_messages.push_back(StateAction::DiscoveredNode { ... });
        }
        ...
    }
}
fn on_peer_action(&mut self, action: PeerAction) {
    match action {
        PeerAction::Connect { peer_id, remote_addr } => {
            self.queued_messages.push_back(StateAction::Connect { peer_id, remote_addr });
        }
        ...
    }
}
```
- `PeerMetadata` is in `crates/net/network/src/transactions/mod.rs`, not `test_utils`.```1963:1975:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
/// Tracks a single peer in the context of [`TransactionsManager`].
pub struct PeerMetadata<N: NetworkPrimitives = EthNetworkPrimitives> {
```
- `UnifiedStatus` and `BlockRangeInfo` live in the claimed files.```10:31:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire-types/src/status.rs
/// `UnifiedStatus` is an internal superset of all ETH status fields for all `eth/` versions.
pub struct UnifiedStatus {
```
```14:42:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/types.rs
/// Information about the range of full blocks available from a peer.
pub struct BlockRangeInfo {
```

## Corrections / Caveats
- “EIP-1459” is not referenced in code; DNS discovery is implemented via `reth_dns_discovery`, but the EIP mention should be treated as unverified within this pack.
- `PeerMetadata` is not in `test_utils`; it’s in `crates/net/network/src/transactions/mod.rs` (corrected above).
- The “transactions task is spawned” depends on the caller. `NetworkConfig::start_network` spawns the network and eth-request handler tasks; it doesn’t spawn a `TransactionsManager` unless the caller wires and spawns it (e.g., node builder).
- “FetchClient used by the sync pipeline” is plausible but not proven by this pack alone; the pipeline lives in `crates/stages/**`.

## Actionable Pointers
- Start/configure the network: `reth/crates/net/network/src/config.rs` (`NetworkConfigBuilder`, `start_network`).
- Core runtime loop: `reth/crates/net/network/src/manager.rs` (`NetworkManager::new`, `Swarm::new`).
- P2P session lifecycle: `reth/crates/net/network/src/session/mod.rs`, `.../session/active.rs`.
- Discovery plumbing: `reth/crates/net/network/src/discovery.rs` and `.../state.rs`.
- Request routing: `reth/crates/net/network/src/fetch/client.rs` + `.../fetch/mod.rs` + `.../state.rs`.
- Transactions task wiring: `reth/crates/net/network/src/builder.rs` + `.../transactions/mod.rs`.
