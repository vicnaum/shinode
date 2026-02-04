# era-downloader

## Purpose
`reth-era-downloader` crate: provides an async streaming interface for discovering and downloading ERA history files (`.era1` and `.era`) from remote hosts (or reading `.era1` from local dirs), with configurable concurrency and on-disk retention limits.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `EraClient` (index/checksum + downloads), `EraStream` (scheduling), and local `read_dir()` ingestion.
- [x] `tests/` - integration tests plus embedded fixtures for stubbed HTTP/local-dir scenarios.

### Files
- `Cargo.toml` - crate manifest (reqwest streaming + sha2 checksum verification for `.era1`).
  - **Key items**: `description = "An asynchronous stream interface for downloading ERA1 files"`

## Key APIs (no snippets)
- **Remote**: `EraClient`, `EraStream`, `EraStreamConfig`, `EraMeta`
- **Local**: `read_dir()`

## Relationships
- **Used by**: import tooling (e.g. CLI `import-era`) that streams history files for ingestion into storage/pipeline.
