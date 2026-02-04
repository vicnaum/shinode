# test_utils

## Purpose
Test-only helpers for `reth-downloaders`: generators for mock headers/bodies (including optional file-backed fixtures) and a `BodiesClient` stub for downloader tests.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Test utilities entrypoint: exposes `TestBodiesClient`, provides body/header generators, and (when `file-client` is enabled) writes generated bodies into a temporary file using the downloader file codec.
- **Key items**: `TestBodiesClient`, `TEST_SCOPE`, `generate_bodies()`, `generate_bodies_file()`
- **Interactions**: Uses `bodies::test_utils::create_raw_bodies()` and `file_codec::BlockFileCodec` when `file-client` is enabled.
- **Knobs / invariants**: `generate_bodies_file()` is feature-gated (`file-client`) and rewinds the temp file before returning.

### `bodies_client.rs`
- **Role**: In-memory `BodiesClient` implementation for tests, supporting delayed responses, max batch sizing, and optional empty responses to exercise downloader behavior.
- **Key items**: `TestBodiesClient`, `with_bodies()`, `with_should_delay()`, `with_empty_responses()`, `with_max_batch_size()`, `times_requested()`, `should_respond_empty()`
- **Interactions**: Implements `DownloadClient` and `BodiesClient` for use by `bodies` and `request` tests.
- **Knobs / invariants**: `empty_response_mod` triggers empty responses every N requests; `times_requested` counts requests for assertions.

## End-to-end flow (high level)
- Use `generate_bodies()` to create a header list and body map for a block range.
- Build a `TestBodiesClient` with `with_bodies()` and optional behaviors (delay, max batch, empty responses).
- Pass the client into `BodiesDownloader`/`BodiesRequestFuture` tests to assert ordering and retry behavior.
- If file-client tests are enabled, call `generate_bodies_file()` to create a temp file encoded with `BlockFileCodec`.

## Key APIs (no snippets)
- `TestBodiesClient`
- `generate_bodies()`, `generate_bodies_file()`
