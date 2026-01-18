# Q005 - RPC Entry Points & Server Wiring

Status: verified

## Question
Where are the RPC entry points (module builder, server wiring, transports), and how do they connect?

## Context Packs
- `spec/reth_repo_context/context-rpc.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-rpc.xml

Question:
Where are the RPC entry points (module builder, server wiring, transports), and how do they connect?
I want a map of RPC server construction, module registration, and runtime wiring.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Identify entry points (builder types, server start, CLI wiring).
3. Explain how RPC modules are composed and how requests are routed to providers.
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
Based on the `context-rpc.xml` file, here is the high-level architecture map of Reth’s RPC layer, covering entry points, module construction, and runtime wiring.

### 1. Summary
*   **Trait-First Implementation**: RPC namespaces (like `eth`, `debug`, `trace`) are defined as traits in `reth-rpc-api` and implemented via helper traits in `reth-rpc-eth-api`, allowing for chain-agnostic logic.
*   **Modular Assembly**: The `RpcModuleBuilder` acts as a factory, taking core node components (Provider, Pool, Network) and producing a registry of API handlers.
*   **Transport Agnostic logic**: Handlers are written to be transport-blind; the `TransportRpcModules` container later maps these handlers to specific HTTP, WS, or IPC listeners.
*   **Asynchronous Cache Layer**: To avoid blocking the RPC server with disk I/O, the `EthStateCache` provides a frontend to an async service that manages LRU caches for blocks, headers, and receipts.
*   **The "Eth" Super-Namespace**: The `eth_` namespace is so large it is split into specialized sub-components (Blocks, Transactions, Fees, State, Filter, PubSub) coordinated by the `EthApi` struct.
*   **Middleware-Heavy Pipeline**: Reth wraps `jsonrpsee` servers in layers for JWT authentication, Prometheus metrics, and resource-based rate limiting.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/node/core/src/args/rpc_server.rs` | `RpcServerArgs` | Maps CLI flags (e.g., `--http.api`, `--rpc.gascap`) into internal configuration objects. |
| `crates/rpc/rpc-builder/src/lib.rs` | `RpcModuleBuilder` | The primary orchestrator used to initialize all RPC namespaces with node dependencies. |
| `crates/rpc/rpc-builder/src/lib.rs` | `RpcServerConfig` | Handles the physical binding to sockets (TCP/IPC) and the injection of Tower middleware. |
| `crates/rpc/rpc/src/eth/core.rs` | `EthApi` | The core implementation of the Ethereum JSON-RPC spec, delegating logic to helper traits. |
| `crates/rpc/rpc-builder/src/auth.rs` | `AuthServerConfig` | Specifically handles the Engine API server (port 8551) used by Consensus Clients. |

---

### 3. Lifecycle Flow

#### Step 1: CLI Configuration
CLI arguments are parsed into `RpcServerArgs`. The `transport_rpc_module_config()` function determines which APIs (eth, net, web3, etc.) are exposed on which transport (`crates/node/core/src/args/rpc_server.rs`).

#### Step 2: Builder Initialization
The node creates an `RpcModuleBuilder`, providing it with the `Provider` (database access), `Pool` (mempool), and `Network` (p2p info) handles (`crates/rpc/rpc-builder/src/lib.rs`).

#### Step 3: Module Registration (`RpcRegistryInner`)
The builder populates an `RpcRegistryInner`. When `register_eth()` or `register_debug()` is called, the registry instantiates the implementation (e.g., `EthApi`) and wraps it in a `jsonrpsee::Methods` object (`crates/rpc/rpc-builder/src/lib.rs`).

#### Step 4: Transport Mapping
`create_transport_rpc_modules` takes the registry and splits the methods into three potential `RpcModule` instances: `http`, `ws`, and `ipc`, based on the user's CLI selection (`crates/rpc/rpc-builder/src/lib.rs`).

#### Step 5: Server Startup
`RpcServerConfig::start` is invoked. It:
1.  Binds to the `SocketAddr`.
2.  Adds `AuthLayer` (if JWT is required).
3.  Adds `RpcRequestMetrics` for Prometheus tracking.
4.  Spawns the `jsonrpsee` server task (`crates/rpc/rpc-builder/src/lib.rs`).

#### Step 6: Request Routing (Runtime)
When an `eth_call` arrives:
1.  **Transport**: `jsonrpsee` identifies the method.
2.  **Handler**: `EthApi` receives the request.
3.  **State Lookup**: `EthApi` calls `state_at_block_id`, which queries the `EthStateCache` (`crates/rpc/rpc-eth-api/src/helpers/state.rs`).
4.  **Execution**: Logic is dispatched to the `REVM` (logic outside this pack) via the `SpawnBlocking` trait to ensure EVM execution doesn't stall the async runtime (`crates/rpc/rpc-eth-api/src/helpers/blocking_task.rs`).

---

### 4. Key Abstractions / Types

*   **`EthHandlers<EthApi>`** (`crates/rpc/rpc-builder/src/eth.rs`): A container that groups the standard API, the polling-based `EthFilter`, and the subscription-based `EthPubSub`.
*   **`EthStateCache`** (`crates/rpc/rpc-eth-types/src/cache/mod.rs`): A multi-consumer LRU cache that manages `RecoveredBlock` and `Receipt` objects to reduce database load.
*   **`RpcNodeCore`** (`crates/rpc/rpc-eth-api/src/node.rs`): A trait that abstracts the requirements for a node to support RPC (must provide Primitives, Provider, Pool, and Network).
*   **`PendingBlockKind`** (`crates/rpc/rpc-eth-types/src/builder/config.rs`): An enum (Full/Empty/None) that controls how Reth simulates the "pending" block when users query for it.
*   **`SignersForRpc`** (`crates/rpc/rpc-eth-api/src/helpers/spec.rs`): A thread-safe handle to local developer keys (e.g., `DevSigner`) used for `eth_sign` and `eth_sendTransaction`.

---

### 5. "Where to Look Next"
1.  **`crates/rpc/rpc/src/eth/helpers/call.rs`**: To see how Reth actually executes an `eth_call` or `eth_estimateGas` using overrides.
2.  **`crates/rpc/rpc-builder/src/auth.rs`**: To understand the specific security and routing requirements of the Engine API.
3.  **`crates/rpc/rpc-eth-api/src/helpers/pending_block.rs`**: To see the complex logic for building a local block from the mempool to satisfy "pending" tag queries.
4.  **`crates/rpc/rpc/src/eth/pubsub.rs`**: To see how Reth uses `tokio::sync::broadcast` to push new headers and logs to websocket clients.

---

### 6. Suggested Follow-up Questions
*   "How does the `EthStateCacheService` handle cache invalidation during a chain reorg?"
*   "What is the difference between `spawn_blocking_io` and `spawn_tracing` in the `SpawnBlocking` trait?"
*   "How is the `eth_simulateV1` method implemented to support multi-block simulations?"
*   "How does Reth enforce the `rpc-eth-proof-window` when a user calls `eth_getProof` on a very old block?"

## Verification
- `RpcModuleBuilder` is the main entrypoint; it builds `TransportRpcModules` and the registry wires modules to transports.```6:12:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
//! The [`RpcModuleBuilder`] is the main entrypoint for configuring all reth modules. It takes
//! instances of components required to start the servers, such as provider impls, network and
//! transaction pool. [`RpcModuleBuilder::build`] returns a [`TransportRpcModules`] which contains
//! the transport specific config (what APIs are available via this transport).
//! ...
//! The [`RpcServerConfig`] is used to assemble and start the http server, ws server, ipc servers,
//! it requires the [`TransportRpcModules`] so it can start the servers with the configured modules.
```
```322:403:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
pub fn build<EthApi>(
    self,
    module_config: TransportRpcModuleConfig,
    eth: EthApi,
    engine_events: EventSender<ConsensusEngineEvent<N>>,
) -> TransportRpcModules<()> {
    let mut modules = TransportRpcModules::default();
    if !module_config.is_empty() {
        let TransportRpcModuleConfig { http, ws, ipc, config } = module_config.clone();
        let mut registry = self.into_registry(config.unwrap_or_default(), eth, engine_events);
        modules.config = module_config;
        modules.http = registry.maybe_module(http.as_ref());
        modules.ws = registry.maybe_module(ws.as_ref());
        modules.ipc = registry.maybe_module(ipc.as_ref());
    }
    modules
}
```
- `TransportRpcModules` explicitly holds per‑transport `RpcModule`s (http/ws/ipc).```1629:1639:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
/// Holds installed modules per transport type.
pub struct TransportRpcModules<Context = ()> {
    /// The original config
    config: TransportRpcModuleConfig,
    /// rpcs module for http
    http: Option<RpcModule<Context>>,
    /// rpcs module for ws
    ws: Option<RpcModule<Context>>,
    /// rpcs module for ipc
    ipc: Option<RpcModule<Context>>,
}
```
- `RpcServerConfig::start` wires CORS/JWT/compression and `RpcRequestMetrics` into jsonrpsee servers.```1278:1389:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
fn maybe_jwt_layer(jwt_secret: Option<JwtSecret>) -> Option<AuthLayer<JwtAuthValidator>> { ... }
...
pub async fn start(self, modules: &TransportRpcModules) -> Result<RpcServerHandle, RpcError> {
    ...
    let server = ServerBuilder::new()
        .set_http_middleware(
            tower::ServiceBuilder::new()
                .option_layer(Self::maybe_cors_layer(cors)?)
                .option_layer(Self::maybe_jwt_layer(self.jwt_secret))
                .option_layer(Self::maybe_compression_layer(self.http_disable_compression)),
        )
        .set_rpc_middleware(
            RpcServiceBuilder::default()
                .layer(modules.http.as_ref().map(RpcRequestMetrics::http).unwrap_or_default())
                .layer(self.rpc_middleware.clone()),
        )
        .set_config(config.build())
        .build(http_socket_addr)
        .await?;
    ...
}
```
- `RpcServerArgs` maps CLI flags and transport module selection.```412:458:/Users/vicnaum/github/stateless-history-node/reth/crates/node/core/src/args/rpc_server.rs
#[derive(Debug, Clone, Args, PartialEq, Eq)]
pub struct RpcServerArgs {
    /// Enable the HTTP-RPC server
    pub http: bool,
    /// Rpc Modules to be configured for the HTTP server
    pub http_api: Option<RpcModuleSelection>,
    /// Enable the WS-RPC server
    pub ws: bool,
    /// Rpc Modules to be configured for the WS server
    pub ws_api: Option<RpcModuleSelection>,
    ...
}
```
- `EthApi` is implemented in `crates/rpc/rpc/src/eth/core.rs` and uses `EthStateCache` + `PendingBlockKind`.```54:160:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/core.rs
/// `Eth` API implementation.
pub struct EthApi<N: RpcNodeCore, Rpc: RpcConvert> { ... }
...
pub fn new(
    components: N,
    eth_cache: EthStateCache<N::Primitives>,
    ...
    pending_block_kind: PendingBlockKind,
    ...
) -> Self { ... }
```
- `EthHandlers` groups `EthFilter` and `EthPubSub` with the main `EthApi`.```6:14:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/eth.rs
pub struct EthHandlers<EthApi: EthApiTypes> {
    /// Main `eth_` request handler
    pub api: EthApi,
    /// Polling based filter handler available on all transports
    pub filter: EthFilter<EthApi>,
    /// Handler for subscriptions only available for transports that support it (ws, ipc)
    pub pubsub: EthPubSub<EthApi>,
}
```
- `EthStateCache` is a frontend for an async cache service (spawned task, LRU caches).```58:140:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-types/src/cache/mod.rs
/// Provides async access to cached eth data
/// This is the frontend for the async caching service which manages cached data on a different task.
pub struct EthStateCache<N: NodePrimitives> {
    to_service: UnboundedSender<CacheAction<N::Block, N::Receipt>>,
}
...
pub fn spawn_with<Provider, Tasks>(provider: Provider, config: EthStateCacheConfig, executor: Tasks) -> Self {
    ...
    executor.spawn_critical("eth state cache", Box::pin(service));
    this
}
```
- `RpcNodeCore`, `PendingBlockKind`, and `SignersForRpc` are defined in the claimed files.```24:63:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-api/src/node.rs
pub trait RpcNodeCore: Clone + Send + Sync + Unpin + 'static {
    type Primitives: NodePrimitives;
    type Provider: BlockReaderIdExt<...> + ChainSpecProvider<...> + StateProviderFactory + ...;
    type Pool: TransactionPool<...>;
    type Evm: ConfigureEvm<...>;
    type Network: NetworkInfo + Clone;
    ...
}
```
```21:31:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-types/src/builder/config.rs
pub enum PendingBlockKind {
    Empty,
    None,
    #[default]
    Full,
}
```
```75:79:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-api/src/helpers/spec.rs
pub type SignersForRpc<Provider, Rpc> = parking_lot::RwLock<
    Vec<Box<dyn EthSigner<<Provider as TransactionsProvider>::Transaction, RpcTxReq<Rpc>>>>,
>;
```
- Rate limiter exists as a middleware layer, but it is not automatically applied unless configured as RPC middleware.```1:37:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/rate_limiter.rs
/// Rate limiter for the RPC server.
pub struct RpcRequestRateLimiter { ... }
impl<S> Layer<S> for RpcRequestRateLimiter { ... }
```

## Corrections / Caveats
- “Prometheus metrics” is not explicitly stated; the code uses `RpcRequestMetrics` (metrics layer), but the backend exporter is not implied here.
- Rate limiting is available as a middleware layer but is not applied by default in `RpcServerConfig::start`.
- The Engine API auth server uses `AuthServerConfig`; the exact default port comes from constants, not hardcoded here.

## Actionable Pointers
- RPC server build/start: `reth/crates/rpc/rpc-builder/src/lib.rs` (build, TransportRpcModules, RpcServerConfig::start).
- Engine API auth: `reth/crates/rpc/rpc-builder/src/auth.rs`.
- RPC CLI wiring: `reth/crates/node/core/src/args/rpc_server.rs` and `reth/crates/rpc/rpc-builder/src/config.rs`.
- Eth API core: `reth/crates/rpc/rpc/src/eth/core.rs`.
- Eth cache: `reth/crates/rpc/rpc-eth-types/src/cache/mod.rs`.
