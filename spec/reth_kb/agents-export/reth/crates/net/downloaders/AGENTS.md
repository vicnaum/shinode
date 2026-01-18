# downloaders

## Purpose
`reth-downloaders` crate: implements block body and header downloaders (network-based) plus optional file-based import clients and shared downloader metrics.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core downloaders, file clients/codecs, metrics, and test utilities.

### Files
- `Cargo.toml` - Crate manifest and feature flags for file-based clients and test utilities.
  - **Key items**: features `file-client`, `test-utils`; deps `reth-network-p2p`, `reth-consensus`, `reth-config`, `reth-tasks`, `tokio`, `async-compression`

## Key APIs (no snippets)
- `BodiesDownloader` / `BodiesDownloaderBuilder`
- `ReverseHeadersDownloader` / `ReverseHeadersDownloaderBuilder`
- `FileClient`, `ChunkedFileReader`, `ReceiptFileClient`

## Relationships
- **Depends on**: `reth-network-p2p` for download traits and network clients; `reth-consensus` for validation; `reth-storage-api` for header access; `reth-metrics` for instrumentation.
- **Used by**: `reth-stages` and node builder/CLI import flows to drive header/body sync and file imports.
