# headers

## Purpose
Shared header download abstractions for p2p: request types, client trait, downloader interface, and validation/error helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for header client, downloader, and error types.
- **Key items**: `client`, `downloader`, `error`

### `client.rs`
- **Role**: Defines `HeadersClient` and request types for fetching block headers with priority.
- **Key items**: `HeadersRequest`, `HeadersClient`, `HeadersFut`, `SingleHeaderRequest`, `HeadersDirection`

### `downloader.rs`
- **Role**: Header downloader trait and sync target helpers, plus validation utility.
- **Key items**: `HeaderDownloader`, `SyncTarget`, `HeaderSyncGap`, `validate_header_download()`
- **Interactions**: Uses `HeaderValidator` to validate parent linkage and standalone header rules.

### `error.rs`
- **Role**: Downloader error type for header validation failures (e.g., detached head).
- **Key items**: `HeadersDownloaderError`, `HeadersDownloaderResult`

## End-to-end flow (high level)
- A `HeadersClient` issues `HeadersRequest` with a direction and limit.
- A `HeaderDownloader` streams validated `SealedHeader` batches toward a `SyncTarget`.

## Key APIs (no snippets)
- `HeadersClient`, `HeaderDownloader`, `HeadersRequest`, `SyncTarget`
