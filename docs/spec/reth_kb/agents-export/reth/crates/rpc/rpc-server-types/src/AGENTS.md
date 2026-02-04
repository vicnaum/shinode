# src

## Purpose
Shared RPC server constants, module selection, and error helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public re-exports.
- **Key items**: `RethRpcModule`, `RpcModuleSelection`, `ToRpcResult`

### `constants.rs`
- **Role**: Default constants for RPC servers and gas/trace limits.
- **Key items**: `DEFAULT_HTTP_RPC_PORT`, `DEFAULT_MAX_BLOCKS_PER_FILTER`, `DEFAULT_PROOF_PERMITS`, `gas_oracle::*`

### `module.rs`
- **Role**: RPC module enums and selection utilities.
- **Key items**: `RethRpcModule`, `RpcModuleSelection`, `RpcModuleValidator`
- **Knobs / invariants**: Supports `All`, `Standard`, or explicit selections.

### `result.rs`
- **Role**: RPC error conversion helpers and macros.
- **Key items**: `ToRpcResult`, `impl_to_rpc_result!`, `rpc_err()`, `invalid_params_rpc_err()`

## End-to-end flow (high level)
- Use constants to configure RPC servers and limits.
- Select modules via `RpcModuleSelection` and validators.
- Map internal errors to JSON-RPC error objects with `ToRpcResult`.

## Key APIs (no snippets)
- `RethRpcModule`, `RpcModuleSelection`, `ToRpcResult`
