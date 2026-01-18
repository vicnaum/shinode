# launch

## Purpose
Node launch orchestration: type-state launch context, engine/debug launchers, execution extensions, and invalid-block hooks.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Launch trait and module wiring.
  - **Key items**: `LaunchNode`
- `common.rs` - Type-state launch context with attachments for configs, providers, and components.
  - **Key items**: `LaunchContext`, `LaunchContextWith`, `Attached`, `WithConfigs`, `WithComponents`
- `engine.rs` - Main engine node launcher integrating pipeline, network, and engine APIs.
  - **Key items**: `EngineNodeLauncher`
- `debug.rs` - Debug launcher that wraps a launcher to add RPC/Etherscan consensus clients.
  - **Key items**: `DebugNode`, `DebugNodeLauncher`, `DebugNodeLauncherFuture`
- `exex.rs` - Execution extension launcher that wires ExEx manager and WAL.
  - **Key items**: `ExExLauncher`
- `invalid_block_hook.rs` - Invalid block hook creation helpers.
  - **Key items**: `InvalidBlockHookExt`, `create_invalid_block_hook()`

## Key APIs (no snippets)
- `LaunchContext`, `LaunchNode`
- `EngineNodeLauncher`, `DebugNodeLauncher`, `ExExLauncher`
