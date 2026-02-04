# src

## Purpose
Implements `reth-era` core logic for e2store-based history files: the shared `.e2s` record layer, `.era` (consensus/CL) and `.era1` (execution/EL) file models with streaming readers/writers, and helper utilities for compression/decompression and naming.

## Contents (one hop)
### Subdirectories
- [x] `common/` - shared decoding helpers and generic file-format traits/naming utilities.
- [x] `e2s/` - core e2store TLV records, readers/writers, and error types (foundation for `.era`/`.era1`).
- [x] `era/` - `.era` consensus-layer file support (beacon block/state compression wrappers, group/index/id types, reader/writer).
- [x] `era1/` - `.era1` execution-layer file support (compressed header/body/receipts, accumulator/index, reader/writer).

### Files
- `lib.rs` - crate root documentation and module wiring.
  - **Key items**: `common`, `e2s`, `era`, `era1`
- `test_utils.rs` - test-only helpers for constructing sample compressed blocks/states and receipts for unit/integration tests.
  - **Key items**: `create_header()`, `create_sample_block()`, `create_test_block_with_compressed_data()`, `create_beacon_block()`, `create_beacon_state()`

## Key APIs (no snippets)
- **E2Store**: `E2StoreReader`, `E2StoreWriter`, `Entry`, `E2sError`
- **Era (CL)**: `EraFile`, `EraReader`, `EraWriter`, `CompressedSignedBeaconBlock`, `CompressedBeaconState`
- **Era1 (EL)**: `Era1File`, `Era1Reader`, `Era1Writer`, `BlockTuple`, `CompressedHeader`, `CompressedBody`, `CompressedReceipts`

## Relationships
- **Used by**: downloader/utility crates that fetch and process era/era1 history files for history expiry workflows.
