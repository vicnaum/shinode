# src

## Purpose
RPC server configuration and builder utilities: module selection, server setup, middleware, and metrics.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Main RPC builder API for assembling modules and starting servers.
- **Key items**: `RpcModuleBuilder`, `RpcModuleConfig`, `RpcServerConfig`, `TransportRpcModuleConfig`, `TransportRpcModules`
- **Interactions**: Wires `reth-rpc` handlers with `jsonrpsee` servers and transport configs.

### `auth.rs`
- **Role**: Auth (engine API) server configuration and startup helpers.
- **Key items**: `AuthServerConfig`, `AuthServerConfigBuilder`, `AuthRpcModule`, `AuthServerHandle`
- **Interactions**: Applies JWT auth middleware and optional IPC server startup.

### `config.rs`
- **Role**: RPC server config trait for CLI integration.
- **Key items**: `RethRpcServerConfig`, `transport_rpc_module_config()`, `rpc_server_config()`, `auth_server_config()`
- **Interactions**: Builds module selections and server configs from CLI args.

### `cors.rs`
- **Role**: CORS parsing and layer creation.
- **Key items**: `CorsDomainError`, `create_cors_layer()`
- **Knobs / invariants**: Disallows wildcard mixed with explicit domains.

### `error.rs`
- **Role**: RPC server error types and conflict detection.
- **Key items**: `RpcError`, `ServerKind`, `ConflictingModules`, `WsHttpSamePortError`
- **Knobs / invariants**: Distinguishes address-in-use vs generic start failures.

### `eth.rs`
- **Role**: Bundles `eth` core, filter, and pubsub handlers.
- **Key items**: `EthHandlers`, `bootstrap()`
- **Interactions**: Creates `EthFilter` and `EthPubSub` alongside main `EthApi`.

### `metrics.rs`
- **Role**: Metrics middleware for RPC requests and connections.
- **Key items**: `RpcRequestMetrics`, `RpcRequestMetricsService`, `MeteredRequestFuture`, `MeteredBatchRequestsFuture`
- **Interactions**: Wraps `RpcServiceT` to record method timings and counts.

### `middleware.rs`
- **Role**: Trait alias for supported RPC middleware layers.
- **Key items**: `RethRpcMiddleware`

### `rate_limiter.rs`
- **Role**: Rate limiting middleware for expensive methods.
- **Key items**: `RpcRequestRateLimiter`, `RpcRequestRateLimitingService`, `RateLimitingRequestFuture`
- **Knobs / invariants**: Applies semaphore limits to `trace_` and `debug_` calls.

## End-to-end flow (high level)
- Build RPC handlers via `RpcModuleBuilder` with provider/pool/network/consensus.
- Configure module selection per transport in `TransportRpcModuleConfig`.
- Start HTTP/WS/IPC servers via `RpcServerConfig` or auth server via `AuthServerConfig`.
- Apply middleware layers (auth, CORS, rate limiting, metrics).
- Serve `jsonrpsee` modules with the configured API set.

## Key APIs (no snippets)
- `RpcModuleBuilder`, `RpcServerConfig`, `TransportRpcModules`
- `AuthServerConfig`, `AuthRpcModule`
- `RpcRequestMetrics`, `RpcRequestRateLimiter`
