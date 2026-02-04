# fetch

## Purpose
Implements network fetch plumbing: a stateful request coordinator (`StateFetcher`) that selects peers and tracks in-flight eth requests, plus a `FetchClient` that exposes `HeadersClient`/`BodiesClient` APIs backed by the coordinator.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core fetcher state machine that queues header/body requests, selects the best peer, dispatches `GetBlockHeaders`/`GetBlockBodies`, and processes responses with reputation outcomes.
- **Key items**: `StateFetcher`, `DownloadRequest`, `FetchAction`, `BlockResponseOutcome`, `Peer`, `PeerState`, `BestPeerRequirements`
- **Interactions**: Emits `BlockRequest` messages to sessions (`crate::message`); uses `PeersHandle` for reputation changes; relies on `EthResponseValidator` to detect bad responses and returns `ReputationChangeKind`.
- **Knobs / invariants**: Prioritizes low-latency and range-capable peers; tracks per-peer range info and response quality; re-queues high-priority requests ahead of normal queue.

### `client.rs`
- **Role**: Front-end `FetchClient` implementing `HeadersClient` and `BodiesClient` by sending requests over a channel to the `StateFetcher` and flattening responses.
- **Key items**: `FetchClient`, `HeadersClientFuture`, `get_headers_with_priority()`, `get_block_bodies_with_priority_and_range_hint()`
- **Interactions**: Uses `FlattenedResponse` for oneshot response handling; delegates reputation changes to `PeersHandle`; exposes `DownloadClient` and `BlockClient` for downloader integration.

## End-to-end flow (high level)
- Construct a `StateFetcher` with a `PeersHandle` and active-peer counter, then create a `FetchClient` via `StateFetcher::client()`.
- Call `FetchClient` methods to queue `DownloadRequest`s for headers or bodies; responses are delivered through oneshot channels.
- `StateFetcher::poll()` drains queued requests, selects the next best idle peer (range/latency aware), and emits a `FetchAction::BlockRequest`.
- When responses arrive, `StateFetcher` validates for likely bad responses, updates per-peer state, and optionally returns `BlockResponseOutcome::BadResponse`.
- Successful peers can receive immediate follow-up requests; failed peers are penalized via `ReputationChangeKind`.

## Key APIs (no snippets)
- `StateFetcher`
- `FetchClient`
- `DownloadRequest`, `FetchAction`, `BlockResponseOutcome`
