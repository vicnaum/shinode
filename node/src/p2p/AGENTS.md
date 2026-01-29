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
  - **Key items**: `connect_mainnet_peers()`, `PeerPool`, `NetworkSession`, `fetch_payloads_for_peer()`, `discover_head_p2p()`, `FetchStageStats`, `P2pStats`

## Key APIs (no snippets)
- **Types**: `NetworkPeer`, `NetworkSession`, `PeerPool`, `P2pStats`, `P2pLimits`, `PayloadFetchOutcome`, `HeadersChunkedResponse`, `ChunkedResponse<T>`, `FetchStageStats`, `HeaderCountMismatch`
- **Functions**: `connect_mainnet_peers()`, `p2p_limits()`, `request_headers_batch()`, `discover_head_p2p()`, `fetch_payloads_for_peer()`, `request_headers_chunked_with_stats()`, `request_headers_chunked_strict()`, `PeerPool::get_peer_head()`, `PeerPool::update_peer_head()`, `re_probe_peer_head()`, `request_receipts()`

## Relationships
- **Depends on**: `reth_network` / `reth_network_api` (peer sessions + requests), `reth_eth_wire_types` (message structs), `tokio` (timeouts/tasks).
- **Uses**: `node/src/storage` for persisted peer cache (`StoredPeer`, `Storage::load_peers()` / `flush_peer_cache()`).
- **Used by**: `node/src/sync/historical` for ingest (full payload fetch).

## Files (detailed)

### `mod.rs`
- **Role**: Starts the network, tracks active peer sessions, and exposes safe, bounded fetch helpers that tolerate partial responses and flaky peers.
- **Key constants**: `REQUEST_TIMEOUT` (4s), `MAX_HEADERS_PER_REQUEST` (1024), `MAX_OUTBOUND` (400), `MAX_INBOUND` (200), `MAX_CONCURRENT_DIALS` (200), `PEER_CACHE_TTL_DAYS` (7), `PEER_CACHE_MAX` (5000), `PEER_START_WARMUP_SECS` (2)
- **Key types**:
  - `P2pStats`: Atomic counters for discovery and session metrics (`discovered_count`, `genesis_mismatch_count`, `sessions_established`, `sessions_closed`)
  - `P2pLimits`: Configuration struct with all limits, exportable via `p2p_limits()`
  - `NetworkPeer`: Active peer with `peer_id`, `eth_version`, `messages` (request sender), `head_number`
  - `PeerPool`: Thread-safe peer collection with `add_peer()`, `remove_peer()`, `get_peer_head()`, `update_peer_head()`, `snapshot()`
  - `PeerCacheBuffer` / `PeerCacheHandle`: In-memory buffer for peer discovery/session events, flushed to storage on shutdown
- **Peer management**:
  - Head probing on session establishment via 24-slot semaphore (limits concurrency)
  - DHT discovery watcher logs progress every 30s
  - `re_probe_peer_head()`: Re-probe a single peer's head to confirm it can serve blocks
- **Fetch helpers** (tolerant of partial responses):
  - `request_headers_chunked_with_stats()` / `request_headers_chunked_strict()`
  - `request_bodies_chunked_partial_with_stats()` / `request_receipts_chunked_partial_with_stats()`
  - `fetch_payloads_for_peer()`: End-to-end fetch composing headers + concurrent bodies/receipts
- **Protocol versions**: eth/68 (`GetReceipts`), eth/69 (`GetReceipts69`), eth/70 (`GetReceipts70` with `last_block_incomplete` flag)
- **Interactions**: `fetch_payloads_for_peer()` composes chunked requests into `sync::BlockPayload` outputs.
- **Knobs / invariants**:
  - Header probes (`discover_head_p2p()`) only advance the head when headers are actually retrievable above a baseline (does not trust session status head).
  - Chunked request helpers stop early on partial peer responses and report missing blocks upstream.
  - Peer cache is TTL'd and capped before seeding static peers.
  - `get_peer_head()` allows the sync pipeline to refresh stale peer `head_number` clones from re-probe updates written to the pool.
  - Genesis mismatch peers are rejected during session establishment.

## End-to-end flow (high level)
- Build a `reth-network` mainnet config and start networking (`connect_mainnet_peers()`).
- Watch `NetworkEvent` streams to add/remove peers and to probe peer heads.
- Optionally seed and persist peer addresses via the storage-backed peer cache.
- For a requested range, fetch headers (chunked) then concurrently fetch bodies and receipts.
- Align responses by block number and return `(payloads, missing_blocks, fetch_stats)` to the caller.
