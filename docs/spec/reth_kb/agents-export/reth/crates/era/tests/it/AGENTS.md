# it

## Purpose
Integration test harness for `reth-era`: downloads a small curated set of `.era`/`.era1` files from public hosts and validates that the library can read, decompress/decode, and roundtrip them correctly.

## Contents (one hop)
### Subdirectories
- [x] `era/` - `.era` file integration tests (genesis + roundtrip).
- [x] `era1/` - `.era1` file integration tests (genesis + roundtrip).

### Files
- `main.rs` - shared test harness: fixture URL lists, caching downloader, and helpers for opening files with `EraReader`/`Era1Reader`.
  - **Key items**: `EraTestDownloader`, `EraFileType`, `EraReader`, `Era1Reader`

## Relationships
- **Uses**: `reth-era-downloader` to fetch file lists and download specific files to a temp dir for testing.
