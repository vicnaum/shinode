# core

## Purpose
`reth-node-core` crate: core node configuration, CLI args, directory management, and version/build metadata utilities.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Node config types, CLI args, and core utilities.

### Files
- `Cargo.toml` - Manifest with core node dependencies and build features.
  - **Key items**: deps `reth-network`, `reth-transaction-pool`, `reth-config`; features `otlp`, `tracy`, log level gates
- `build.rs` - Build-time version metadata generation using vergen.
  - **Key items**: `VERGEN_*` envs, `RETH_SHORT_VERSION`, `RETH_LONG_VERSION_*`

## Key APIs (no snippets)
- `NodeConfig`
