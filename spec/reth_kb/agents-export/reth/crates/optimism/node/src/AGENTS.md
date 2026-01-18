# src

## Purpose
Optimism node configuration, components, and engine/RPC wiring.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and re-exports for OP node components.
  - **Key items**: `OpEngineTypes`, `OpEngineApiBuilder`, `OpPayloadBuilder`, `OpStorage`
- `args.rs` - Rollup CLI arguments for OP nodes.
  - **Key items**: `RollupArgs`
- `engine.rs` - Engine types and validator for OP engine API.
  - **Key items**: `OpEngineTypes`, `OpEngineValidator`
- `node.rs` - Optimism node type and component builder wiring.
  - **Key items**: `OpNode`, `OpNodeComponentBuilder`, `OpAddOnsBuilder`
- `rpc.rs` - RPC engine API builder for OP.
  - **Key items**: `OpEngineApiBuilder`
- `utils.rs` - Test utilities for OP node setup and payload attributes.
  - **Key items**: `setup()`, `advance_chain()`, `optimism_payload_attributes()`
- `version.rs` - OP client name constant.
  - **Key items**: `OP_NAME_CLIENT`

## Key APIs (no snippets)
- `OpNode`, `OpEngineTypes`, `OpEngineValidator`
