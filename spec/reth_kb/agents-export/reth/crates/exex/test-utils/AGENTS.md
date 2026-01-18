# test-utils

## Purpose
`reth-exex-test-utils` crate: helpers for writing ExEx tests-builds a minimal test node configuration and provides utilities to create and drive `ExExContext` instances with in-memory/on-disk test infrastructure.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `TestNode` + ExEx context harness utilities.

### Files
- `Cargo.toml` - crate manifest (pulls in test-utils feature sets across provider/db/network/node-builder and Ethereum node wiring).

## Key APIs (no snippets)
- `test_exex_context_with_chain_spec()`
- `TestExExHandle`
