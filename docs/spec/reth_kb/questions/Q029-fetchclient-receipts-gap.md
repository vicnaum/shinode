# Q029 - Why FetchClient Lacks Receipts + Manual Path

Status: verified

## Question
Why doesn’t FetchClient support receipts, and what is the recommended manual path?
I want where FetchClient is defined, what request types it supports, and the alternative path for receipts.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q029):
Why doesn’t FetchClient support receipts, and what is the recommended manual path?
I want where FetchClient is defined, what request types it supports, and the alternative path for receipts.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Show which requests FetchClient supports and where that is defined.
3. Show the manual receipts path using PeerRequestSender and ActiveSession.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. FetchClient capabilities (what it supports)
4. Manual receipts path (step-by-step)
5. Key abstractions/types (with file paths)
6. "Where to look next"
7. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-network.xml` pack, here is the analysis of why `FetchClient` lacks receipt support and how to perform manual receipt requests.

### 1. Summary
*   **Pipeline Design**: `FetchClient` is the specific interface for the **Sync Pipeline**. In Reth's design (Staged Sync), the node downloads Headers and Bodies, but **generates Receipts locally** by executing transactions. Therefore, the pipeline fetcher has no need to request receipts from peers.
*   **Limited Enum**: The underlying `DownloadRequest` enum used by `FetchClient` strictly explicitly only supports `GetBlockHeaders` and `GetBlockBodies`.
*   **Manual Path**: To download receipts (e.g., for debugging, or if building a Snap Sync client), you must bypass the `FetchClient` and use the **`PeerRequestSender`** interface.
*   **Session Direct Access**: This is done by obtaining a handle to a specific peer session (often via `NetworkHandle` or event listeners) and sending a `PeerRequest::GetReceipts` message directly to that session's background task.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/fetch/client.rs` | `FetchClient` | The high-level client used by the Sync Pipeline. **Only** implements `HeadersClient` and `BodiesClient`. |
| `crates/net/network/src/fetch/mod.rs` | `DownloadRequest` | The internal enum defining what the `FetchClient` is physically capable of sending to the `StateFetcher`. |
| `crates/net/network-api/src/events.rs` | `PeerRequest` | The broader enum that includes **all** supported requests, including `GetReceipts`. |
| `crates/net/network-api/src/events.rs` | `PeerRequestSender` | The handle used to send `PeerRequest`s to a specific active session. |

---

### 3. FetchClient Capabilities (Headers & Bodies Only)

The `FetchClient` is restricted by the `DownloadRequest` enum. It cannot send a receipt request because the enum variant simply does not exist in the code path that `FetchClient` uses (`StateFetcher`).

**Definition of `FetchClient`:**
*   **File**: `crates/net/network/src/fetch/client.rs`
*   It implements `HeadersClient` and `BodiesClient`.
*   It sends messages into `request_tx`, which is a sender for `DownloadRequest`.

**Definition of `DownloadRequest`:**
*   **File**: `crates/net/network/src/fetch/mod.rs`
*   **Code**:
    ```rust
    pub(crate) enum DownloadRequest<N: NetworkPrimitives> {
        GetBlockHeaders { ... },
        GetBlockBodies { ... },
    }
    ```
*   **Observation**: There is no `GetReceipts` variant here. The `StateFetcher` (which consumes these requests) has no logic to handle receipt responses or routing.

---

### 4. Recommended Manual Path for Receipts

To fetch receipts, you must interact with the **Session Layer** directly or via the **Network Handle**, bypassing the `StateFetcher`.

#### Option A: Using `NetworkHandle` (Easiest)
You can route a request to a specific peer ID via the Network Manager.

1.  **Construct Request**: Create the `GetReceipts` payload.
2.  **Create Channel**: Create a `oneshot` channel to receive the response.
3.  **Wrap**: Wrap it in `PeerRequest::GetReceipts`.
4.  **Send**: Use `network_handle.send_request(peer_id, request)`.

**Tracing the code:**
*   `NetworkHandle::send_request` (`crates/net/network/src/network.rs`) sends a `NetworkHandleMessage::EthRequest` to the manager.
*   `NetworkManager::on_handle_message` (`crates/net/network/src/manager.rs`) handles this by calling `self.swarm.sessions_mut().send_message(...)`.
*   `SessionManager` forwards it to the `ActiveSession` via `commands_to_session`.

#### Option B: Using `PeerRequestSender` (Direct Session Access)
If you are listening to `NetworkEvent::ActivePeerSession`, you receive a `PeerRequestSender` directly connected to that peer's socket task.

**Step-by-Step Flow:**

1.  **Define Request**:
    ```rust
    // crates/net/eth-wire-types/src/receipts.rs
    let request = GetReceipts(vec![block_hash]);
    ```

2.  **Prepare Response Channel**:
    ```rust
    let (tx, rx) = tokio::sync::oneshot::channel();
    ```

3.  **Wrap in API Enum**:
    ```rust
    // crates/net/network-api/src/events.rs
    let peer_req = PeerRequest::GetReceipts {
        request,
        response: tx,
    };
    ```

4.  **Send**:
    ```rust
    // crates/net/network-api/src/events.rs
    // sender comes from NetworkEvent::ActivePeerSession or PeersHandle
    peer_request_sender.try_send(peer_req)?;
    ```

5.  **Await Response**:
    ```rust
    let receipts = rx.await?; // Returns RequestResult<Receipts>
    ```

---

### 5. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `DownloadRequest` | `crates/net/network/src/fetch/mod.rs` | The constrained enum used by `FetchClient`. **Excludes** receipts. |
| `PeerRequest` | `crates/net/network-api/src/events.rs` | The comprehensive enum used by the Session layer. **Includes** `GetReceipts`, `GetPooledTransactions`, etc. |
| `PeerRequestSender` | `crates/net/network-api/src/events.rs` | A direct channel to an `ActiveSession`. Bypasses the `StateFetcher`. |
| `ActiveSession` | `crates/net/network/src/session/active.rs` | The worker that actually processes `PeerRequest`s and writes to the TCP socket. |

---

### 6. Where to Look Next
*   **`crates/net/network/src/session/active.rs`**: Look at `on_internal_peer_request`. This function handles the `PeerRequest` enum. You will see the specific match arm for `GetReceipts` that is missing from the `FetchClient` logic.
*   **`crates/net/network/src/network.rs`**: Look at `send_request`. This is the high-level entry point for sending manual requests without needing a direct handle to the session.

---

### 7. Suggested Follow-up Questions
*   "How does `ActiveSession` determine whether to send a `GetReceipts` (Legacy) or `GetReceipts70` (EIP-7975) request based on the peer's capabilities?"
*   "If I use `NetworkHandle::send_request`, how do I handle the case where the peer disconnects before replying?"
*   "Why does `FetchClient` support `HeadersRequest` (which supports direction/skip) but `GetBlockBodies` (in `FetchClient`) is just a list of hashes?"

## Verification
- `FetchClient` only implements `HeadersClient` and `BodiesClient`, and uses `DownloadRequest::GetBlockHeaders` and `DownloadRequest::GetBlockBodies` to send requests through its channel.```34:102:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/fetch/client.rs
pub struct FetchClient<N: NetworkPrimitives = EthNetworkPrimitives> {
    pub(crate) request_tx: UnboundedSender<DownloadRequest<N>>,
    // ...
}
impl<N: NetworkPrimitives> HeadersClient for FetchClient<N> {
    fn get_headers_with_priority(...) -> Self::Output {
        let (response, rx) = oneshot::channel();
        if self.request_tx
            .send(DownloadRequest::GetBlockHeaders { request, response, priority })
            .is_ok()
        { /* ... */ }
    }
}
impl<N: NetworkPrimitives> BodiesClient for FetchClient<N> {
    fn get_block_bodies_with_priority_and_range_hint(...) -> Self::Output {
        let (response, rx) = oneshot::channel();
        if self.request_tx
            .send(DownloadRequest::GetBlockBodies { request, response, priority, range_hint })
            .is_ok()
        { /* ... */ }
    }
}
```
- `DownloadRequest` only contains header and body variants; no receipts.```492:508:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/fetch/mod.rs
pub(crate) enum DownloadRequest<N: NetworkPrimitives> {
    GetBlockHeaders { ... },
    GetBlockBodies { ... },
}
```
- The broader `PeerRequest` enum includes receipts variants (`GetReceipts`, `GetReceipts69`, `GetReceipts70`) and `PeerRequestSender` is the direct session channel.```223:249:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
GetReceipts { request: GetReceipts, response: oneshot::Sender<RequestResult<Receipts<N::Receipt>>> },
GetReceipts69 { request: GetReceipts, response: oneshot::Sender<RequestResult<Receipts69<N::Receipt>>> },
GetReceipts70 { request: GetReceipts70, response: oneshot::Sender<RequestResult<Receipts70<N::Receipt>>> },
```
```309:333:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
pub struct PeerRequestSender<R = PeerRequest> { pub peer_id: PeerId, pub to_session_tx: mpsc::Sender<R> }
pub fn try_send(&self, req: R) -> Result<(), mpsc::error::TrySendError<R>> { self.to_session_tx.try_send(req) }
```
- `NetworkHandle::send_request` is a manual path for targeting a specific peer. It sends a `NetworkHandleMessage::EthRequest` which the manager forwards to the session.```123:125:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/network.rs
pub fn send_request(&self, peer_id: PeerId, request: PeerRequest<N>) {
    self.send_message(NetworkHandleMessage::EthRequest { peer_id, request })
}
```
```668:670:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
NetworkHandleMessage::EthRequest { peer_id, request } => {
    self.swarm.sessions_mut().send_message(&peer_id, PeerMessage::EthRequest(request))
}
```
- The session request channel is created when the session is established and wired into `ActiveSession`.```528:562:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
let (to_session_tx, messages_rx) = mpsc::channel(self.session_command_buffer);
let messages = PeerRequestSender::new(peer_id, to_session_tx);
// ...
let session = ActiveSession {
    internal_request_rx: ReceiverStream::new(messages_rx).fuse(),
    // ...
};
```
- `ActiveSession::on_internal_peer_request` builds the wire message and queues it for sending.```317:323:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
let msg = request.create_request_message(request_id).map_versioned(self.conn.version());
self.queued_outgoing.push_back(msg.into());
```
```631:637:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
if let Some(msg) = this.queued_outgoing.pop_front() {
    let res = match msg {
        OutgoingMessage::Eth(msg) => this.conn.start_send_unpin(msg),
        // ...
    };
    // ...
}
```

## Corrections / Caveats
- The motivation that receipts are generated locally by execution is consistent with staged sync, but it is not explicitly stated in these network pack files. This is a design inference and should be treated as context, not a code‑level guarantee.
- `PeerRequestSender` is obtained from `NetworkEvent::ActivePeerSession`, not from `PeersHandle`.
- `ActiveSession::on_internal_peer_request` does not have a receipts‑specific match arm; it handles all `PeerRequest` variants generically via `create_request_message(...).map_versioned(...)`.

## Actionable Pointers
- FetchClient and DownloadRequest limits: `reth/crates/net/network/src/fetch/client.rs` and `reth/crates/net/network/src/fetch/mod.rs`.
- Manual request path: `reth/crates/net/network/src/network.rs`, `reth/crates/net/network/src/manager.rs`, `reth/crates/net/network/src/session/active.rs`.
- Receipts request variants: `reth/crates/net/network-api/src/events.rs`.
