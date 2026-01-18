# rpc

## Purpose
RPC subsystem crates: interfaces, implementations, builders, and supporting types.

## Contents (one hop)
### Subdirectories
- [x] `ipc/` - IPC transport client/server for JSON-RPC.
- [x] `rpc/` - Concrete RPC handler implementations for all namespaces.
- [x] `rpc-api/` - JSON-RPC trait definitions for all namespaces.
- [x] `rpc-builder/` - RPC module assembly and server configuration helpers.
- [x] `rpc-convert/` - Conversion layer between primitives and RPC types.
- [x] `rpc-e2e-tests/` - End-to-end RPC compatibility tests (execution-apis).
- [x] `rpc-engine-api/` - Engine API implementation for consensus clients.
- [x] `rpc-eth-api/` - Shared `eth_` namespace traits and helpers.
- [x] `rpc-eth-types/` - `eth` RPC types, config, caches, and errors.
- [x] `rpc-layer/` - RPC middleware layers (auth, compression).
- [x] `rpc-server-types/` - Shared server constants and module selection.
- [x] `rpc-testing-util/` - Helpers for testing debug/trace RPC APIs.

### Files
- (none)
