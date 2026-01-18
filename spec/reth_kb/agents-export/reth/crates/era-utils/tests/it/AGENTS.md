# it

## Purpose
Integration tests for `reth-era-utils`: validate importing `.era1` history into a fresh provider factory and exporting history back into `.era1` files (including roundtrip expectations and file naming/chunking).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `main.rs` - test harness: defines an `HttpClient` wrapper that fakes the remote index to expose only a single known era1 file while delegating actual downloads to a real `reqwest::Client`.
  - **Key items**: `ClientWithFakeIndex`, `ITHACA_ERA_INDEX_URL`
- `genesis.rs` - verifies exporting when the DB contains only genesis (exports a single `.era1` file and asserts basic properties).
  - **Key items**: `test_export_with_genesis_only`, `ExportConfig`, `export()`
- `history.rs` - imports from a remote era1 stream into a fresh state and tests export-after-import roundtrips (chunking and filename format assertions).
  - **Key items**: `test_history_imports_from_fresh_state_successfully`, `test_roundtrip_export_after_import`, `import()`, `export()`

## Relationships
- **Exercises**: `reth_era_downloader::EraStream` -> `reth_era_utils::import()` -> provider/static-files; then `reth_era_utils::export()` -> `.era1` files on disk.
