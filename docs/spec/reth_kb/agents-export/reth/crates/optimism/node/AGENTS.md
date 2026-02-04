# node

## Purpose
`reth-optimism-node` crate: OP node type definitions, component wiring, and engine/RPC integration.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP node config, engine types, and component builders.
- [x] `tests/` - E2E and integration tests for OP node behavior.

### Files
- `Cargo.toml` - Manifest for OP node dependencies and features.
  - **Key items**: features `js-tracer`, `test-utils`, `reth-codec`; deps `reth-node-builder`, `reth-optimism-rpc`
