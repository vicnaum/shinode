# Q030 - Session Request Failure Handling (Timeouts + Wrong Variant + send_request)

Status: verified

## Question
How are session request failures handled for manual receipts requests?
I want timeouts, wrong response variant handling, and how disconnect/timeout is surfaced when using NetworkHandle::send_request.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q030):
How are session request failures handled for manual receipts requests?
I want timeouts, wrong response variant handling, and how disconnect/timeout is surfaced when using NetworkHandle::send_request.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Explain ActiveSession inflight timeout handling and resulting errors.
3. Explain what happens if a response variant doesn’t match the original request (e.g., Receipts69 vs Receipts).
4. Trace how errors reach the caller for NetworkHandle::send_request (oneshot, RequestError, etc.).
5. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Timeout handling (step-by-step)
4. Wrong-variant handling (step-by-step)
5. Error propagation to callers (step-by-step)
6. Key abstractions/types (with file paths)
7. "Where to look next"
8. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-network.xml` pack, here is the analysis of how manual receipt request failures are handled in Reth.

### 1. Summary
*   **Oneshot Channel**: Manual requests use a `oneshot::Sender` embedded in the `PeerRequest` enum to deliver results back to the caller.
*   **In-Session Management**: The `ActiveSession` task tracks all manual requests in an `inflight_requests` map. It does *not* rely on the `StateFetcher`.
*   **Timeout Logic**: Timeouts are enforced internally by the `ActiveSession`. If a response isn't received within `INITIAL_REQUEST_TIMEOUT` (default 20s), the session sends a `RequestError::Timeout` to the caller.
*   **Strict Variant Matching**: Reth enforces strict 1:1 mapping between the request type (e.g., `GetReceipts`) and the response type (e.g., `Receipts`). If a peer responds with `Receipts69` when the internal request was tracked as `GetReceipts`, the session treats it as a **Bad Response**.
*   **Drop on Disconnect**: If the session drops (disconnect/error), the `inflight_requests` map is dropped. This drops the `oneshot::Sender`, causing the caller's `rx.await` to fail with a `RecvError`.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/network.rs` | `NetworkHandle::send_request` | Public entry point. Wraps the request in `NetworkHandleMessage` and sends it to the Manager. |
| `crates/net/network/src/session/active.rs` | `ActiveSession::on_internal_peer_message` | Receives the request from the Manager and queues it. |
| `crates/net/network/src/session/active.rs` | `InflightRequest` | Struct tracking the pending request, its timestamp, and the response channel. |
| `crates/net/network/src/session/active.rs` | `check_timed_out_requests` | Polling logic that identifies expired requests. |

---

### 3. Timeout Handling

Timeouts are managed actively by the `ActiveSession` event loop, not by the caller's future.

#### Step 1: Registration
When `on_internal_peer_request` receives a request, it wraps it in an `InflightRequest` with a `deadline`.
*   **File**: `crates/net/network/src/session/active.rs`
*   **Code**: `deadline` is calculated as `Instant::now() + internal_request_timeout` (default 20s).

#### Step 2: Polling
Inside `ActiveSession::poll`, the `internal_request_timeout_interval` ticks.
*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `poll`
*   **Call**: `if this.check_timed_out_requests(Instant::now()) ...`

#### Step 3: Enforcement
Inside `check_timed_out_requests`:
1.  Iterates `inflight_requests`.
2.  If `req.is_timed_out(now)` is true:
3.  Calls `req.timeout()`.
4.  `req.timeout()` calls `req.send_err_response(RequestError::Timeout)`.
5.  **Result**: The caller's `await` returns `Ok(Err(RequestError::Timeout))`.

---

### 4. Wrong-Variant Handling

Reth enforces that the response wire message matches the request variant stored in memory. This is critical for Receipts because there are three versions (`Receipts`, `Receipts69`, `Receipts70`).

#### Step 1: Reception
The `ActiveSession` receives an `EthMessage` from the `EthStream`.
*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `on_incoming_message`

#### Step 2: Dispatch
The code matches the enum variant.
*   Case A: `EthMessage::Receipts(resp)` calls `on_response!(resp, GetReceipts)`.
*   Case B: `EthMessage::Receipts69(resp)` calls `on_response!(resp, GetReceipts69)`.

#### Step 3: Verification (The Mismatch)
The `on_response!` macro looks up the `request_id` in `inflight_requests`. It then pattern matches the **stored request**.

*   **Scenario**: User sent `PeerRequest::GetReceipts`. Peer responded with `Receipts69`.
*   **Execution**:
    1.  `EthMessage::Receipts69` arm triggers.
    2.  Macro expects `PeerRequest::GetReceipts69` in `inflight_requests`.
    3.  Actual found is `PeerRequest::GetReceipts`.
    4.  Match falls through to `RequestState::Waiting(request) => request.send_bad_response()`.

#### Step 4: Error Propagation
`send_bad_response` sends `RequestResult::Err(RequestError::BadResponse)`.
*   **Result**: The caller receives `Ok(Err(RequestError::BadResponse))`.

> **Note**: This implies that when using `NetworkHandle::send_request` manually, the user **must** check the peer's negotiated capabilities and send the correct variant (`GetReceipts` vs `GetReceipts69`), otherwise the session will reject the response even if it's valid RLP.

---

### 5. Error Propagation to Callers

Errors reach the caller in two ways: via the `Result` inside the channel, or by the channel closing.

#### A. Explicit Logic Errors (Timeout, BadResponse)
Defined in `crates/net/network-p2p/src/error.rs`.
1.  Session calls `response.send(Err(RequestError::Timeout))`.
2.  Caller awaits channel: `rx.await` returns `Ok(Err(RequestError::Timeout))`.

#### B. Connection Dropped / Session Closed
If the peer disconnects or the session crashes:
1.  `ActiveSession` is dropped.
2.  `inflight_requests` map is dropped.
3.  The `oneshot::Sender` inside `InflightRequest` is dropped without sending.
4.  Caller awaits channel: `rx.await` returns `Err(RecvError)`.

---

### 6. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `InflightRequest` | `crates/net/network/src/session/active.rs` | Holds the `oneshot::Sender` and the `deadline` timestamp. |
| `PeerRequest` | `crates/net/network-api/src/events.rs` | Enum of possible requests (`GetReceipts`, `GetReceipts69`, etc). |
| `RequestError` | `crates/net/p2p/src/error.rs` | Enum for failures: `ChannelClosed`, `ConnectionDropped`, `Timeout`, `BadResponse`. |
| `ActiveSession` | `crates/net/network/src/session/active.rs` | The actor managing the state, enforcing timeouts, and routing responses. |

---

### 7. Where to Look Next
1.  **`crates/net/network/src/session/active.rs`**: Specifically the `check_timed_out_requests` function to see exactly how timeouts trigger.
2.  **`crates/net/network/src/session/active.rs`**: The `on_incoming_message` match block to see the `on_response!` macro expansion and how it enforces type strictness.
3.  **`crates/net/network-p2p/src/error.rs`**: To see the definition of `RequestError` and how `EthResponseValidator` (used for headers) is *not* used for receipts (which use exact matching).

---

### 8. Suggested Follow-up Questions
*   "How can I determine the negotiated `EthVersion` of a peer before sending a request to ensure I choose `GetReceipts` vs `GetReceipts69`?"
*   "Does `ActiveSession` penalize the peer's reputation when a `Timeout` or `BadResponse` occurs for a manual request?"
*   "Why does `ActiveSession` automatically upgrade outgoing messages via `map_versioned` but not automatically downgrade/map incoming response types?"

## Verification
- `NetworkHandle::send_request` sends `NetworkHandleMessage::EthRequest`, which the manager forwards to the session.```123:126:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/network.rs
pub fn send_request(&self, peer_id: PeerId, request: PeerRequest<N>) {
    self.send_message(NetworkHandleMessage::EthRequest { peer_id, request })
}
```
```668:670:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
NetworkHandleMessage::EthRequest { peer_id, request } => {
    self.swarm.sessions_mut().send_message(&peer_id, PeerMessage::EthRequest(request))
}
```
- The session assigns deadlines on `PeerMessage::EthRequest` and uses `request_deadline()` to compute them.```348:369:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
PeerMessage::EthRequest(req) => {
    let deadline = self.request_deadline();
    self.on_internal_peer_request(req, deadline);
}
fn request_deadline(&self) -> Instant {
    Instant::now() +
        Duration::from_millis(self.internal_request_timeout.load(Ordering::Relaxed))
}
```
- Default timeouts are configured in `SessionsConfig`: 20s internal timeout, 2 minutes protocol breach timeout.```6:71:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-types/src/session/config.rs
pub const INITIAL_REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
pub const PROTOCOL_BREACH_REQUEST_TIMEOUT: Duration = Duration::from_secs(2 * 60);
pub struct SessionsConfig { pub initial_internal_request_timeout: Duration, pub protocol_breach_request_timeout: Duration, ... }
```
- `check_timed_out_requests` triggers `RequestError::Timeout` via `InflightRequest::timeout`, and a second-stage timeout causes a protocol breach event.```497:514:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
fn check_timed_out_requests(&mut self, now: Instant) -> bool {
    for (id, req) in &mut self.inflight_requests {
        if req.is_timed_out(now) {
            if req.is_waiting() {
                debug!(..., "timed out outgoing request");
                req.timeout();
            } else if now - req.timestamp > self.protocol_breach_request_timeout {
                return true
            }
        }
    }
    false
}
```
```815:821:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
fn timeout(&mut self) {
    // ...
    if let RequestState::Waiting(req) = req {
        req.send_err_response(RequestError::Timeout);
    }
}
```
- Response variant mismatches are treated as bad responses via `send_bad_response` in `PeerRequest`.```189:201:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
match req.request {
    RequestState::Waiting(PeerRequest::$item { response, .. }) => { let _ = response.send(Ok(message)); }
    RequestState::Waiting(request) => { request.send_bad_response(); }
    // ...
}
```
```254:269:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
pub fn send_bad_response(self) { self.send_err_response(RequestError::BadResponse) }
```
- Receipts response variants are matched explicitly.```266:283:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
EthMessage::Receipts(resp) => { on_response!(resp, GetReceipts) }
EthMessage::Receipts69(resp) => { on_response!(resp, GetReceipts69) }
EthMessage::Receipts70(resp) => { on_response!(resp, GetReceipts70) }
```
- If the oneshot sender is dropped, `RequestError::ChannelClosed` is the canonical conversion target.```124:127:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/error.rs
impl From<oneshot::error::RecvError> for RequestError {
    fn from(_: oneshot::error::RecvError) -> Self { Self::ChannelClosed }
}
```

## Corrections / Caveats
- The internal timeout is adaptive: `update_request_timeout` recalculates `internal_request_timeout` based on observed RTT and resets the interval; it is not a fixed 20s once the session starts.```521:528:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
let request_timeout = calculate_new_timeout(current, elapsed);
self.internal_request_timeout.store(request_timeout.as_millis() as u64, Ordering::Relaxed);
self.internal_request_timeout_interval = tokio::time::interval(request_timeout);
```
- The "session dropped -> RecvError" path is an expected oneshot behavior, but is not explicitly emitted in these files (the sender is dropped when the session task is dropped).

## Actionable Pointers
- Timeout logic and protocol breach handling: `reth/crates/net/network/src/session/active.rs`.
- Timeout defaults/config: `reth/crates/net/network-types/src/session/config.rs`.
- Request error variants: `reth/crates/net/p2p/src/error.rs`.
- Manual request routing: `reth/crates/net/network/src/network.rs` and `reth/crates/net/network/src/manager.rs`.
