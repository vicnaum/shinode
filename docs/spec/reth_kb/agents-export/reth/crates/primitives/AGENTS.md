# primitives

## Purpose
`reth-primitives` crate: common block, transaction, and receipt types plus benchmarks.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for signature recovery and blob validation.
- [x] `src/` - Primitive type aliases and transaction helpers.

### Files
- `Cargo.toml` - Manifest for primitives dependencies and feature flags.
  - **Key items**: features `reth-codec`, `c-kzg`, `arbitrary`, `serde-bincode-compat`
