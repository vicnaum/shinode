# src

## Purpose
Implements `reth-era-utils`: higher-level utilities for importing `.era1` history into reth storage (via `storage-api`/providers/static files) and exporting stored block history back into `.era1` files.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: wires modules and re-exports the import/export helper APIs.
- **Key items**: `ExportConfig`, `export()`, `import()`, `process()`, `process_iter()`, `build_index()`, `save_stage_checkpoints()`
- **Interactions**: re-exports are consumed by CLI import/export paths and tests.
- **Knobs / invariants**: none (mostly surface consolidation).

#### `export.rs`
- **Role**: Exports block history from a provider into `.era1` files on disk using `Era1Writer`, chunking by configurable "blocks per file" and building accumulator + block index records.
- **Key items**: `ExportConfig`, `ExportConfig::validate()`, `export()`, `determine_export_range()`, `compress_block_data()`, `MAX_BLOCKS_PER_ERA1`
- **Interactions**: reads headers/bodies/receipts via `reth_storage_api` provider traits; uses `reth_era` compressed record types and `Era1Id` naming.
- **Knobs / invariants**: `max_blocks_per_file` must be \(1..=8192\); filename includes optional era count when using custom blocks-per-file.

#### `history.rs`
- **Role**: Imports history from downloaded `.era1` files into storage: opens/iterates the era1 reader, decodes headers/bodies, appends into static files + DB tables, and updates stage checkpoints/index tables.
- **Key items**: `import()`, `process()`, `ProcessIter`, `open()`, `decode()`, `process_iter()`, `build_index()`, `save_stage_checkpoints()`
- **Interactions**: consumes an `EraMeta` stream (from `reth_era_downloader`); appends headers into `StaticFileSegment::Headers` and bodies into DB; uses an ETL `Collector` to build hash->number indexes.
- **Knobs / invariants**: import updates stage checkpoints for `Headers`/`Bodies` to keep pipeline stages consistent with imported data.

## Key APIs (no snippets)
- **Import**: `import()`, `process()`, `process_iter()`, `ProcessIter`
- **Export**: `export()`, `ExportConfig`

## Relationships
- **Builds on**: `reth-era` (file formats), `reth-era-downloader` (file streaming), and `reth-storage-api`/providers (persisting history into reth storage).
