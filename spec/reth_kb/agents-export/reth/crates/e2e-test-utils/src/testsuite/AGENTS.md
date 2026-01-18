# testsuite

## Purpose
E2E test framework for reth: defines the multi-node `Environment` model, RPC/Engine API client wrappers, and setup/build utilities; test logic is expressed as composable `Action`s (produce blocks, forks/reorgs, sync assertions, engine API scenarios).

## Contents (one hop)
### Subdirectories
- [x] `actions/` - Action library for driving nodes and asserting behavior (block production, FCU/newPayload, forks/reorgs, sync ops).
- [x] `assets/` - (skip: static assets) Test fixtures (e.g. `genesis.json`) used by setup/import helpers.

### Files
- `mod.rs` - framework core types: `Environment`, `NodeClient`, `NodeState`, block registry/tagging, and shared state tracking for Engine API testing.
  - **Key items**: `Environment<I>`, `NodeClient<Payload>`, `NodeState<I>`, `BlockInfo`
- `setup.rs` - setup helper that creates node networks (optionally importing an RLP chain), waits for readiness, and initializes environment state.
  - **Key items**: `Setup<I>`, `Setup::apply()`, `Setup::apply_with_import()`, `NetworkSetup`
- `README.md` - documentation and usage guide for the e2e framework and action catalog.
  - **Key items**: `binary(e2e_testsuite)` nextest conventions, action categories

## Key APIs (no snippets)
- **Framework state**: `Environment<I>`, `NodeClient<I>`, `NodeState<I>`, `BlockInfo`
- **Setup**: `Setup<I>`
- **Actions**: `actions::Action<I>`, `actions::Sequence<I>`

## Relationships
- **Used by**: crate-level e2e test binaries under `tests/e2e-testsuite/` across the workspace.
- **Connects**: node instances <-> JSON-RPC + Engine API clients <-> high-level `Action` sequences that mutate/inspect environment state.
