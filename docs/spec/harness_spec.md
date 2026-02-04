Here’s a **minimal “receipt-availability harness” spec** built on **reth as a library**, with **Option 1** constraints: *no receipt validation, no execution, no DB/state*, just “what peers exist + what history they actually serve” (ETH mainnet + Base).

---

## 0) What the harness must answer

For each network (Ethereum mainnet, Base):

1. **How many peers can I connect to and keep a session with?**
2. For each connected peer, **how far back will it answer**:

   * `GetBlockHeaders` (by number / hash)
   * `GetReceipts` (by block hash)
3. **Distribution + failure modes**:

   * success rate by depth/age
   * timeouts, disconnects, “not serving”, rate limits, malformed responses
   * latency stats and response sizes

No canonicality / reorg handling yet. No receiptsRoot checking yet.

---

## 1) Minimal probing model (what you actually do)

### Peer lifecycle

* Maintain up to `N` concurrent peer sessions (say 50–200).
* On `PeerConnected`, record:

  * `peer_id`, `client_version`, `eth_version`, `capabilities`, remote addr, status payload, etc. (reth exposes this via `PeerInfo`). ([Reth][1])
* On `PeerDisconnected`, record reason + how long session lasted.

### Probe plan per peer

You need *peer head number* so you can probe “how far back” relative to that peer.

1. Read peer’s `status.best_hash` from `PeerInfo.status`.
2. Request **that header** (`GetBlockHeaders` by hash, `amount=1`).
3. Parse returned header to learn `head_number`.
4. Choose a list of probe block numbers relative to `head_number`:

   * Example (block offsets): `0, 1_000, 10_000, 100_000, 1_000_000, 5_000_000`
   * Or (time-based): “now”, “1d”, “7d”, “30d”, “180d”, “365d” by first getting header timestamps and binary searching (later).
5. For each target block number:

   * `GetBlockHeaders(number, amount=1)`
   * If header returned: `GetReceipts([header.hash])`
   * Record success/err + latency + `#receipts`, `#logs`, encoded size (optional)

Keep it **tiny** (amount=1) to avoid being “that guy” and to keep results attributable.

---

## 2) What to store (outputs)

### Per-peer record (JSONL is enough)

* Static:

  * `network`, `peer_id`, `client_version`, `enode/enr` if present, `eth_version`, capabilities ([Reth][1])
* Session:

  * connected_at, disconnected_at, disconnect_reason
  * derived `peer_head_number`
* Probe results (for each depth):

  * `depth_blocks`, `target_number`, `header_ok`, `receipts_ok`
  * `t_header_ms`, `t_receipts_ms`
  * `receipts_count`, `logs_count`, `bytes_receipts` (optional)
  * error code/class: timeout vs response error vs disconnect

### Aggregate outputs

* Availability curve:

  * `P(peer serves receipts at depth D)`
* Latency percentile curves for header + receipts
* Peer client mix (op-geth/op-reth/etc via client_version)

---

## 3) Reth components you can reuse vs what you write

### Reuse from reth (you should not write these)

**Networking / devp2p / session management**

* `reth_network`’s `NetworkManager` + `NetworkHandle` to run a node and interact with peers. ([Reth][2])
* Event streams (`peer_events()` / `event_listener()`) to track peer connect/disconnect. ([Reth][3])

**Request/response machinery**

* Use `NetworkHandle::send_request(peer_id, PeerRequest::...)`. ([Reth][3])
* Use `PeerRequest::{GetBlockHeaders, GetReceipts}` (reth already handles eth/68/69/70 variants internally). ([Reth][4])

**Bootnodes**

* Ethereum: `reth_network_peers::mainnet_nodes()` (same idea as `base_nodes`, below).
* Base: `reth_network_peers::base_nodes()` ([Reth][5])

**Chain specs / network constants**

* Ethereum: `reth_chainspec::MAINNET` (and other built-ins)
* Base (OP Stack): `reth_op::chainspec::BASE_MAINNET` (backed by included genesis JSON + sealed genesis header)

### What you write yourself

* Probe scheduler (per-peer rate limiting, timeouts, retries, concurrency)
* “Depth selection” strategy (block offsets first; time-based later)
* Metrics + persistence (JSONL/Parquet/SQLite)
* A small “classification layer” for errors (timeout vs refused vs malformed)
* (Optional but useful) “head spoofing”: after you learn a plausible head, call `NetworkHandle::update_status(...)` so more peers treat you as non-useless (pragmatic, no validation). `update_status` exists on `NetworkHandle`. ([Reth][3])

---

## 4) The minimal harness runtime topology

Single binary, two modes:

### `discover`

* Start network, connect to peers, collect `PeerInfo`, write `peers.jsonl`.

### `probe`

* Load known peers (or rely on discovery), run probe plan, write `probes.jsonl`.

No DB, no canonical chain, no syncing loop.

---

## 5) “Network data” (genesis hash, bootnodes, chain ids) — where to get it in reth

### Programmatic access

Reth’s chain specs implement `EthChainSpec`, which exposes:

* `genesis_hash()`
* `genesis_header()`
* `bootnodes()`
* `chain_id()`
  etc. ([Reth][6])

### “Give me the actual embedded config / genesis JSON”

* Ethereum: `reth dump-genesis --chain mainnet` prints the built-in genesis JSON. ([Reth][7])
* OP Stack (Base): `op-reth dump-genesis --chain <...>` exists similarly. ([Reth][8])

### Base genesis hash specifically (in-repo constant)

Base’s chainspec in reth includes a **sealed genesis header with an explicit hash constant** (so you can literally point at code).

### Bootnodes

* Base bootnodes are exposed as `reth_network_peers::base_nodes()`. ([Reth][5])

---

## 6) Practical defaults (so your first run succeeds)

* Target connections: 50 (then 200)
* Request timeouts: headers 2–3s, receipts 3–5s
* Per-peer budget: max 10 probes / minute (start lower)
* Always do:

  1. get peer head header (by best hash)
  2. then probe a small set of depths

This gets you quick signal on “do peers actually serve receipts beyond recent?” without arguing about EIP-4444 yet.

---

If you want, next step is I can map this into a **concrete crate layout** (Cargo workspace + exact reth crates/imports/features for `reth_ethereum` + `reth_op`) and a minimal async control loop (still no full code dump unless you ask).

[1]: https://reth.rs/docs/reth/network/struct.PeerInfo.html "PeerInfo in reth::network - Rust"
[2]: https://reth.rs/docs/reth/network/struct.NetworkManager.html "NetworkManager in reth::network - Rust"
[3]: https://reth.rs/docs/reth/network/struct.NetworkHandle.html "NetworkHandle in reth::network - Rust"
[4]: https://reth.rs/docs/reth/network/enum.PeerRequest.html "PeerRequest in reth::network - Rust"
[5]: https://reth.rs/docs/reth_network_peers/fn.base_nodes.html "base_nodes in reth_network_peers - Rust"
[6]: https://reth.rs/docs/reth_chainspec/trait.EthChainSpec.html "EthChainSpec in reth_chainspec - Rust"
[7]: https://reth.rs/cli/reth/dump-genesis/?utm_source=chatgpt.com "reth dump-genesis"
[8]: https://reth.rs/cli/op-reth/dump-genesis/?utm_source=chatgpt.com "op-reth dump-genesis"
