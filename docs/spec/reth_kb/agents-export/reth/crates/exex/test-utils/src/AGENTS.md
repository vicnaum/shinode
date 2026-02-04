# src

## Purpose
Implements `reth-exex-test-utils`: utilities for testing ExEx integrations, including a lightweight `TestNode` configuration, helpers to construct `ExExContext` instances, and harness types to drive notifications/events during tests.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - test node and harness wiring: provides `TestNode`, pool/executor/consensus builders, `TestExExContext`/`TestExExHandle`, and helpers to create an ExEx context with an ephemeral provider factory + WAL.
  - **Key items**: `TestNode`, `TestPoolBuilder`, `TestExecutorBuilder`, `TestConsensusBuilder`, `TestExExHandle`, `test_exex_context_with_chain_spec()`
