# it

## Purpose
Integration tests for RPC builder configuration, auth server, middleware, and transport startup.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring for integration tests.
- **Key items**: `auth`, `http`, `middleware`, `serde`, `startup`, `utils`

### `auth.rs`
- **Role**: Auth server integration tests for engine API endpoints.
- **Key items**: `test_basic_engine_calls()`, `test_auth_endpoints_http()`, `test_auth_endpoints_ws()`

### `http.rs`
- **Role**: HTTP/WS RPC endpoint compatibility tests.
- **Key items**: `RawRpcParamsBuilder`, `test_rpc_call_ok()`, `test_filter_calls()`, `test_basic_eth_calls()`

### `middleware.rs`
- **Role**: Custom RPC middleware integration test.
- **Key items**: `MyMiddlewareLayer`, `MyMiddlewareService`, `test_rpc_middleware()`

### `serde.rs`
- **Role**: Serde and raw-params handling tests.
- **Key items**: `RawRpcParams`, `test_eth_balance_serde()`

### `startup.rs`
- **Role**: Server startup and port conflict tests.
- **Key items**: `test_http_addr_in_use()`, `test_ws_addr_in_use()`, `test_launch_same_port_different_modules()`

### `utils.rs`
- **Role**: Test helpers to launch RPC servers and build test modules.
- **Key items**: `test_address()`, `launch_auth()`, `launch_http()`, `launch_ws()`, `test_rpc_builder()`

## End-to-end flow (high level)
- Build test RPC modules with `test_rpc_builder`.
- Launch HTTP/WS/auth servers with helper utilities.
- Exercise core namespaces and engine API endpoints.
- Validate middleware hooks, serde behavior, and startup error handling.

## Key APIs (no snippets)
- `launch_auth()`, `launch_http()`, `launch_ws()`
- `test_basic_engine_calls()`, `test_rpc_middleware()`
