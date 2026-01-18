# src

## Purpose
End-to-end RPC compatibility testing utilities and actions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and module wiring.
- **Key items**: `rpc_compat`

### `rpc_compat.rs`
- **Role**: Actions for running execution-apis RPC compatibility tests.
- **Key items**: `RpcTestCase`, `RunRpcCompatTests`, `InitializeFromExecutionApis`, `parse_io_file()`, `compare_json_values()`
- **Interactions**: Integrates with `reth-e2e-test-utils` action framework.

## End-to-end flow (high level)
- Load execution-apis test data from `.io` files.
- Convert test cases into `RpcTestCase` structs.
- Execute JSON-RPC requests against node clients.
- Compare actual responses against expected fixtures.

## Key APIs (no snippets)
- `RunRpcCompatTests`, `RpcTestCase`
