# builder

## Purpose
Node builder state machine: types and helpers that progressively configure node types, components, add-ons, and launch context.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Main `NodeBuilder` implementation and builder flow utilities.
  - **Key items**: `NodeBuilder`, `WithLaunchContext`, `BuilderContext`, `RethFullAdapter`
- `states.rs` - Builder state types and adapters for configured types/components.
  - **Key items**: `NodeBuilderWithTypes`, `NodeTypesAdapter`, `NodeAdapter`, `NodeBuilderWithComponents`
- `add_ons.rs` - Container for node add-ons, hooks, and execution extensions.
  - **Key items**: `AddOns`

## Key APIs (no snippets)
- `NodeBuilder`
- `NodeBuilderWithComponents`
