# Q024 - EthRequestHandler (DevP2P ETH Requests)

Status: verified

## Question
How are devp2p ETH requests served (EthRequestHandler)?

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q024):
How are devp2p ETH requests served (EthRequestHandler)?
I want the request flow, how it hits providers/storage, and how responses are built.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Trace at least one request end-to-end (headers, bodies, receipts, etc.).
3. Describe any rate limits, timeouts, or peer validation that affect request serving.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Algorithm flow (step-by-step)
4. Key abstractions/types (with file paths)
5. "Where to look next"
6. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-network.xml` pack, here is the architectural breakdown of how Reth handles incoming DevP2P ETH requests via the `EthRequestHandler`.

### 1. Summary
*   **Dedicated Task**: Request serving is offloaded to a standalone background task (`EthRequestHandler`) to prevent database I/O from blocking the main `NetworkManager` or peer sessions.
*   **Channel-Based Routing**: Requests flow from `ActiveSession` (TCP) $\to$ `SessionManager` $\to$ `NetworkManager` $\to$ `EthRequestHandler` via unbounded and bounded channels.
*   **Oneshot Responses**: The `ActiveSession` creates a `oneshot` channel for every incoming request. The `EthRequestHandler` executes the database query and sends the result back through this channel, where `ActiveSession` picks it up and writes to the socket.
*   **Soft Limits**: Responses are capped by "Soft Limits" (e.g., 2MB or 1024 items) to prevent DoS attacks and bandwidth saturation, adhering to the DevP2P specs.
*   **Trait-Abstracted Storage**: The handler does not know about MDBX or RocksDB directly; it interacts with a generic `client` implementing traits like `BlockReader` and `HeaderProvider`.
*   **Protocol Agnostic Logic**: The handler serves data based on the request type (e.g., stripping receipt blooms for `eth/69` requests) while using shared underlying data fetching logic.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/eth_requests.rs` | `EthRequestHandler` | The worker struct that polls for requests and fetches data from the DB. |
| `crates/net/network/src/eth_requests.rs` | `IncomingEthRequest` | Enum wrapping the specific request (Headers/Bodies/Receipts) and the `oneshot::Sender` for the response. |
| `crates/net/network/src/manager.rs` | `NetworkManager::on_eth_request` | The routing function that moves requests from the swarm into the request handler channel. |
| `crates/net/network/src/session/active.rs` | `ActiveSession::on_incoming_message` | The initial point where bytes off the wire are decoded into a `Get*` request and the `oneshot` channel is created. |

---

### 3. Request Flow: "GetBlockHeaders" End-to-End

#### Step 1: Ingress (Session Layer)
An `ActiveSession` receives an encrypted packet, decodes it into `EthMessage::GetBlockHeaders`.
*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `on_incoming_message`
*   **Logic**:
    1.  It matches `EthMessage::GetBlockHeaders(req)`.
    2.  It calls `oneshot::channel()` to create a `(tx, rx)` pair.
    3.  It stores the `rx` (receiver) in `self.received_requests_from_remote`.
    4.  It wraps the `tx` and the request into `PeerRequest::GetBlockHeaders`.
    5.  It emits this up to the Manager via `try_emit_request`.

#### Step 2: Routing (Manager Layer)
The `SessionManager` receives the event and bubbles it to the `NetworkManager`.
*   **File**: `crates/net/network/src/manager.rs`
*   **Function**: `on_peer_message` $\to$ `on_eth_request`
*   **Logic**:
    1.  `NetworkManager` receives `PeerMessage::EthRequest`.
    2.  It delegates to `delegate_eth_request`.
    3.  It converts the `PeerRequest` into an `IncomingEthRequest`.
    4.  It sends this object into `self.to_eth_request_handler` (a bounded mpsc channel).

#### Step 3: Processing (Handler Layer)
The `EthRequestHandler` loop picks up the message.
*   **File**: `crates/net/network/src/eth_requests.rs`
*   **Function**: `EthRequestHandler::poll`
*   **Logic**:
    1.  It drains `incoming_requests` stream using a budget.
    2.  Matches `IncomingEthRequest::GetBlockHeaders`.
    3.  Calls `self.on_headers_request(peer_id, request, response)`.

#### Step 4: Storage Access & Response Building
Inside `get_headers_response`, the handler queries the database.
*   **File**: `crates/net/network/src/eth_requests.rs`
*   **Function**: `get_headers_response`
*   **Logic**:
    1.  **Resolve Start**: Converts `BlockHashOrNumber` to a concrete block number using `self.client.block_hash` or `header_by_hash_or_number`.
    2.  **Loop**: Iterates up to `request.limit` (max 1024).
    3.  **Fetch**: Calls `self.client.header_by_hash_or_number`. (Note: The implementation of `BlockReader` is outside this pack).
    4.  **Enforce Limits**: Accumulates byte size. If `total_bytes > SOFT_RESPONSE_LIMIT` (2MB), breaks the loop early.
    5.  **Traversal**: Calculates the next block number based on `request.skip` and `request.direction` (Rising/Falling).

#### Step 5: Egress
The handler sends the result back.
*   **Logic**: `response.send(Ok(BlockHeaders(headers)))`.
*   **Back to Session**: The `ActiveSession` polling loop sees the `oneshot` `rx` is ready, wraps the `BlockHeaders` in an RLP message, and queues it for TCP transmission.

---

### 4. Key Abstractions & Types

*   **`EthRequestHandler<C>`**: Generic over `C`, the client. This decoupling allows the network crate to function without knowing if the DB is MDBX, in-memory, or a mock.
*   **`IncomingEthRequest`**:
    ```rust
    pub enum IncomingEthRequest<N: NetworkPrimitives> {
        GetBlockHeaders {
            peer_id: PeerId,
            request: GetBlockHeaders,
            response: oneshot::Sender<RequestResult<BlockHeaders<N::BlockHeader>>>,
        },
        // ... Bodies, Receipts, etc.
    }
    ```
*   **Limits Constants** (`crates/net/network/src/eth_requests.rs`):
    *   `MAX_HEADERS_SERVE`: 1024
    *   `MAX_BODIES_SERVE`: 1024
    *   `MAX_RECEIPTS_SERVE`: 1024
    *   `SOFT_RESPONSE_LIMIT`: 2 * 1024 * 1024 (2MB)

---

### 5. Where to Look Next

1.  **`crates/net/network/src/eth_requests.rs`**: Read `get_headers_response` and `on_bodies_request` to see the exact logic for traversal, skipping, and size limiting.
2.  **`crates/net/network/src/session/active.rs`**: Look at `on_incoming_message` to see how the `oneshot` channel is created and how `received_requests_from_remote` tracks pending DB queries.
3.  **`crates/net/network/src/manager.rs`**: Look at `delegate_eth_request` to see how the node handles the case where the `EthRequestHandler` channel is full (metrics increment: `total_dropped_eth_requests_at_full_capacity`).

---

### 6. Suggested Follow-up Questions
*   "Since `BlockReader` is a trait, where is the concrete implementation that reads from MDBX located?"
*   "How does `EthRequestHandler` handle `GetReceipts` differently for `eth/66` versus `eth/69` (bloom filter removal)?"
*   "What happens to the `ActiveSession` if the `EthRequestHandler` takes too long to respond (timeout logic in `active.rs`)?"

## Verification
- `EthRequestHandler` is a background service (Future) that polls incoming requests with a budget and dispatches to request-specific handlers.```316:356:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
impl<C, N> Future for EthRequestHandler<C, N> { ... metered_poll_nested_stream_with_budget!(..., this.incoming_requests.poll_next_unpin(cx), |incoming| { match incoming { ... }}) }
```
- Requests are created in `ActiveSession::on_incoming_message` with a `oneshot` sender and queued `ReceivedRequest` that is later polled for the response.```168:185:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
let (tx, response) = oneshot::channel();
let received = ReceivedRequest { request_id, rx: PeerResponse::$resp_item { response }, received: Instant::now() };
self.received_requests_from_remote.push(received);
self.try_emit_request(PeerMessage::EthRequest(PeerRequest::$req_item { request, response: tx }))
```
- Pending responses are polled and encoded back onto the wire.```618:628:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
match req.rx.poll(cx) { Poll::Ready(resp) => { this.handle_outgoing_response(req.request_id, resp); } ... }
```
```374:378:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
fn handle_outgoing_response(&mut self, id: u64, resp: PeerResponseResult<N>) { ... self.queued_outgoing.push_back(msg.into()); }
```
- Routing is ActiveSession -> SessionManager -> Swarm -> NetworkManager -> EthRequestHandler.```470:472:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
ActiveSessionMessage::ValidMessage { peer_id, message } => Poll::Ready(SessionEvent::ValidMessage { peer_id, message })
```
```151:152:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/swarm.rs
SessionEvent::ValidMessage { peer_id, message } => Some(SwarmEvent::ValidMessage { peer_id, message })
```
```610:635:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
PeerMessage::EthRequest(req) => { self.on_eth_request(peer_id, req); }
```
```498:505:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
PeerRequest::GetBlockHeaders { request, response } => { self.delegate_eth_request(IncomingEthRequest::GetBlockHeaders { peer_id, request, response }) }
```
- Channel capacity for EthRequestHandler is bounded (256) and drops on full with metrics increment.```19:71:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/builder.rs
pub(crate) const ETH_REQUEST_CHANNEL_CAPACITY: usize = 256;
let (tx, rx) = mpsc::channel(ETH_REQUEST_CHANNEL_CAPACITY);
```
```484:492:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
if let TrySendError::Full(_) = e { self.metrics.total_dropped_eth_requests_at_full_capacity.increment(1); }
```
- Soft limits and max counts enforced for headers/bodies/receipts; SOFT_RESPONSE_LIMIT is 2MB and MAX_* is 1024.```32:49:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
pub const MAX_RECEIPTS_SERVE: usize = 1024;
pub const MAX_HEADERS_SERVE: usize = 1024;
pub const MAX_BODIES_SERVE: usize = 1024;
pub const SOFT_RESPONSE_LIMIT: usize = 2 * 1024 * 1024;
```
- Headers are served via `block_hash` + `header_by_hash_or_number`, respecting skip/direction and size limits.```88:116:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
let GetBlockHeaders { start_block, limit, skip, direction } = request;
...
if let Some(header) = self.client.header_by_hash_or_number(block).unwrap_or_default() {
    total_bytes += header.length();
    headers.push(header);
    if headers.len() >= MAX_HEADERS_SERVE || total_bytes > SOFT_RESPONSE_LIMIT { break }
    match direction { HeadersDirection::Rising => ..., HeadersDirection::Falling => ... }
}
```
- Receipts: eth/66 uses `ReceiptWithBloom`, eth/69 skips bloom, eth/70 supports partial last block.```190:219:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
let receipts = self.get_receipts_response(request, |receipts_by_block| { receipts_by_block.into_iter().map(ReceiptWithBloom::from).collect::<Vec<_>>() });
```
```205:218:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
// skip bloom filter for eth69
```
```221:281:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/eth_requests.rs
GetReceipts70 { first_block_receipt_index, block_hashes } ... last_block_incomplete ... partial_block ...
```

## Corrections / Caveats
- The routing channels are bounded (e.g., ETH request handler capacity is 256); there is no unbounded channel in this path.
- `IncomingEthRequest::GetNodeData` currently only increments metrics; it does not build or send a response.
- The "soft limit" is enforced by encoded byte length, not by number of items alone; MAX_* limits are additional caps.
- Request throttling on the session side is via `MAX_QUEUED_OUTGOING_RESPONSES` (4), which pauses receiving when too many responses are pending.

## Actionable Pointers
- Request handling and limits: `reth/crates/net/network/src/eth_requests.rs`.
- Session request/response lifecycle: `reth/crates/net/network/src/session/active.rs`.
- Routing + channel backpressure: `reth/crates/net/network/src/manager.rs` and `reth/crates/net/network/src/builder.rs`.
