# src

## Purpose
Node builder implementation: type-safe component assembly, launch orchestration, RPC wiring, hooks, and execution extensions.

## Contents (one hop)
### Subdirectories
- [x] `builder/` - Node builder state machine and add-on containers.
- [x] `components/` - Component builder traits for pool/network/payload/consensus/EVM.
- [x] `launch/` - Launch contexts and engine/debug/ExEx launchers.

### Files
- `lib.rs` - Crate entrypoint and re-exports for builder, components, launch, and RPC helpers.
  - **Key items**: `NodeBuilder`, `EngineNodeLauncher`, `DebugNodeLauncher`, `NodeHandle`
- `node.rs` - Node trait presets and `FullNode` wrapper with RPC handles.
  - **Key items**: `Node`, `AnyNode`, `FullNode`, `ComponentsFor`
- `rpc.rs` - RPC add-ons, hooks, and builders for Engine API and Eth API.
  - **Key items**: `RethRpcServerHandles`, `RpcHooks`, `RpcRegistry`, `RethRpcAddOns`
- `setup.rs` - Pipeline and downloader setup helpers.
  - **Key items**: `build_pipeline()`, `build_networked_pipeline()`
- `hooks.rs` - Hook containers for component initialization and node start.
  - **Key items**: `NodeHooks`, `OnComponentInitializedHook`, `OnNodeStartedHook`
- `handle.rs` - `NodeHandle` wrapper with node components and exit future.
  - **Key items**: `NodeHandle`
- `exex.rs` - Traits for launching execution extensions (ExEx).
  - **Key items**: `LaunchExEx`, `BoxedLaunchExEx`, `BoxExEx`
- `engine_api_ext.rs` - Wrapper to capture built Engine API instances.
  - **Key items**: `EngineApiExt`
- `aliases.rs` - Trait alias for block reader bounds.
  - **Key items**: `BlockReaderFor`

## Key APIs (no snippets)
- `NodeBuilder`, `NodeHandle`, `FullNode`
- `RethRpcAddOns`, `RpcRegistry`
