# server

## Purpose
IPC server implementation for JSON-RPC: connection handling, request processing, and middleware wiring.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Main IPC server implementation with settings, middleware builders, and connection lifecycle orchestration.
- **Key items**: `IpcServer`, `Settings`, `Builder`, `RpcServiceBuilder`, `TowerServiceNoHttp`, `ServiceData`
- **Interactions**: Uses `IpcConnDriver` from `connection` and `RpcService` from `rpc_service` to execute methods.
- **Knobs / invariants**: Limits for request/response size, connections, subscriptions, and message buffer capacity; optional socket permissions.

### `connection.rs`
- **Role**: IPC connection stream/sink adapter and driver that multiplexes requests and responses.
- **Key items**: `IpcConn`, `IpcConnDriver`, `JsonRpcStream`
- **Interactions**: Wraps `StreamCodec` framing and drives a `tower::Service` for RPC handling.
- **Knobs / invariants**: Flushes on readiness to avoid buffering stalls; uses a small budget to yield.

### `ipc.rs`
- **Role**: Request parsing and execution helpers for single and batch JSON-RPC calls.
- **Key items**: `process_single_request`, `process_batch_request`, `call_with_service`, `execute_call_with_tracing`
- **Interactions**: Calls `RpcServiceT` methods and builds JSON-RPC responses.
- **Knobs / invariants**: Enforces max request size; batches are assembled into a single response message.

### `rpc_service.rs`
- **Role**: Middleware wrapper implementing `RpcServiceT` to route calls and subscriptions.
- **Key items**: `RpcService`, `RpcServiceCfg`, `call`, `batch`
- **Interactions**: Uses `Methods` registry, `MethodSink` for subscriptions, and `BoundedSubscriptions` limits.
- **Knobs / invariants**: Rejects subscriptions when limits are exceeded; enforces response size limits.

## End-to-end flow (high level)
- Build an `IpcServer` with `Builder` and configure limits/middleware.
- Accept socket connections and wrap them with `StreamCodec` framing.
- Spawn an `IpcConnDriver` task to read requests and write responses.
- Parse single or batch requests and dispatch to `RpcService`.
- Enforce subscription and payload size limits, then send responses back.

## Key APIs (no snippets)
- `IpcServer`, `Builder`, `Settings`
- `IpcConn`, `IpcConnDriver`
- `RpcService`, `RpcServiceCfg`
