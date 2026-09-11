# p2p

## Purpose
Devp2p networking and fetch helpers built on `reth-network`. Maintains a pool of active peers and
provides batched requests for headers, bodies, and receipts (with partial-response handling) to
feed the historical sync pipeline.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `chain_tests.rs` - Typed receipt/payload tests, chain identity and header-probe rejection, and opt-in 90-second live Base/Ethereum diagnostics.
- `mod.rs` - Chain-typed peer connectivity, peer pool tracking, peer cache integration, and chunked fetch utilities.
  - **Key items**: `connect_peers<C>()`, `PeerPool`, `NetworkSession`, `fetch_payloads_for_peer()`, `discover_head_p2p()`, `FetchStageStats`, `P2pStats`

## Key APIs (no snippets)
- **Types**: `NetworkPeer`, `NetworkSession`, `PeerPool`, `P2pStats`, `P2pLimits`, `PayloadFetchOutcome`, `HeadersChunkedResponse`, `ChunkedResponse<T>`, `FetchStageStats`, `HeaderCountMismatch`
- **Functions**: `connect_peers<C>()`, `p2p_limits()`, `request_headers_batch()`, `discover_head_p2p()`, `fetch_payloads_for_peer()`, `request_headers_chunked_with_stats()`, `request_headers_chunked_strict()`, `PeerPool::get_peer_head()`, `PeerPool::update_peer_head()`, `re_probe_peer_head()`, `request_receipts()`

## Relationships
- **Depends on**: `reth_network` / `reth_network_api` (peer sessions + requests), `reth_eth_wire_types` (message structs), `tokio` (timeouts/tasks).
- **Uses**: `node/src/storage` for persisted peer cache (`StoredPeer`, `Storage::load_peers()` / `flush_peer_cache()`).
- **Used by**: `node/src/sync/historical` for ingest (full payload fetch), `node/src/run/startup.rs` (initialization), `node/src/ui/progress.rs` (peer counts).
- **Data/control flow**:
  1. `connect_peers<C>()` initializes network, loads cached peers, spawns watchers
  2. `spawn_peer_watcher()` manages session lifecycle: validate genesis, create `NetworkPeer`, probe head
  3. `spawn_peer_discovery_watcher()` tracks DHT events, upserts to peer cache
  4. Fetch APIs (`fetch_payloads_for_peer`) request headers+bodies+receipts in parallel with 4s timeout
  5. `discover_head_p2p()` probes multiple peers to find best observed chain head
  6. Shutdown: `flush_peer_cache()` persists buffer to `peers.json`

## Files (detailed)

### `mod.rs`
- **Role**: Complete P2P subsystem with separate protocol and connectivity tests. Starts the network, tracks active peer sessions, and exposes safe, bounded fetch helpers that tolerate partial responses and flaky peers.
- **Key constants**: `REQUEST_TIMEOUT` (4s), `MAX_HEADERS_PER_REQUEST` (1024), `MAX_OUTBOUND` (400), `MAX_INBOUND` (200), `MAX_CONCURRENT_DIALS` (200), `PEER_CACHE_TTL_DAYS` (7), `PEER_CACHE_MAX` (5000), `PEER_START_WARMUP_SECS` (2), `PEER_REFILL_INTERVAL_MS` (500)
- **Key types**:
  - `P2pStats`: Atomic counters for discovery and session metrics (`discovered_count`, `genesis_mismatch_count`, `sessions_established`, `sessions_closed`)
  - `P2pLimits`: Configuration struct with all limits, exportable via `p2p_limits()`
  - `NetworkPeer`: Active peer with `peer_id`, `eth_version`, `messages` (request sender), `head_number`, advertised `earliest_block` / `latest_block`
  - `PeerPool`: Thread-safe peer collection with `add_peer()`, `remove_peer()`, `get_peer_head()`, `update_peer_head()`, `snapshot()`
  - `PeerCacheBuffer` / `PeerCacheHandle`: In-memory buffer for peer discovery/session events, flushed to storage on shutdown
- **Peer management**:
  - Head probing on session establishment via 24-slot semaphore (limits concurrency)
  - DHT discovery watcher logs progress at DEBUG level every 30s
  - `re_probe_peer_head()`: Re-probe a single peer's head to confirm it can serve blocks
- **Fetch helpers** (tolerant of partial responses):
  - `request_headers_chunked_with_stats()` / `request_headers_chunked_strict()`
  - `request_bodies_chunked_partial_with_stats()` / `request_receipts_chunked_partial_with_stats()`
  - `fetch_payloads_for_peer()`: End-to-end fetch composing headers + concurrent bodies/receipts
- **Protocol versions**: eth/68 (`GetReceipts`), eth/69 (`GetReceipts69`), eth/70 (`GetReceipts70` with `last_block_incomplete` flag)
- **Interactions**: `fetch_payloads_for_peer()` composes chunked requests into `sync::BlockPayload` outputs. `run/startup.rs` calls `connect_peers<C>()`. `sync/historical/fetch.rs` wraps `fetch_payloads_for_peer()`. `follow.rs` uses `discover_head_p2p()`. `reorg.rs` uses `request_headers_batch()`.
- **Knobs / invariants**:
  - Header probes (`discover_head_p2p()`) only advance the head when headers are actually retrievable above a baseline (does not trust session status head).
  - Chunked request helpers stop early on partial peer responses and report missing blocks upstream.
  - Peer cache is TTL'd and capped before seeding static peers.
  - `get_peer_head()` allows the sync pipeline to refresh stale peer `head_number` clones from re-probe updates written to the pool.
  - Genesis mismatch peers are rejected during session establishment.

## End-to-end flow (high level)

1. `connect_peers<C>()`: generate key, configure limits, start network, load cached peers, spawn watchers, wait for min peers
2. `spawn_peer_watcher()`: on session open, validate genesis, add to pool, probe head; on close, remove from pool
3. `fetch_payloads_for_peer()`: request chunked headers (auto-split >1024), parallel bodies+receipts, zip into `BlockPayload` vector
4. `discover_head_p2p()`: probe N peers from pool, request headers from baseline, track highest observed
5. Protocol-aware receipts: eth/68 (legacy), eth/69 (typed), eth/70 (with partial flag)
6. Peer cache: discovery events buffered in memory, flushed to storage on shutdown (TTL 7d, cap 5000)

## Chain wiring

`NetworkPeer`, `PeerPool`, `NetworkSession`, and `PayloadFetchOutcome` take a
`ChainTypes` parameter (default Ethereum). Base uses OP wire primitives, a typed
noop provider, verified bootnodes, and `basev0` discv5 with discv4 disabled.
Session admission checks chain ID and genesis; advertised history bounds remain
separate from the header-probed head. Discovery metrics also run without storage.
eth/70 incomplete final receipt blocks are dropped and reported missing for retry.
The storage cache is namespaced by chain ID and genesis; connecting with a cache
for a different chain fails before network startup. Both chain runtimes use this connector through startup dispatch.

Fetch assembly validates body and receipt commitments via `sync::validation`
before producing payloads. `invalid_blocks` is a subset of `missing_blocks`;
omitted or inconsistent responses are retried. Header requests check sequential
numbers and parent links, including chunk boundaries. `NetworkPeer::serving_range`
combines advertised history with the caller's head cap and verified head updates.
Head discovery probes peers concurrently under the existing request timeout.
