# src

## Purpose
Node API traits and configuration interfaces for assembling a full reth node with pluggable components and add-ons.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint that re-exports engine/payload/payload-builder/EVM abstractions and node types.
  - **Key items**: re-exports `reth_engine_primitives`, `reth_payload_primitives`, `reth_payload_builder_primitives`, `ConfigureEvm`, `FullProvider`
- `node.rs` - Core node configuration traits and add-on interface.
  - **Key items**: `FullNodeTypes`, `FullNodeTypesAdapter`, `FullNodeComponents`, `PayloadBuilderFor`, `NodeAddOns`, `AddOnsContext`

## Key APIs (no snippets)
- `FullNodeTypes`, `FullNodeComponents`
- `NodeAddOns`, `AddOnsContext`
