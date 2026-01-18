# bin

## Purpose
`op-reth` binary crate: CLI entrypoint and feature wiring for op-reth.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Binary entrypoint and re-exports.

### Files
- `Cargo.toml` - Manifest with op-reth binary dependencies and features.
  - **Key items**: features `jemalloc`, `js-tracer`, `keccak-cache-global`, `asm-keccak`
