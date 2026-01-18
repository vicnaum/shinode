# headers

## Purpose
Implements block header downloader algorithms for `reth-downloaders`: a reverse (tip-to-head) concurrent headers downloader, plus a task-driven wrapper and a noop implementation for pipeline wiring.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring for header downloaders: exports the reverse downloader, a noop implementation, a task-driven wrapper, and optional test helpers.
- **Key items**: `reverse_headers`, `noop`, `task`, `test_utils` (cfg)

### `reverse_headers.rs`
- **Role**: Main concurrent reverse headers downloader that fills the gap between local head and sync target by issuing multiple in-flight requests, buffering out-of-order responses, and validating headers before emitting sorted batches.
- **Key items**: `ReverseHeadersDownloader`, `ReverseHeadersDownloaderBuilder`, `SyncTargetBlock`, `REQUESTS_PER_PEER_MULTIPLIER`, `set_batch_size()`, `update_local_head()`, `update_sync_target()`
- **Interactions**: Uses `HeaderValidator` to validate parent links; requests via `HeadersClient`; records `HeaderDownloaderMetrics`; can be driven as a `TaskDownloader` (`task.rs`).
- **Knobs / invariants**: Downloads in reverse (falling block numbers); uses `request_limit`, `stream_batch_size`, `min/max_concurrent_requests`, `max_buffered_responses`; retries and penalizes peers on invalid responses; clears/reset state on detached head or target changes.

### `task.rs`
- **Role**: Task wrapper that runs any `HeaderDownloader` on a spawned task and forwards results over channels, allowing sync target/head updates and batch-size changes asynchronously.
- **Key items**: `TaskDownloader`, `HEADERS_TASK_BUFFER_SIZE`, `spawn()`, `spawn_with()`, `DownloaderUpdates`
- **Interactions**: `ReverseHeadersDownloader::into_task*()` returns this wrapper; forwards update commands to the underlying downloader and streams validated batches.

### `noop.rs`
- **Role**: No-op `HeaderDownloader` implementation used for unwind-only pipelines or configurations that must satisfy a downloader interface without doing work.
- **Key items**: `NoopHeaderDownloader`

### `test_utils.rs`
- **Role**: Small test helper to build a child header from a parent.
- **Key items**: `child_header()`

## End-to-end flow (high level)
- Construct a `ReverseHeadersDownloader` via `builder()` or `ReverseHeadersDownloaderBuilder::new(HeadersConfig)`.
- Set boundaries by calling `update_local_head()` and `update_sync_target()` (or via task wrapper updates).
- The downloader requests the sync target header first, validates it, and initializes request trackers.
- It issues reverse `GetBlockHeaders` requests concurrently, keeping per-peer request capacity based on connected peers.
- Responses are sorted, validated against parent links and consensus rules, and buffered if they arrive out of order.
- Validated headers are queued in descending order and emitted in batches (`stream_batch_size`) as a `Stream`.
- On bad responses, the peer is penalized and the request is retried at higher priority; on detached heads, state is reset to await a new target.

## Key APIs (no snippets)
- **Downloaders**: `ReverseHeadersDownloader`, `ReverseHeadersDownloaderBuilder`, `TaskDownloader`, `NoopHeaderDownloader`
- **Sync target tracking**: `SyncTargetBlock`
