# config

## Purpose
`reth-config` crate: standalone configuration schema for reth (pipeline stage knobs, pruning modes, peer/session settings, static-file layout) with optional TOML serialization/deserialization via the `serde` feature.

## Contents (one hop)
### Subdirectories
- [x] `src/` - configuration structs and (feature-gated) TOML load/save helpers.

### Files
- `Cargo.toml` - crate manifest (optional `serde`/`toml` support).
  - **Key items**: `features.serde`

## Key APIs (no snippets)
- **Types**: `Config`, `StageConfig`, `PruneConfig`, `StaticFilesConfig`

## Relationships
- **Used by**: node/CLI configuration loading (feeds settings into storage, pipeline stages, and pruning/static-file behavior).
