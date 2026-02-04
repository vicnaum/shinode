# era

## Purpose
`reth-era` crate: core logic for e2store-based history files, implementing `.era` (consensus-layer) and `.era1` (execution-layer) formats with streaming readers/writers, compression wrappers, indices, and standardized file naming for history expiry workflows.

## Contents (one hop)
### Subdirectories
- [x] `src/` - e2store primitives plus `.era`/`.era1` models, types, and readers/writers.
- [x] `tests/` - integration tests that download sample `.era`/`.era1` files and verify parsing/roundtrips.

### Files
- `Cargo.toml` - crate manifest (SSZ + snappy + RLP dependencies; test-time downloader wiring).
  - **Key items**: `description = "e2store and era1 files core logic"`

## Key APIs (no snippets)
- **E2Store**: `E2StoreReader`, `E2StoreWriter`, `Entry`, `E2sError`
- **Era / Era1**: `EraFile`/`EraReader`/`EraWriter`, `Era1File`/`Era1Reader`/`Era1Writer`

## Relationships
- **Used by**: ERA download/import/export tooling to store and distribute pruned history data.
