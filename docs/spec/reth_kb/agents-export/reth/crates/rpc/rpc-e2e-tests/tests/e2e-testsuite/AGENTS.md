# e2e-testsuite

## Purpose
Execution-apis RPC compatibility test suite entrypoints.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Runs local and external execution-apis compatibility tests.
- **Key items**: `test_local_rpc_tests_compat()`, `test_execution_apis_compat()`, `RunRpcCompatTests`
- **Interactions**: Uses `InitializeFromExecutionApis` and e2e test framework actions.

## End-to-end flow (high level)
- Load test data paths (repo-local or env-based).
- Initialize chain state from RLP + forkchoice.
- Run compatibility test actions across discovered methods.

## Key APIs (no snippets)
- `RunRpcCompatTests`, `InitializeFromExecutionApis`
