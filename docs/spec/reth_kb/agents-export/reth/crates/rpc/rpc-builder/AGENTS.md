# rpc-builder

## Purpose
`reth-rpc-builder` crate: configure RPC modules and start HTTP/WS/IPC/auth servers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC builders, configs, middleware, and metrics.
- [x] `tests/` - Integration tests for RPC server startup and middleware.

### Files
- `Cargo.toml` - Manifest for RPC builder utilities and server dependencies.
  - **Key items**: `reth-ipc`, `jsonrpsee`, `reth-rpc`, `reth-rpc-layer`
