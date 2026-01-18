# src

## Purpose
P2P protocol abstractions for downloaders: request clients, error types, full-block fetch helpers, SNAP traits, and sync state interfaces.

## Contents (one hop)
### Subdirectories
- [x] `bodies/` - Body download client traits, downloader interface, and response wrapper.
- [x] `headers/` - Header request/client traits, downloader interface, and validation helpers.
- [x] `snap/` - SNAP sync client trait and response variants.
- [x] `test_utils/` - Mock clients and downloaders for tests.

### Files
- `download.rs` - Base trait for download clients used across bodies/headers/SNAP.
  - **Key items**: `DownloadClient`
- `either.rs` - `Either` adapter implementing `DownloadClient`, `BodiesClient`, and `HeadersClient` for two backends.
  - **Key items**: `Either` impls for `DownloadClient`/`BodiesClient`/`HeadersClient`
- `error.rs` - Common request/response error types and validation helpers.
  - **Key items**: `RequestError`, `DownloadError`, `EthResponseValidator`, `RequestResult`, `PeerRequestResult`
- `full_block.rs` - Full-block fetcher built from header/body clients with validation.
  - **Key items**: `FullBlockClient`, `FetchFullBlockFuture`, `FetchFullBlockRangeFuture`, `NoopFullBlockClient`
- `lib.rs` - Crate entrypoint and public re-exports.
  - **Key items**: modules `bodies`, `headers`, `snap`, `sync`, `full_block`; trait `BlockClient`
- `priority.rs` - Request priority enum used by clients and downloaders.
  - **Key items**: `Priority`
- `sync.rs` - Sync state traits and a noop updater.
  - **Key items**: `SyncStateProvider`, `NetworkSyncUpdater`, `SyncState`, `NoopSyncStateUpdater`

## Key APIs (no snippets)
- `BodiesClient`, `HeadersClient`, `SnapClient`
- `DownloadClient`, `RequestError`, `DownloadError`
- `FullBlockClient`, `BlockClient`, `NetworkSyncUpdater`
