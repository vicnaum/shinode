# bodies

## Purpose
Shared traits and types for block body downloads in the p2p layer, including client abstractions, downloader interface, and response wrapper.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for body download traits and response types.
- **Key items**: `client`, `downloader`, `response`

### `client.rs`
- **Role**: Defines the `BodiesClient` trait and helpers for fetching bodies with optional priority/range hints.
- **Key items**: `BodiesClient`, `BodiesFut`, `SingleBodyRequest`, `get_block_bodies_with_priority_and_range_hint()`
- **Interactions**: Extends `DownloadClient`; used by downloaders in `reth-downloaders` and full-block helpers.

### `downloader.rs`
- **Role**: Downloader trait for streaming body batches derived from headers.
- **Key items**: `BodyDownloader`, `BodyDownloaderResult`, `set_download_range()`

### `response.rs`
- **Role**: Wrapper for body responses, distinguishing empty vs full blocks with helpers.
- **Key items**: `BlockResponse`, `block_number()`, `difficulty()`, `into_body()`, `body()`
- **Interactions**: Implements `InMemorySize` for buffering/memory accounting.

## End-to-end flow (high level)
- A `BodiesClient` issues `GetBlockBodies` requests with optional priority and range hints.
- A `BodyDownloader` consumes headers, issues body requests via the client, and yields `BlockResponse` batches.

## Key APIs (no snippets)
- `BodiesClient`, `BodyDownloader`, `BlockResponse`
