# Q023 - RPC Safety Knobs

Status: verified

## Question
What RPC safety knobs exist (bind/auth/rate limit)?

## Context Packs
- `spec/reth_repo_context/context-rpc.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-rpc.xml

Question (Q023):
What RPC safety knobs exist (bind/auth/rate limit)?
I want how CLI flags map into server config, what defaults exist, and how middleware is applied for http/ws/ipc and auth.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Include defaults for ports, JWT, CORS, compression, max sizes, and max connections.
3. Explain how auth RPC differs from regular RPC in configuration and middleware.
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
Based on the `context-rpc.xml` pack, here is the analysis of the RPC safety mechanisms in Reth.

### 1. Summary
*   **Layered Safety**: Reth applies safety controls at multiple levels: **Network** (Socket binding, CORS), **Protocol** (JWT Auth, Compression), **Server** (Request/Response sizes, Max connections), and **Application** (Semaphores for heavy EVM tasks).
*   **Secure Defaults**: Servers bind to `127.0.0.1` (localhost) by default to prevent accidental public exposure.
*   **Mandatory Auth for Engine**: The Engine API (`auth_server`) strictly requires a JWT secret, whereas the regular RPC server makes authentication optional.
*   **Resource Semaphores**: Instead of simple request-per-second rate limiting, Reth limits **concurrency** based on workload type. It uses specific semaphores for "Blocking IO" (e.g., `eth_call`) and "CPU Heavy" (e.g., `debug_traceTransaction`) tasks to prevent starvation.
*   **DoS Protection**: Strict limits on request body size (15MB default) and response body size (160MB default) prevent memory exhaustion attacks.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/node/core/src/args/rpc_server.rs` | `RpcServerArgs` | Defines all CLI flags (`--http`, `--ws`, `--rpc.gascap`) and their default values. |
| `crates/rpc/rpc-builder/src/config.rs` | `RethRpcServerConfig` | Maps CLI arguments into the internal `RpcServerConfig` builder. |
| `crates/rpc/rpc-builder/src/lib.rs` | `RpcServerConfig` | Builds the `jsonrpsee` server, applying middleware layers (CORS, Auth, Compression). |
| `crates/rpc/rpc-builder/src/auth.rs` | `AuthServerConfig` | specialized builder for the Engine API (port 8551) with mandatory authentication. |
| `crates/rpc/rpc-eth-api/src/helpers/blocking_task.rs` | `SpawnBlocking` | Trait defining how internal semaphores (`blocking_io_task_guard`, `tracing_task_guard`) are acquired. |

---

### 3. Safety Mechanisms & Configuration

#### A. Network Binding & CORS
*   **Defaults**: HTTP/WS bind to `127.0.0.1`.
*   **Flags**: `--http.addr`, `--ws.addr`.
*   **CORS**: Configured via `CorsLayer`.
    *   **Http**: `--http.corsdomain`.
    *   **WS**: `--ws.origins`.
    *   **Logic**: If `*` is provided, it allows any origin. Otherwise, it parses comma-separated values.
    *   **Conflict Check**: If HTTP and WS run on the same port, their CORS settings must match, or `WsHttpSamePortError` is returned (`crates/rpc/rpc-builder/src/config.rs`).

#### B. Authentication (JWT)
*   **Engine API (Auth Server)**:
    *   **Mandatory**: `AuthServerConfig::start` adds `AuthLayer` with `JwtAuthValidator`.
    *   **Secret**: Loaded via `--authrpc.jwtsecret`. If missing, one is auto-generated in the data dir.
    *   **File**: `crates/rpc/rpc-builder/src/auth.rs`.
*   **Regular RPC**:
    *   **Optional**: Configured via `--rpc.jwtsecret`.
    *   **Layer**: If present, `RpcServerConfig` adds the same `AuthLayer`.
    *   **File**: `crates/rpc/rpc-builder/src/lib.rs`.

#### C. DoS Protection (Server Limits)
These limits are applied at the `jsonrpsee` server level before requests reach Reth logic.
*   **Request Size**: `--rpc.max-request-size` (Default: 15 MB). Prevents large payloads from consuming bandwidth/memory.
*   **Response Size**: `--rpc.max-response-size` (Default: 160 MB). Important for `trace_` calls which can generate huge outputs.
*   **Connections**: `--rpc.max-connections` (Default: 500).
*   **Subscriptions**: `--rpc.max-subscriptions-per-connection` (Default: 1024).
*   **File**: `crates/node/core/src/args/rpc_server.rs`.

#### D. Application Rate Limiting (Concurrency Control)
Reth limits the concurrency of expensive operations rather than raw request counts.

1.  **Blocking IO (EVM Execution)**:
    *   **Used by**: `eth_call`, `eth_estimateGas`.
    *   **Flag**: `--rpc.max-blocking-io-requests` (Default: 256).
    *   **Mechanism**: `EthApi` holds a `Semaphore`. Requests must acquire a permit via `acquire_owned_blocking_io` before spawning the task.
    *   **File**: `crates/rpc/rpc/src/eth/core.rs` and `crates/rpc/rpc-eth-api/src/helpers/blocking_task.rs`.

2.  **Tracing (CPU Heavy)**:
    *   **Used by**: `debug_traceTransaction`, `trace_call`.
    *   **Flag**: `--rpc.max-tracing-requests`.
    *   **Default**: `std::thread::available_parallelism() - 2` (reserving cores for the rest of the node).
    *   **Mechanism**: A `BlockingTaskGuard` acts as a semaphore for the Rayon thread pool dedicated to tracing.
    *   **File**: `crates/rpc/rpc-server-types/src/constants.rs` (`default_max_tracing_requests`).

---

### 4. Key Defaults Table

| Configuration | CLI Flag | Default Value | Defined In |
| :--- | :--- | :--- | :--- |
| **HTTP Port** | `--http.port` | `8545` | `constants.rs` |
| **WS Port** | `--ws.port` | `8546` | `constants.rs` |
| **Auth Port** | `--authrpc.port` | `8551` | `constants.rs` |
| **Gas Cap** | `--rpc.gascap` | `50,000,000` | `constants.rs` |
| **Tx Fee Cap** | `--rpc.txfeecap` | `1.0` ETH | `rpc_server.rs` |
| **Max Request** | `--rpc.max-request-size` | 15 MB | `rpc_server.rs` |
| **Max Response** | `--rpc.max-response-size`| 160 MB | `rpc_server.rs` |
| **Max Logs** | `--rpc.max-logs-per-response`| 20,000 | `constants.rs` |
| **Max Trace Blocks**| `--rpc.max-trace-filter-blocks` | 100 | `constants.rs` |

---

### 5. Middleware Flow (HTTP/WS)

The `RpcServerConfig::start` method (`crates/rpc/rpc-builder/src/lib.rs`) constructs the stack:

1.  **Socket Binding**: `server.build(socket_addr)`.
2.  **HTTP Middleware** (`tower::ServiceBuilder`):
    *   `CorsLayer`: Validates Origin.
    *   `AuthLayer`: Validates JWT (if configured).
    *   `CompressionLayer`: Handles Gzip/Zstd (unless `--http.disable-compression` is set).
3.  **RPC Middleware** (`RpcServiceBuilder`):
    *   `RpcRequestMetrics`: Tracks calls/errors for Prometheus.
    *   *Custom Middleware*: (e.g., `RpcRequestRateLimiter` if injected manually, though standard CLI uses internal semaphores).
4.  **Handler**: The Request is dispatched to `EthApi` / `DebugApi`.
5.  **Internal Safety**: `EthApi` checks semaphores (`blocking_io_task_guard`) before executing.

---

### 6. Suggested Follow-up Questions
*   "How does the `JwtAuthValidator` validate tokens, and does it support key rotation?"
*   "How does the `EthStateCache` eviction policy interact with `rpc_evm_memory_limit` to prevent OOM?"
*   "What happens to a `debug_traceTransaction` request if the `BlockingTaskGuard` has zero permits available? Does it queue or fail immediately?"

## Verification
- Default binding is localhost for HTTP/WS/auth, with default ports 8545/8546/8551.```362:380:/Users/vicnaum/github/stateless-history-node/reth/crates/node/core/src/args/rpc_server.rs
http_addr: Ipv4Addr::LOCALHOST.into(),
http_port: constants::DEFAULT_HTTP_RPC_PORT,
ws_addr: Ipv4Addr::LOCALHOST.into(),
ws_port: constants::DEFAULT_WS_RPC_PORT,
auth_addr: Ipv4Addr::LOCALHOST.into(),
auth_port: constants::DEFAULT_AUTH_PORT,
```
- Default request/response size, max connections, and subscriptions are defined in `RpcServerArgs` defaults.```32:47:/Users/vicnaum/github/stateless-history-node/reth/crates/node/core/src/args/rpc_server.rs
pub(crate) const RPC_DEFAULT_MAX_SUBS_PER_CONN: u32 = 1024;
pub(crate) const RPC_DEFAULT_MAX_REQUEST_SIZE_MB: u32 = 15;
pub(crate) const RPC_DEFAULT_MAX_RESPONSE_SIZE_MB: u32 = 160;
pub(crate) const RPC_DEFAULT_MAX_CONNECTIONS: u32 = 500;
```
- Request/response sizes are converted from MB to bytes and applied to server builders.```128:173:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/config.rs
fn rpc_max_request_size_bytes(&self) -> u32 { self.rpc_max_request_size.get().saturating_mul(1024 * 1024) }
fn rpc_max_response_size_bytes(&self) -> u32 { self.rpc_max_response_size.get().saturating_mul(1024 * 1024) }
...
ServerConfigBuilder::new()
    .max_connections(self.rpc_max_connections.get())
    .max_request_body_size(self.rpc_max_request_size_bytes())
    .max_response_body_size(self.rpc_max_response_size_bytes())
    .max_subscriptions_per_connection(self.rpc_max_subscriptions_per_connection.get())
```
- CORS supports `*` or a comma list, and WS/HTTP same-port requires matching CORS.```22:49:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/cors.rs
match http_cors_domains.trim() { "*" => CorsLayer::new().allow_origin(Any) ... }
...
if iter.clone().any(|o| o == "*") { return Err(CorsDomainError::WildCardNotAllowed { ... }) }
```
```1332:1344:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
if self.http_addr == self.ws_addr && self.http_server_config.is_some() && self.ws_server_config.is_some() {
    if ws_cors.trim() != http_cors.trim() { return Err(WsHttpSamePortError::ConflictingCorsDomains { ... }.into()); }
}
```
- Regular RPC uses optional JWT auth; auth server always uses `AuthLayer`.```184:206:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/config.rs
let mut config = RpcServerConfig::default().with_jwt_secret(self.rpc_secret_key());
```
```1283:1440:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/lib.rs
option_layer(Self::maybe_jwt_layer(self.jwt_secret))
```
```70:96:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/auth.rs
let middleware = tower::ServiceBuilder::new().layer(AuthLayer::new(JwtAuthValidator::new(secret)));
```
- Auth server defaults differ (larger request/response sizes, max connections).```202:225:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/auth.rs
ServerConfig::builder()
    .max_response_body_size(750 * 1024 * 1024)
    .max_connections(500)
    .max_request_body_size(128 * 1024 * 1024)
```
- JWT secret for auth is read from file or created if missing.```239:245:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-builder/src/config.rs
match self.auth_jwtsecret.as_ref() { Some(fpath) => JwtSecret::from_file(fpath), None => get_or_create_jwt_secret_from_path(&default_jwt_path) }
```
- Blocking IO concurrency uses a semaphore and is acquired in `eth_call`.```303:307:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/core.rs
/// Semaphore to limit concurrent blocking IO requests (`eth_call`, `eth_estimateGas`, etc.)
blocking_io_request_semaphore: Arc<Semaphore>,
```
```216:226:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-api/src/helpers/call.rs
let _permit = self.acquire_owned_blocking_io().await;
```
- Tracing concurrency is governed by `BlockingTaskGuard` with default based on available CPUs.```35:43:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-server-types/src/constants.rs
pub fn default_max_tracing_requests() -> usize { ... available_parallelism ... }
```
```52:67:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/trace.rs
async fn acquire_trace_permit(&self) -> Result<OwnedSemaphorePermit, AcquireError> {
    self.inner.blocking_task_guard.clone().acquire_owned().await
}
```

## Corrections / Caveats
- HTTP/WS servers are disabled by default (`http: false`, `ws: false`), even though default bind addresses are localhost.
- CORS conflict checks happen in `RpcServerConfig::start` (lib.rs), not in config.rs.
- Auth server uses different default size limits than regular RPC (128MB request, 750MB response).
- The `RpcRequestRateLimiter` layer exists but is not attached by default; concurrency limits are primarily via semaphores.

## Actionable Pointers
- CLI defaults and knobs: `reth/crates/node/core/src/args/rpc_server.rs`.
- Server middleware + CORS/auth/compression: `reth/crates/rpc/rpc-builder/src/lib.rs` and `reth/crates/rpc/rpc-builder/src/cors.rs`.
- Auth server defaults and JWT enforcement: `reth/crates/rpc/rpc-builder/src/auth.rs`.
- Concurrency guards: `reth/crates/rpc/rpc-eth-api/src/helpers/blocking_task.rs`, `reth/crates/rpc/rpc-eth-api/src/helpers/call.rs`, `reth/crates/rpc/rpc/src/trace.rs`.
