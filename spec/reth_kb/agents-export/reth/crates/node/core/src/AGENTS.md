# src

## Purpose
Core node utilities and configuration: CLI argument types, config helpers, directory paths, version metadata, and startup/shutdown utilities.

## Contents (one hop)
### Subdirectories
- [x] `args/` - CLI argument groups for node subsystems.
- [x] `cli/` - CLI configuration traits for payload builder, network, and txpool.

### Files
- `lib.rs` - Crate entrypoint and module exports; re-exports primitives and RPC compat helpers.
  - **Key items**: modules `args`, `cli`, `dirs`, `node_config`, `utils`, `version`
- `node_config.rs` - `NodeConfig` type and helpers for building full node configuration.
  - **Key items**: `NodeConfig`, `DEFAULT_CROSS_BLOCK_CACHE_SIZE_MB`
- `dirs.rs` - Data directory and XDG path helpers.
  - **Key items**: `data_dir()`, `PlatformPath`, `ChainPath`, `MaybePlatformPath`
- `exit.rs` - Future for waiting on node shutdown conditions.
  - **Key items**: `NodeExitFuture`
- `utils.rs` - Startup helpers (path parsing, JWT secret handling) and single header/body fetch.
  - **Key items**: `parse_path()`, `get_or_create_jwt_secret_from_path()`, `get_single_header()`, `get_single_body()`
- `version.rs` - Version metadata constants and helpers.
  - **Key items**: `RethCliVersionConsts`, `version_metadata()`, `default_extra_data()`

## Key APIs (no snippets)
- `NodeConfig`
- `PlatformPath`, `ChainPath`
- `RethCliVersionConsts`
