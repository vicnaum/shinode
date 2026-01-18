# era-utils

## Purpose
`reth-era-utils` crate: utilities to import block history from downloaded `.era1` files into reth storage (static files + DB + stage checkpoints) and to export stored history back into `.era1` files (with configurable chunking and naming).

## Contents (one hop)
### Subdirectories
- [x] `src/` - import/export logic on top of `reth-era` and `reth-storage-api` providers.
- [x] `tests/` - integration tests for genesis-only export and import->export roundtrips.

### Files
- `Cargo.toml` - crate manifest (depends on `reth-era`, `reth-era-downloader`, storage/provider APIs).
  - **Key items**: `description = "Utilities to store and fetch history data with storage-api"`

## Key APIs (no snippets)
- **Import**: `import()`, `process()`, `process_iter()`, `build_index()`, `save_stage_checkpoints()`
- **Export**: `export()`, `ExportConfig`

## Relationships
- **Used by**: CLI era import/export commands and other tooling that integrates ERA history files with reth's storage layout.
