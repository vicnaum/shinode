# Q021 - RPC Server Wiring

Status: verified

## Question
How is the RPC server built and modules wired together?

## Context Packs
- `spec/reth_repo_context/context-rpc.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-rpc.xml

Question (Q021):
How is the RPC server built and modules wired together?
I want the construction flow, module registration (eth, net, admin, debug, trace, engine), and how transport configs map to modules.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe how RpcModuleBuilder/TransportRpcModules are created and configured.
3. Explain where auth RPC is configured and how rate limiting/middleware attach.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Algorithm flow (step-by-step)
4. Key abstractions/types (with file paths)
5. "Where to look next"
6. Suggested follow-up questions
```

## Gemini Answer
Based on the provided `context-rpc.xml` pack, here is the architectural map of how the Reth RPC server is built, configured, and wired.

### 1. Summary
*   **Builder Pattern Construction**: The `RpcModuleBuilder` acts as the central factory, taking core node components (Provider, Pool, Network, EVM) and producing a registry of instantiated RPC handlers.
*   **Transport-Specific Isolation**: Modules are not globally enabled; they are explicitly mapped to transports (HTTP, WS, IPC) via `TransportRpcModuleConfig`. This allows exposing different APIs on different interfaces.
*   **Registry as Hub**: The `RpcRegistryInner` is the internal state container that holds the actual instances of API handlers (e.g., `EthApi`, `DebugApi`) and lazily generates `jsonrpsee` method tables.
*   **Separation of Auth**: The Engine API (Auth RPC) is constructed separately via `AuthServerConfig` and `AuthRpcModule` to enforce JWT authentication and isolate consensus-critical methods.
*   **Layered Configuration**: Configuration flows from CLI arguments (`RpcServerArgs`) -> Builder Configs (`RpcModuleConfig`) -> Server Configs (`RpcServerConfig`).
*   **Middleware Stack**: The server supports a middleware stack for functionality like CORS, JWT validation, and Rate Limiting (`RpcRequestRateLimiter`), applied via Tower layers.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/rpc/rpc-builder/src/lib.rs` | `RpcModuleBuilder` | The primary entry point for assembling the RPC layer. It requires node components to initialize. |
| `crates/node/core/src/args/rpc_server.rs` | `RpcServerArgs` | Parses CLI flags (`--http`, `--http.api`) and converts them into server configurations. |
| `crates/rpc/rpc-builder/src/lib.rs` | `RpcRegistryInner` | The internal container that instantiates API implementations (`EthApi`, `NetApi`, etc.) and maps them to namespaces. |
| `crates/rpc/rpc-builder/src/lib.rs` | `RpcServerConfig` | Configures and starts the actual `jsonrpsee` servers (binds ports, applies middleware). |
| `crates/rpc/rpc-builder/src/auth.rs` | `AuthServerConfig` | Specifically handles the configuration and startup of the Authenticated Engine API (port 8551). |

---

### 3. Construction & Wiring Flow

#### Step 1: Configuration Parsing
The process begins with `RpcServerArgs`. It defines methods like `transport_rpc_module_config()` which determines which `RethRpcModule`s (e.g., `eth`, `admin`) are active for which transport.
*   **File**: `crates/node/core/src/args/rpc_server.rs`

#### Step 2: Builder Initialization
The node initializes `RpcModuleBuilder` with the core components.
*   **Function**: `RpcModuleBuilder::new`
*   **File**: `crates/rpc/rpc-builder/src/lib.rs`

#### Step 3: Registry Creation (`into_registry`)
The builder creates an `RpcRegistryInner`. This struct instantiates the heavy-lifting API handlers.
*   **Logic**: It creates the `EthApi` (using `EthApiBuilder`), `EthFilter`, and `EthPubSub`. It stores these in `EthHandlers`.
*   **File**: `crates/rpc/rpc-builder/src/lib.rs`

#### Step 4: Module Registration & Method Generation
When `create_transport_rpc_modules` is called, the registry iterates over the selected `RethRpcModule` enums.
*   **Function**: `reth_methods` in `crates/rpc/rpc-builder/src/lib.rs`.
*   **Wiring**:
    *   Matches `RethRpcModule::Eth` -> Calls `register_eth` -> Merges `EthApi`, `EthFilter`, `EthPubSub`, and `EthBundle` into one `jsonrpsee::Methods` map.
    *   Matches `RethRpcModule::Debug` -> Instantiates `DebugApi`.
    *   Matches `RethRpcModule::Admin` -> Instantiates `AdminApi`.

#### Step 5: Transport Grouping
The builder outputs a `TransportRpcModules` struct. This contains three optional `RpcModule<()>` fields: one for HTTP, one for WS, and one for IPC. Each contains the specific set of methods allowed on that transport.
*   **File**: `crates/rpc/rpc-builder/src/lib.rs`

#### Step 6: Server Startup
Finally, `RpcServerConfig::start` is called with the `TransportRpcModules`.
*   **Logic**:
    1.  Builds the `jsonrpsee` server.
    2.  Applies Middleware (CORS, JWT, RateLimits).
    3.  Starts the server handle with the configured modules.
*   **File**: `crates/rpc/rpc-builder/src/lib.rs`

---

### 4. Auth (Engine) RPC Configuration

The Auth RPC is handled distinctly to ensure security for Consensus Layer communication.

1.  **Configuration**: Handled by `AuthServerConfig`.
2.  **Module**: `AuthRpcModule` is created via `RpcRegistryInner::create_auth_module`.
3.  **Wiring**:
    *   It registers the **Engine API** implementation provided by the node.
    *   It merges a **subset** of the `EthApi` (specifically `syncing`, `chainId`, `blockNumber`, `call`, `sendRawTransaction`, etc.) required by the Engine spec.
    *   It specifically uses `EngineEthApi` wrapper to limit the scope of exposed `eth_` methods.
4.  **Security**: `AuthServerConfig::start` enforces `AuthLayer` with a `JwtSecret`.

*   **Cite**: `crates/rpc/rpc-builder/src/auth.rs` and `crates/rpc/rpc-api/src/engine.rs`

---

### 5. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `RethRpcModule` | `crates/rpc/rpc-server-types/src/module.rs` | Enum of supported namespaces (`Eth`, `Debug`, `Trace`, etc.). |
| `RpcModuleSelection` | `crates/rpc/rpc-server-types/src/module.rs` | logic for selecting modules (e.g., "All", "Standard", or specific list). |
| `TransportRpcModuleConfig` | `crates/rpc/rpc-builder/src/lib.rs` | Maps `RpcModuleSelection` to specific transports (Http, Ws, Ipc). |
| `RpcRegistryInner` | `crates/rpc/rpc-builder/src/lib.rs` | Holds the live instances of `EthApi`, `Pool`, `Network` and generates `Methods` on demand. |
| `RpcRequestRateLimiter` | `crates/rpc/rpc-builder/src/rate_limiter.rs` | Tower layer for rate-limiting expensive calls (like `trace_` or `debug_`) using a Semaphore. |
| `EthHandlers` | `crates/rpc/rpc-builder/src/eth.rs` | A struct grouping `EthApi`, `EthFilter`, and `EthPubSub` so they can be registered together. |

---

### 6. Where to Look Next

1.  **`crates/rpc/rpc-builder/src/lib.rs` -> `reth_methods`**: This massive match statement is the definitive source of truth for which Rust struct implements which RPC namespace string (e.g., "debug" maps to `DebugApi`).
2.  **`crates/rpc/rpc-builder/src/config.rs`**: Look here to see how CLI flags translate into the server configuration.
3.  **`crates/rpc/rpc-builder/src/auth.rs`**: To understand exactly how the Engine API is secured and isolated from the standard user-facing RPC.

---

### 7. Suggested Follow-up Questions

*   "How does `EthApi` handle the difference between `eth_call` (state access) and `eth_sendRawTransaction` (pool access) internally?"
*   "How are blocking tasks (like `debug_traceTransaction`) managed to prevent stalling the async RPC server loop?" (Hint: Look at `SpawnBlocking` in `helpers/blocking_task.rs`).
*   "How does the `RpcModuleBuilder` handle IPC socket permissions and endpoints on different operating systems?"

## Verification
- `RpcModuleBuilder::build` and `build_with_auth_server` create a registry, build transport modules, and optionally build the auth module.```322:399:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
pub fn build_with_auth_server<EthApi>(...) -> (TransportRpcModules, AuthRpcModule, RpcRegistryInner<...>) { ... }
...
pub fn build<EthApi>(...) -> TransportRpcModules<()> { ... }
```
- `RpcRegistryInner` holds `EthHandlers`, a module cache, and lazily generates `Methods` via `reth_methods`.```521:553:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
pub fn new(...) -> Self { ... let eth = EthHandlers::bootstrap(...); ... modules: Default::default(), ... }
```
```917:1007:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
pub fn reth_methods(...) -> Vec<Methods> { ... match namespace { RethRpcModule::Eth => { ... } ... } }
```
- `reth_methods` merges `EthApi`, `EthFilter`, `EthPubSub`, and `EthBundle` for the `eth_` namespace; other namespaces create their own APIs on demand.```947:999:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
RethRpcModule::Eth => { ... module.merge(eth_filter...).merge(eth_pubsub...)... EthBundle::new(...) ... }
```
- Auth module wiring merges engine API and a restricted `EngineEthApi` subset.```857:871:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
pub fn create_auth_module(&self, engine_api: impl IntoEngineApiRpcModule) -> AuthRpcModule { ... EngineEthApi::new(...) ... module.merge(engine_eth.into_rpc()) ... }
```
```24:147:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/engine.rs
pub struct EngineEthApi<...> { ... }
// methods: syncing, chain_id, block_number, call, get_code, block_by_hash, block_by_number, block_receipts, send_raw_transaction, transaction_receipt, logs, get_proof
```
- `AuthServerConfig::start` enforces JWT via `AuthLayer` and starts the auth server (HTTP/WS + optional IPC).```70:116:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/auth.rs
let middleware = tower::ServiceBuilder::new().layer(AuthLayer::new(JwtAuthValidator::new(secret)));
...
let server = ServerBuilder::new()...set_http_middleware(middleware)...build(socket_addr)...
```
- `RpcServerConfig::start` applies CORS/JWT/compression layers and uses `rpc_middleware` for custom RPC middleware.```1298:1448:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
ServerBuilder::new()
    .set_http_middleware(... option_layer(Self::maybe_cors_layer(...)) ... option_layer(Self::maybe_jwt_layer(...)) ... option_layer(Self::maybe_compression_layer(...)))
    .set_rpc_middleware(RpcServiceBuilder::default().layer(...).layer(self.rpc_middleware.clone()))
```
- `TransportRpcModules` stores per-transport `RpcModule` instances.```1629:1639:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
pub struct TransportRpcModules<Context = ()> { config: TransportRpcModuleConfig, http: Option<RpcModule<Context>>, ws: Option<RpcModule<Context>>, ipc: Option<RpcModule<Context>> }
```
- `RpcServerArgs` produces `TransportRpcModuleConfig`, defaulting http/ws to `standard_modules` and ipc to `default_ipc_modules`.```140:162:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/config.rs
if self.http { config = config.with_http(self.http_api.clone().unwrap_or_else(|| RpcModuleSelection::standard_modules().into())); }
if self.ws { config = config.with_ws(self.ws_api.clone().unwrap_or_else(|| RpcModuleSelection::standard_modules().into())); }
if self.is_ipc_enabled() { config = config.with_ipc(RpcModuleSelection::default_ipc_modules()); }
```
- `RethRpcModule` includes more namespaces than the summary list (txpool, rpc, reth, ots, mev, flashbots, etc.).```295:327:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-server-types/src/module.rs
pub enum RethRpcModule { Admin, Debug, Eth, Net, Trace, Txpool, Web3, Rpc, Reth, Ots, Flashbots, Miner, Mev, Testing, Other(String) }
```

## Corrections / Caveats
- `RpcRequestRateLimiter` is provided as a middleware layer, but it is not wired into `RpcServerConfig::start` by default; it must be injected via `rpc_middleware` if desired.
- The `register_*` helpers exist, but the transport flow uses `reth_methods` to lazily instantiate methods for selected modules.
- Engine API wiring uses `EngineEthApi` from `crates/rpc/rpc/src/engine.rs`, not `crates/rpc/rpc-api/src/engine.rs`.

## Actionable Pointers
- Module wiring and transport mapping: `reth/crates/rpc/rpc-builder/src/lib.rs` (`build`, `create_transport_rpc_modules`, `reth_methods`).
- CLI to server config: `reth/crates/rpc/rpc-builder/src/config.rs` (`transport_rpc_module_config`, `rpc_server_config`, `auth_server_config`).
- Auth server and engine subset: `reth/crates/rpc/rpc-builder/src/auth.rs`, `reth/crates/rpc/rpc/src/engine.rs`.
