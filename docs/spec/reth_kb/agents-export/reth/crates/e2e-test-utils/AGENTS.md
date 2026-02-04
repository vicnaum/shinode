# e2e-test-utils

## Purpose
`reth-e2e-test-utils` crate: shared end-to-end testing utilities for reth, including multi-node in-process setup helpers, RPC/Engine API clients, an action-based testsuite framework, and an example `e2e_testsuite` test binary.

## Contents (one hop)
### Subdirectories
- [x] `src/` - framework and helpers for launching nodes, driving payloads/blocks, and running action sequences.
- [x] `tests/` - `e2e_testsuite` test binary entrypoint using the framework.

### Files
- `Cargo.toml` - crate manifest and test-binary registration.
  - **Key items**: `[[test]] e2e_testsuite`

## Key APIs (no snippets)
- **Framework**: `testsuite::Environment`, `testsuite::Setup`, `testsuite::actions::*`
- **Builder**: `E2ETestSetupBuilder`

## Relationships
- **Used by**: other crates' `tests/e2e-testsuite/*` binaries across the workspace (shared e2e patterns + helpers).
