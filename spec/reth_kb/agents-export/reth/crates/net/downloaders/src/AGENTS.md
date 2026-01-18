# src

## Purpose
Core implementation of `reth-downloaders`: bodies/headers downloaders, file-based block/receipt clients for import workflows, shared metrics, and test utilities.

## Contents (one hop)
### Subdirectories
- [x] `bodies/` - Concurrent block-body downloader with request queue/futures, buffering/reordering, task wrapper, and noop/test helpers (`BodiesDownloader`, `TaskDownloader`).
- [x] `headers/` - Reverse (tip-to-head) header downloader with concurrent requests, validation/buffering, task wrapper, and noop/test helpers (`ReverseHeadersDownloader`).
- [x] `test_utils/` - Test helpers and in-memory `BodiesClient` for downloader tests; optional file-backed body fixtures.

### Files
- `lib.rs` - Crate entrypoint wiring modules and feature flags; re-exports file-client helpers when enabled.
  - **Key items**: feature flags `test-utils`, `file-client`; modules `bodies`, `headers`, `metrics`, `file_client`, `receipt_file_client`, `file_codec`, `test_utils`; re-exports `DecodedFileChunk`, `FileClientError`
- `metrics.rs` - Metrics types for body and header downloaders plus per-response gauges.
  - **Key items**: `BodyDownloaderMetrics`, `HeaderDownloaderMetrics`, `ResponseMetrics`, `increment_errors()`
  - **Interactions**: Used by `bodies/` and `headers/` to account for in-flight requests, buffering, and error categories.
- `file_client.rs` - File-based block client and chunked reader for RLP-encoded block files, with consensus validation and gzip support.
  - **Key items**: `FileClient`, `FileClientError`, `ChunkedFileReader`, `DEFAULT_BYTE_LEN_CHUNK_CHAIN_FILE`, `FromReader`, `DecodedFileChunk`
  - **Interactions**: Uses `BlockFileCodec` (`file_codec.rs`) for RLP framing; implements `HeadersClient`/`BodiesClient` for downloaders; bridges to `FromReceiptReader` (`receipt_file_client.rs`) for receipt imports.
- `file_codec.rs` - RLP codec for block files used by `FileClient` and test utilities.
  - **Key items**: `BlockFileCodec`
  - **Knobs / invariants**: Requires `FramedRead::with_capacity` to cover a full block payload to avoid `InputTooShort`.
- `receipt_file_client.rs` - File client for sequential receipt streams with block-number association, used by chunked import flows.
  - **Key items**: `ReceiptDecoder`, `ReceiptFileClient`, `FromReceiptReader`, `ReceiptWithBlockNumber`
  - **Interactions**: Consumed via `ChunkedFileReader::next_receipts_chunk()` in file import commands.

## Key APIs (no snippets)
- **Downloaders**: `BodiesDownloaderBuilder`, `ReverseHeadersDownloaderBuilder`
- **File import**: `FileClient`, `ChunkedFileReader`, `ReceiptFileClient`, `DecodedFileChunk`
- **Metrics**: `BodyDownloaderMetrics`, `HeaderDownloaderMetrics`, `ResponseMetrics`

## Relationships
- **Depends on**: `reth-network-p2p` (download traits/clients), `reth-consensus` (header/body validation), `reth-metrics` (metrics), `tokio`/`tokio-util` (async I/O + framing).
- **Used by**: `reth-stages` headers/bodies stages, `reth-node` builder setup, CLI import commands (`reth/crates/cli/commands/src/import_core.rs`, `reth/crates/cli/commands/src/stage/run.rs`), Optimism CLI import tools (`reth/crates/optimism/cli/src/commands/import*.rs`).
