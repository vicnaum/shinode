# it

## Purpose
Integration tests for `reth-node-ethereum` focused on node builder wiring, add-ons, and extension points (RPC hooks and ExEx installation).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - test module root that composes the `it` test suite.
- `builder.rs` - `NodeBuilder` setup tests: basic configuration, callbacks (`on_component_initialized`, `on_node_started`, `on_rpc_started`), launcher usage, and custom tokio runtime wiring for add-ons.
- `exex.rs` - ExEx integration test: installs a dummy execution extension via `install_exex()` and checks launch wiring.
- `testing.rs` - E2E-style test for the `testing_` RPC namespace: wires only `RethRpcModule::Testing` and validates `testing_buildBlockV1` works end-to-end.
