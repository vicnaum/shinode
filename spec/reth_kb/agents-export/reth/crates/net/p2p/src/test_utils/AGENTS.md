# test_utils

## Purpose
Test helpers for p2p interfaces: mock clients for headers/bodies and a full-block in-memory client.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for test helpers.
- **Key items**: `bodies`, `headers`, `full_block`

### `bodies.rs`
- **Role**: Simple bodies client that returns responder-provided results.
- **Key items**: `TestBodiesClient`
- **Interactions**: Implements `DownloadClient` and `BodiesClient` for unit tests.

### `headers.rs`
- **Role**: Test header downloader and mock header client with queued responses and error injection.
- **Key items**: `TestHeaderDownloader`, `TestHeadersClient`
- **Interactions**: Implements `HeaderDownloader`, `HeadersClient`, and uses `RequestError`/`DownloadError` for failure paths.

### `full_block.rs`
- **Role**: In-memory full-block client storing headers and bodies with a soft body response limit.
- **Key items**: `TestFullBlockClient`
- **Interactions**: Implements `HeadersClient`, `BodiesClient`, and `BlockClient`.

## End-to-end flow (high level)
- Configure a test client (headers/bodies/full-block) with in-memory data or custom responder.
- Use the client in downloader or full-block tests to validate request/response behavior.

## Key APIs (no snippets)
- `TestBodiesClient`, `TestHeadersClient`, `TestHeaderDownloader`, `TestFullBlockClient`
