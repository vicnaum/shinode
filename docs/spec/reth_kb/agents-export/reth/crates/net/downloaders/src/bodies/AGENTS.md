# bodies

## Purpose
Implements block body downloader algorithms for `reth-downloaders`: a concurrent, range-based bodies downloader (`BodiesDownloader`) plus helpers for scheduling/validating requests, buffering/reordering responses, and optionally driving the downloader on a separate task.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for body downloaders: exports the main concurrent downloader, a noop implementation, a task-driven wrapper, and optional test helpers; keeps request queue/future internal.
- **Key items**: `bodies`, `noop`, `task`, `queue` (private), `request` (private), `test_utils` (cfg)

### `bodies.rs`
- **Role**: Main concurrent bodies downloader implementation: batches header-derived requests, issues multiple in-flight peer requests, buffers/reorders responses to emit bodies in block-number order, and applies memory/backpressure limits while streaming results.
- **Key items**: `BodiesDownloader`, `BodiesDownloaderBuilder`, `set_download_range()`, `into_task()`, `into_task_with()`
- **Interactions**: Uses `BodiesRequestQueue` (`queue.rs`) + `BodiesRequestFuture` (`request.rs`) for request concurrency; records `BodyDownloaderMetrics` (`crate::metrics`); can be driven as a `TaskDownloader` (`task.rs`).
- **Knobs / invariants**: Respects `request_limit`, `stream_batch_size`, `concurrent_requests_range`, `max_buffered_blocks_size_bytes`; emits in-order batches even if requests complete out-of-order; clears state on fatal errors and on non-consecutive range changes.

### `request.rs`
- **Role**: Request/validation future for a single header batch: repeatedly requests missing bodies until fulfilled, filters empty headers, validates bodies against consensus, and penalizes peers on malformed responses before retrying.
- **Key items**: `BodiesRequestFuture`, `with_headers()`, `on_block_response()`, `try_buffer_blocks()`, `submit_request()`
- **Interactions**: Driven by `BodiesRequestQueue` (`queue.rs`); uses `BodiesClient` for network requests and `Consensus::validate_block_pre_execution`; updates `BodyDownloaderMetrics` + per-response `ResponseMetrics`.
- **Knobs / invariants**: Assumes peers return bodies in request order; empty/overlong responses and validation failures trigger error accounting + peer penalty + high-priority retry.

### `queue.rs`
- **Role**: In-flight request queue wrapper over `FuturesUnordered` that tracks the last requested block number and exposes a `Stream` of request results.
- **Key items**: `BodiesRequestQueue`, `push_new_request()`, `last_requested_block_number`, `clear()`
- **Interactions**: Owned/polled by `BodiesDownloader` (`bodies.rs`); carries `BodyDownloaderMetrics` for queue-level accounting.

### `task.rs`
- **Role**: Task wrapper that runs any `BodyDownloader` on a spawned task and forwards results over channels, allowing download ranges to be updated asynchronously.
- **Key items**: `TaskDownloader`, `BODIES_TASK_BUFFER_SIZE`, `spawn()`, `spawn_with()`
- **Interactions**: `BodiesDownloader::into_task*()` returns this wrapper; uses `reth_tasks::TaskSpawner` + Tokio channels/streams to bridge caller <-> background downloader.

### `noop.rs`
- **Role**: No-op `BodyDownloader` implementation used for unwind-only pipelines or configurations that must satisfy a downloader interface without doing work.
- **Key items**: `NoopBodiesDownloader`

### `test_utils.rs`
- **Role**: Test-only helpers for composing expected body responses and inserting headers into a test provider/static-file writer.
- **Key items**: `zip_blocks()`, `create_raw_bodies()`, `insert_headers()`
- **Interactions**: Used by tests in `bodies.rs`, `request.rs`, `task.rs` and crate-level test utilities.

## End-to-end flow (high level)
- Construct a downloader via `BodiesDownloaderBuilder` (often from `BodiesConfig`) with a `BodiesClient`, `Consensus`, and `HeaderProvider`.
- Set an inclusive block range with `set_download_range()`, which may append consecutive ranges or reset internal state for non-consecutive changes.
- Query headers from the provider and form variable-length batches based on non-empty count and `stream_batch_size`.
- For each header batch, push a `BodiesRequestFuture` into `BodiesRequestQueue`, keeping multiple requests in flight up to a peer-aware concurrency limit.
- Each `BodiesRequestFuture` requests bodies by header hashes, filters empty headers, validates bodies via `Consensus`, and retries/penalizes peers on bad responses.
- Completed responses are buffered and reordered so the stream only yields bodies in block-number order.
- Once enough contiguous bodies are queued, yield a batch (up to `stream_batch_size`) and apply backpressure using queued-count and buffered-bytes limits.
- Optionally, drive the downloader on a background task via `TaskDownloader`, sending range updates over a channel and receiving streamed results.

## Key APIs (no snippets)
- **Downloaders**: `BodiesDownloader`, `BodiesDownloaderBuilder`, `TaskDownloader`, `NoopBodiesDownloader`
- **Internal plumbing**: `BodiesRequestFuture`, `BodiesRequestQueue`
