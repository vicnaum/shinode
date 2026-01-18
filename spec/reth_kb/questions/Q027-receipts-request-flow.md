# Q027 - Manual Receipts Request Flow (PeerRequestSender → Wire)

Status: verified

## Question
How does a manual receipts request flow from user code to the wire and back?
I want the end-to-end path: PeerRequestSender -> PeerRequest -> create_request_message -> map_versioned -> session send -> response routing.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q027):
How does a manual receipts request flow from user code to the wire and back?
I want the end-to-end path: PeerRequestSender -> PeerRequest -> create_request_message -> map_versioned -> session send -> response routing.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Trace the request end-to-end for receipts (legacy/eth69/eth70), including how EthVersion affects the request.
3. Show how the response is matched back to the request and delivered to the caller.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. End-to-end flow (step-by-step)
4. Version handling (eth/68 vs eth/69 vs eth/70)
5. Key abstractions/types (with file paths)
6. "Where to look next"
7. Suggested follow-up questions
```

## Gemini Answer
Based on the provided `context-network.xml` pack, here is the end-to-end flow of a manual receipts request in Reth.

### 1. Summary
*   **Channel-Based Initiation**: Users initiate requests via `PeerRequestSender`, which pushes a `PeerRequest` enum into a bounded channel processed by the peer's `ActiveSession`.
*   **Request ID Allocation**: The `ActiveSession` assigns a unique internal `u64` ID to track the request, wrapping the payload in a `RequestPair`.
*   **Version Upcasting**: The `EthMessage::map_versioned` function automatically upgrades a standard `GetReceipts` request into a `GetReceipts70` request if the connected peer negotiated `eth/70`.
*   **In-Flight Tracking**: The session stores the pending response channel in an `inflight_requests` hash map keyed by the request ID.
*   **Response Normalization**: Incoming responses (`Receipts`, `Receipts69`, or `Receipts70`) are decoded by the stream, matched by ID, and delivered to the caller via a `oneshot` channel.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/network-api/src/events.rs` | `PeerRequestSender` | The user-facing handle used to send a request to a specific peer. |
| `crates/net/network-api/src/events.rs` | `PeerRequest::GetReceipts` | The enum variant representing the user's intent to fetch receipts. |
| `crates/net/network/src/session/active.rs` | `ActiveSession` | The background task managing the peer connection that processes the request. |
| `crates/net/eth-wire-types/src/message.rs` | `EthMessage::map_versioned` | Logic that transforms the request based on the agreed protocol version. |

---

### 3. End-to-End Flow

#### Step 1: User Initiation
The user holds a `PeerRequestSender` (typically obtained via `PeersHandle` or `NetworkHandle`). They create a `oneshot` channel for the response and send the request.

*   **File**: `crates/net/network-api/src/events.rs`
*   **Function**: `PeerRequestSender::try_send`
*   **Data**: The user sends `PeerRequest::GetReceipts { request, response }` where `response` is the `oneshot::Sender`.

#### Step 2: Session Processing
The `ActiveSession` loop picks up the message from `internal_request_rx`.

*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `ActiveSession::poll` matches `internal_request_rx`.
*   **Action**: Calls `self.on_internal_peer_request(req, deadline)`.

#### Step 3: ID Assignment & Message Creation
Inside `on_internal_peer_request`, the session assigns a unique ID and converts the high-level `PeerRequest` into a wire-ready `EthMessage`.

*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `on_internal_peer_request`
*   **Logic**:
    1.  `let request_id = self.next_id();`
    2.  `req.create_request_message(request_id)` creates `EthMessage::GetReceipts(RequestPair { request_id, message })`.

#### Step 4: Version Mapping (`map_versioned`)
Before queueing the message, it is transformed to match the peer's negotiated ETH version.

*   **File**: `crates/net/network/src/session/active.rs`
*   **Call**: `.map_versioned(self.conn.version())`
*   **Implementation**: `crates/net/eth-wire-types/src/message.rs`
*   **Logic**:
    *   If version < `eth/70`: Returns `EthMessage::GetReceipts`.
    *   If version >= `eth/70`: Transforms into `EthMessage::GetReceipts70` by setting `first_block_receipt_index: 0`.

#### Step 5: Queueing & Sending
The `EthMessage` is queued and then flushed to the underlying connection.

*   **File**: `crates/net/network/src/session/active.rs`
*   **Queueing**: `self.queued_outgoing.push_back(msg.into())`.
*   **Tracking**: An `InflightRequest` holding the user's `oneshot::Sender` is inserted into `self.inflight_requests`.
*   **Transmission**: The `ActiveSession::poll` loop calls `conn.start_send_unpin(msg)`, which encodes the message to RLP and writes it to the TCP socket.

#### Step 6: Receiving the Response
The peer responds. The `EthStream` decodes the RLP based on the message ID.

*   **File**: `crates/net/eth-wire/src/ethstream.rs`
*   **Function**: `EthStreamInner::decode_message`
*   **Logic**: The byte stream is decoded into an `EthMessage`. For receipts, this could be `Receipts`, `Receipts69`, or `Receipts70` depending on the version.

#### Step 7: Response Routing
The `ActiveSession` handles the decoded incoming message.

*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `on_incoming_message`
*   **Logic**:
    1.  Matches `EthMessage::Receipts(resp)` (or `69`/`70`).
    2.  Extracts `request_id` from the `RequestPair`.
    3.  Looks up `self.inflight_requests.remove(&request_id)`.
    4.  If found, sends the payload (e.g., `Vec<Vec<Receipt>>`) through the stored `oneshot` channel back to the user.

---

### 4. Version Handling Details

The `EthMessage` enum and `map_versioned` function handle the differences transparently:

*   **Eth/66 - Eth/69**:
    *   **Request**: `GetReceipts` (List of hashes).
    *   **Response**: `Receipts` (legacy, with Bloom) or `Receipts69` (without Bloom).
    *   **Handling**: `ActiveSession` passes the result to the user. The user receives `RequestResult<Receipts<...>>` or `RequestResult<Receipts69<...>>`.

*   **Eth/70** (EIP-7975):
    *   **Request**: `GetReceipts70` (includes `firstBlockReceiptIndex`).
    *   **Conversion**: `map_versioned` converts the user's `GetReceipts` to `GetReceipts70` automatically.
    *   **Response**: `Receipts70` (includes `lastBlockIncomplete` flag).
    *   **Handling**: `ActiveSession` matches `EthMessage::Receipts70` and sends `Receipts70` back to the user.

**Ref**: `crates/net/eth-wire-types/src/message.rs` (`map_versioned` implementation).

---

### 5. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `PeerRequest` | `crates/net/network-api/src/events.rs` | Enum of all possible requests a user can make to a peer. |
| `EthMessage` | `crates/net/eth-wire-types/src/message.rs` | The wire protocol enum containing all variants (Status, GetReceipts, etc.). |
| `RequestPair<T>` | `crates/net/eth-wire-types/src/message.rs` | A struct `{ request_id: u64, message: T }` used for request/response matching. |
| `InflightRequest` | `crates/net/network/src/session/active.rs` | Internal struct storing the `oneshot::Sender` and timestamp for timeout tracking. |
| `PeerRequestSender` | `crates/net/network/src/session/active.rs` | A cloneable handle around `mpsc::Sender<PeerRequest>` used to talk to the session. |

---

### 6. Where to Look Next
1.  **`crates/net/eth-wire-types/src/message.rs`**: To see the specific logic of `map_versioned` and how it differentiates between `Eth70` and older versions.
2.  **`crates/net/network/src/session/active.rs`**: Specifically the `on_incoming_message` huge match statement to see how `Receipts69` and `Receipts70` are routed differently than `Receipts`.
3.  **`crates/net/network-api/src/events.rs`**: To see the definition of `PeerRequest::GetReceipts` variants.

---

### 7. Suggested Follow-up Questions
*   "How does `ActiveSession` handle request timeouts if the peer never responds to the `GetReceipts` request?"
*   "What happens if `map_versioned` converts a request but the peer responds with the wrong message ID variant?"
*   "How does `Receipts69` handle the missing Bloom filter when converting back to the standard `Receipt` type expected by higher-level components?"

## Verification
- `PeerRequest` defines receipts request variants and `create_request_message` maps `GetReceipts`/`GetReceipts69` to `EthMessage::GetReceipts`, with `GetReceipts70` mapping to `EthMessage::GetReceipts70`.```223:297:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
GetReceipts { request: GetReceipts, response: oneshot::Sender<RequestResult<Receipts<N::Receipt>>> },
GetReceipts69 { request: GetReceipts, response: oneshot::Sender<RequestResult<Receipts69<N::Receipt>>> },
GetReceipts70 { request: GetReceipts70, response: oneshot::Sender<RequestResult<Receipts70<N::Receipt>>> },
...
pub fn create_request_message(&self, request_id: u64) -> EthMessage<N> {
    match self {
        Self::GetReceipts { request, .. } | Self::GetReceipts69 { request, .. } => {
            EthMessage::GetReceipts(RequestPair { request_id, message: request.clone() })
        }
        Self::GetReceipts70 { request, .. } => {
            EthMessage::GetReceipts70(RequestPair { request_id, message: request.clone() })
        }
        // ...
    }
}
```
- `PeerRequestSender` is the user-facing handle wrapping an `mpsc::Sender`, created when a session is established and exposed via `NetworkEvent::ActivePeerSession`.```95:106:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
pub enum NetworkEvent<R = PeerRequest> {
    ActivePeerSession { info: SessionInfo, messages: PeerRequestSender<R> },
    // ...
}
```
```309:333:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
pub struct PeerRequestSender<R = PeerRequest> { pub peer_id: PeerId, pub to_session_tx: mpsc::Sender<R> }
pub fn try_send(&self, req: R) -> Result<(), mpsc::error::TrySendError<R>> { self.to_session_tx.try_send(req) }
```
```528:532:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
let (to_session_tx, messages_rx) = mpsc::channel(self.session_command_buffer);
let messages = PeerRequestSender::new(peer_id, to_session_tx);
```
- `ActiveSession::on_internal_peer_request` assigns a request id, applies `create_request_message(...).map_versioned(...)`, enqueues it, and stores the inflight request.```317:329:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
let request_id = self.next_id();
let msg = request.create_request_message(request_id).map_versioned(self.conn.version());
self.queued_outgoing.push_back(msg.into());
let req = InflightRequest { request: RequestState::Waiting(request), timestamp: Instant::now(), deadline };
self.inflight_requests.insert(request_id, req);
```
- `map_versioned` only upgrades `GetReceipts` to `GetReceipts70` for eth/70 peers; there is no eth/69 remapping here.```372:397:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire-types/src/message.rs
if version >= EthVersion::Eth70 {
    return match self {
        Self::GetReceipts(pair) => {
            let RequestPair { request_id, message } = pair;
            let req = RequestPair { request_id, message: GetReceipts70 { first_block_receipt_index: 0, block_hashes: message.0 } };
            Self::GetReceipts70(req)
        }
        other => other,
    }
}
```
- The session sends queued messages to the wire using the connection sink.```631:637:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
while this.conn.poll_ready_unpin(cx).is_ready() {
    if let Some(msg) = this.queued_outgoing.pop_front() {
        let res = match msg {
            OutgoingMessage::Eth(msg) => this.conn.start_send_unpin(msg),
            // ...
        };
        // ...
    }
}
```
- Response routing matches by `request_id` and by the expected `PeerRequest` variant; mismatches are treated as bad responses.```189:203:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
let RequestPair { request_id, message } = $resp;
if let Some(req) = self.inflight_requests.remove(&request_id) {
    match req.request {
        RequestState::Waiting(PeerRequest::$item { response, .. }) => {
            let _ = response.send(Ok(message));
        }
        RequestState::Waiting(request) => {
            request.send_bad_response();
        }
        // ...
    }
}
```
```266:283:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
EthMessage::Receipts(resp) => { on_response!(resp, GetReceipts) }
EthMessage::Receipts69(resp) => { on_response!(resp, GetReceipts69) }
EthMessage::Receipts70(resp) => { on_response!(resp, GetReceipts70) }
```
- For inbound `GetReceipts` handling, the response variant is chosen based on negotiated version (eth/69+ → `Receipts69`).```266:271:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
EthMessage::GetReceipts(req) => {
    if self.conn.version() >= EthVersion::Eth69 {
        on_request!(req, Receipts69, GetReceipts69)
    } else {
        on_request!(req, Receipts, GetReceipts)
    }
}
```

## Corrections / Caveats
- `PeerRequestSender` lives in `crates/net/network-api/src/events.rs` (not `session/active.rs`) and is surfaced via `NetworkEvent::ActivePeerSession`.
- For eth/69, callers must use `PeerRequest::GetReceipts69` to accept a `Receipts69` response; `map_versioned` only upgrades to eth/70 and does not perform eth/69 remapping. Response variants that do not match the original request are treated as bad responses.
- The wire decode step (`EthStreamInner::decode_message`) is part of `reth_eth_wire` and is not required for the receipts request flow as seen in `ActiveSession`; response routing is handled in `on_incoming_message`.

## Actionable Pointers
- Request creation + version mapping: `reth/crates/net/network-api/src/events.rs` and `reth/crates/net/eth-wire-types/src/message.rs`.
- Session send + inflight tracking: `reth/crates/net/network/src/session/active.rs`.
- Session request channel creation: `reth/crates/net/network/src/session/mod.rs`.
