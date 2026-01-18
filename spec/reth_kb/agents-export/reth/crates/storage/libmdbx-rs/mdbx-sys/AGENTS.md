# mdbx-sys

## Purpose
FFI layer and build glue for vendored libmdbx.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Generated Rust bindings.
- [x] `libmdbx/` - Vendored libmdbx sources.

### Files
- `build.rs` - Build script to compile and bind libmdbx.
- `Cargo.toml` - Crate manifest for MDBX FFI bindings.

## Key APIs (no snippets)
- **Modules / Packages**: `libmdbx` (vendor), `bindings`
