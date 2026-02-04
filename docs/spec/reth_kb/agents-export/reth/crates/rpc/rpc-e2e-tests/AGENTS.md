# rpc-e2e-tests

## Purpose
`reth-rpc-e2e-tests` crate: end-to-end RPC compatibility tests (execution-apis).

## Contents (one hop)
### Subdirectories
- [x] `src/` - RPC compatibility test actions.
- [x] `tests/` - Test suite entrypoints for compatibility runs.
- [x] `testdata/` - (skip: execution-apis fixtures and JSON/IO test data).

### Files
- `Cargo.toml` - Manifest for RPC e2e testing dependencies.
  - **Key items**: `reth-e2e-test-utils`, `reth-rpc-api` (client)
- `README.md` - Usage and architecture for compatibility testing.
  - **Key items**: `RunRpcCompatTests`, execution-apis `.io` format
