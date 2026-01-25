# p2p

## Purpose
Devp2p networking and fetch helpers built on `reth-network`. Maintains a pool of active peers and
provides batched requests for headers, bodies, and receipts (with partial-response handling) to
feed the historical sync pipeline.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Mainnet peer connectivity, peer pool tracking, peer cache integration, and chunked fetch utilities.
  - **Key items**: `connect_mainnet_peers()`, `PeerPool`, `NetworkSession`, `fetch_payloads_for_peer()`, `discover_head_p2p()`, `FetchStageStats`

## Key APIs (no snippets)
- **Types / Traits**: `PeerSelector`, `RoundRobinPeerSelector`, `NetworkPeer`, `NetworkSession`, `PeerPool`, `MultiPeerBlockPayloadSource`, `P2pLimits`
- **Functions**: `connect_mainnet_peers()`, `p2p_limits()`, `request_headers_batch()`, `discover_head_p2p()`, `fetch_payloads_for_peer()`, `request_receipt_counts()`

## Relationships
- **Depends on**: `reth_network` / `reth_network_api` (peer sessions + requests), `reth_eth_wire_types` (message structs), `tokio` (timeouts/tasks).
- **Uses**: `node/src/storage` for persisted peer cache (`StoredPeer`, `Storage::load_peers()` / `flush_peer_cache()`).
- **Used by**: `node/src/sync/historical` for both probe (header+receipt counts) and ingest (full payload fetch).

## Files (detailed)

### `mod.rs`
- **Role**: Starts the network, tracks active peer sessions, and exposes safe, bounded fetch helpers that tolerate partial responses and flaky peers.
- **Key items**: `REQUEST_TIMEOUT`, `MAX_HEADERS_PER_REQUEST`, `connect_mainnet_peers()`, `PeerCacheBuffer`, `HeadersChunkedResponse`, `ChunkedResponse<T>`
- **Interactions**: `fetch_payloads_for_peer()` composes `request_headers_chunked_with_stats()` + `request_bodies_chunked_partial_with_stats()` + `request_receipts_chunked_partial_with_stats()` into `sync::BlockPayload` outputs.
- **Knobs / invariants**:
  - Header probes (`discover_head_p2p()`) only advance the head when headers are actually retrievable above a baseline (does not trust session status head).
  - Chunked request helpers stop early on partial peer responses and report missing blocks upstream.
  - Peer cache is TTL'd (`PEER_CACHE_TTL_DAYS`) and capped (`PEER_CACHE_MAX`) before seeding static peers.

## End-to-end flow (high level)
- Build a `reth-network` mainnet config and start networking (`connect_mainnet_peers()`).
- Watch `NetworkEvent` streams to add/remove peers and to probe peer heads.
- Optionally seed and persist peer addresses via the storage-backed peer cache.
- For a requested range, fetch headers (chunked) then concurrently fetch bodies and receipts.
- Align responses by block number and return `(payloads, missing_blocks, fetch_stats)` to the caller.

