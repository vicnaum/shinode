# Q031 - Protocol Enforcement After Handshake (Status + Version Mismatch)

Status: verified

## Question
How does the session enforce protocol correctness after handshake?
I want:
1) how Status messages are rejected after handshake, and
2) what happens if a peer negotiated eth/68 but sends an eth/69-specific message ID.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q031):
How does the session enforce protocol correctness after handshake?
I want:
1) how Status messages are rejected after handshake, and
2) what happens if a peer negotiated eth/68 but sends an eth/69-specific message ID.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Show where handshake completion is tracked.
3. Show where invalid messages are detected and how peers are penalized.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Status-after-handshake enforcement (step-by-step)
4. Wrong-version message handling (step-by-step)
5. Consequences (disconnects, bad message penalties)
6. Key abstractions/types (with file paths)
7. "Where to look next"
8. Suggested follow-up questions
```

## Gemini Answer
Based on the provided `context-network.xml` pack, here is the analysis of how Reth enforces protocol correctness after the handshake, specifically regarding `Status` messages and version mismatches.

### 1. Summary
*   **State-Based Enforcement**: The network stack strictly separates the "Handshake" phase from the "Active" phase using different types. `Status` messages are only allowed during the `UnauthedEthStream` phase. Once upgraded to `EthStream`, receiving a `Status` message triggers an immediate error.
*   **Decoding-Level Validation**: Protocol correctness is enforced primarily during message decoding in `EthStreamInner` and `ProtocolMessage`. The decoder is aware of the negotiated `EthVersion`.
*   **Immediate Rejection**: If a peer sends a `Status` message after the handshake, `EthStreamInner` returns `EthHandshakeError::StatusNotInHandshake`.
*   **Version Gating**: If a peer negotiated `eth/68` but sends an ID reserved for `eth/69` (like `BlockRangeUpdate`), `ProtocolMessage::decode_message` returns `MessageError::Invalid`.
*   **Session Termination**: In both cases, the `ActiveSession` catches the decoding error, classifies it as a protocol breach, and terminates the connection. The `PeersManager` then penalizes the peer.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/eth-wire/src/ethstream.rs` | `EthStreamInner::decode_message` | The gatekeeper. Reads raw bytes, calls the decoder, and performs post-decode checks (like banning `Status`). |
| `crates/net/eth-wire-types/src/message.rs` | `ProtocolMessage::decode_message` | The decoder. Takes `EthVersion` as input and validates that the message ID is allowed for that version. |
| `crates/net/network/src/session/active.rs` | `ActiveSession::poll` | The event loop. It drives the stream and handles errors returned by the decoder, triggering disconnects. |
| `crates/net/eth-wire/src/errors/eth.rs` | `EthHandshakeError::StatusNotInHandshake` | The specific error thrown when a peer sends a Status message too late. |

---

### 3. Status-After-Handshake Enforcement

The enforcement happens inside `EthStreamInner`. The types `UnauthedEthStream` (handshake) and `EthStream` (active) represent the state transition. Once an `EthStream` exists, the handshake is definitionally complete.

#### Step 1: Receiving Bytes
The `ActiveSession` polls `conn.poll_next_unpin(cx)`. This calls into `EthStream::poll_next`.

#### Step 2: Decoding and Checking
Inside `EthStream::poll_next`, it calls `self.eth.decode_message(bytes)`.

*   **File**: `crates/net/eth-wire/src/ethstream.rs`
*   **Function**: `EthStreamInner::decode_message`
*   **Logic**:
    1.  It decodes the bytes into an `EthMessage` enum.
    2.  It explicitly checks if the variant is `Status`.
    ```rust
    if matches!(msg.message, EthMessage::Status(_)) {
        return Err(EthStreamError::EthHandshakeError(EthHandshakeError::StatusNotInHandshake));
    }
    ```

#### Step 3: Error Propagation
The `EthStreamError` bubbles up to `ActiveSession`.
*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `ActiveSession::poll` (handling `poll_next_unpin` result)
*   **Logic**: It matches `Err(err)` and calls `this.close_on_error(err, cx)`.

---

### 4. Wrong-Version Message Handling (eth/68 vs eth/69)

Enforcement happens inside the `eth-wire-types` crate during RLP decoding. The decoder requires the `EthVersion` to be passed in, which `EthStreamInner` holds in its state.

#### Step 1: Attempting Decode
`EthStreamInner` calls `ProtocolMessage::decode_message(self.version, &mut bytes.as_ref())`.
*   **Note**: `self.version` is the version negotiated during the handshake (e.g., `EthVersion::Eth68`).

#### Step 2: Version Validation
Inside `ProtocolMessage::decode_message`, every message ID is checked against the version.

*   **File**: `crates/net/eth-wire-types/src/message.rs`
*   **Function**: `ProtocolMessage::decode_message`
*   **Scenario**: Peer sends `BlockRangeUpdate` (ID `0x11`) but version is `Eth68`.
*   **Logic**:
    ```rust
    EthMessageID::BlockRangeUpdate => {
        if version < EthVersion::Eth69 {
            return Err(MessageError::Invalid(version, EthMessageID::BlockRangeUpdate))
        }
        EthMessage::BlockRangeUpdate(BlockRangeUpdate::decode(buf)?)
    }
    ```
    This returns `MessageError::Invalid`.

#### Step 3: Wrapping the Error
`EthStreamInner` catches `MessageError` and wraps it in `EthStreamError::InvalidMessage`.

---

### 5. Consequences & Penalties

When `ActiveSession` receives these errors, it triggers a disconnect sequence.

#### 1. Session Closure
`ActiveSession::close_on_error` sends an `ActiveSessionMessage::ClosedOnConnectionError` to the `SessionManager`.

#### 2. Peer Reporting
The `SessionManager` processes this event in `on_swarm_event`.
*   **File**: `crates/net/network/src/manager.rs`
*   **Function**: `on_swarm_event` matches `SwarmEvent::SessionClosed`.
*   **Logic**: It calls `self.swarm.state_mut().peers_mut().on_active_session_dropped(...)`.

#### 3. Scoring
The `PeersManager` determines the penalty.
*   **File**: `crates/net/network/src/peers.rs`
*   **Function**: `on_active_session_dropped` calls `on_connection_failure`.
*   **Logic**: It checks `err.is_fatal_protocol_error()`.
    *   `StatusNotInHandshake` maps to `ProtocolBreach` (via `merits_discovery_ban`).
    *   `InvalidMessage` maps to `ReputationChangeKind::Dropped` (via default fallthrough in `SessionError`).
    *   If fatal, the peer is **Banned**.
    *   If not fatal, the peer is **Slashed** (reputation lowered).

---

### 6. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `EthStreamInner` | `crates/net/eth-wire/src/ethstream.rs` | Holds the negotiated `EthVersion` and performs the `decode_message` logic. |
| `ProtocolMessage` | `crates/net/eth-wire-types/src/message.rs` | The RLP decoding logic that switches behavior based on the `version` argument. |
| `EthMessageID` | `crates/net/eth-wire-types/src/message.rs` | Enum of IDs (e.g., `BlockRangeUpdate = 0x11`). |
| `EthHandshakeError` | `crates/net/eth-wire/src/errors/eth.rs` | Error enum containing `StatusNotInHandshake`. |

---

### 7. Where to Look Next
1.  **`crates/net/eth-wire-types/src/message.rs`**: Look at `decode_message` to see the exact version checks for every message type (e.g., `GetNodeData` is rejected if version >= `Eth67`).
2.  **`crates/net/eth-wire/src/ethstream.rs`**: Look at `decode_message` to see the explicit `EthMessage::Status` check.
3.  **`crates/net/network/src/error.rs`**: To see how `EthStreamError` maps to `SessionError` and how `is_fatal_protocol_error` is implemented for different errors.

---

### 8. Suggested Follow-up Questions
*   "How does `ActiveSession` handle `Disconnect` messages sent by the peer during the active phase versus the handshake phase?"
*   "If a peer sends a message size larger than `MAX_MESSAGE_SIZE`, is that treated as a fatal protocol breach or a recoverable error?"
*   "Where are the `Capabilities` checked during the P2P `Hello` handshake before the `eth` handshake even begins?"

## Corrections / Caveats
- `EthStreamError::InvalidMessage` from wrong-version IDs is handled as a connection error. It does not emit a `ProtocolBreach` event; instead it flows through `SessionClosed` and `on_active_session_dropped` with `ReputationChangeKind::Dropped` unless other error types mark it fatal.
- `EthHandshakeError::StatusNotInHandshake` is treated as a fatal protocol error (unless the peer is trusted), which leads to removal/ban and potential discovery ban. This differs from the non-fatal `InvalidMessage` path.
- Status-after-handshake is enforced both in `EthStreamInner::decode_message` and in `ActiveSession::on_incoming_message` (defensive double check).

## Verification
- The eth handshake is completed before creating an `EthStream` and returning it from `UnauthedEthStream::handshake_without_timeout`.```61:105:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/ethstream.rs
pub async fn handshake_without_timeout<N: NetworkPrimitives>(...) -> Result<(EthStream<S, N>, UnifiedStatus), EthStreamError> {
    ...
    let their_status =
        EthereumEthHandshake(&mut self.inner).eth_handshake(status, fork_filter).await?;
    // now we can create the `EthStream` because the peer has successfully completed
    // the handshake
    let stream = EthStream::new(status.version, self.inner);
    Ok((stream, their_status))
}
```
- The session layer emits `PendingSessionEvent::Established` only after the eth handshake succeeds and an `EthStream` is created, marking the transition to active.```1123:1185:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
match handshake.handshake(&mut p2p_stream, status, fork_filter.clone(), HANDSHAKE_TIMEOUT).await {
    Ok(their_status) => {
        let eth_stream = EthStream::new(eth_version, p2p_stream);
        (eth_stream.into(), their_status)
    }
    Err(err) => { return PendingSessionEvent::Disconnected { ... } }
}
...
PendingSessionEvent::Established { conn, status, ... }
```
- `EthStreamInner::decode_message` rejects `Status` after handshake and `encode_message` prevents sending it, returning `EthHandshakeError::StatusNotInHandshake`.```131:167:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/ethstream.rs
pub fn decode_message(&self, bytes: BytesMut) -> Result<EthMessage<N>, EthStreamError> {
    ...
    let msg = match ProtocolMessage::decode_message(self.version, &mut bytes.as_ref()) { ... };
    if matches!(msg.message, EthMessage::Status(_)) {
        return Err(EthStreamError::EthHandshakeError(EthHandshakeError::StatusNotInHandshake));
    }
    Ok(msg.message)
}
pub fn encode_message(&self, item: EthMessage<N>) -> Result<Bytes, EthStreamError> {
    if matches!(item, EthMessage::Status(_)) {
        return Err(EthStreamError::EthHandshakeError(EthHandshakeError::StatusNotInHandshake));
    }
    Ok(Bytes::from(alloy_rlp::encode(ProtocolMessage::from(item))))
}
```
- Version-gated message IDs are validated in `ProtocolMessage::decode_message`; `BlockRangeUpdate` is rejected if version < eth/69 with `MessageError::Invalid`.```139:143:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire-types/src/message.rs
EthMessageID::BlockRangeUpdate => {
    if version < EthVersion::Eth69 {
        return Err(MessageError::Invalid(version, EthMessageID::BlockRangeUpdate))
    }
    EthMessage::BlockRangeUpdate(BlockRangeUpdate::decode(buf)?)
}
```
- The active session treats `EthMessage::Status` as a bad message and closes on error. Incoming stream errors (like `InvalidMessage`) also go through `close_on_error`.```218:222:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
match msg {
    message @ EthMessage::Status(_) => OnIncomingMessageOutcome::BadMessage {
        error: EthStreamError::EthHandshakeError(EthHandshakeError::StatusNotInHandshake),
        message,
    },
    ...
}
```
```714:731:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
match this.on_incoming_message(msg) {
    OnIncomingMessageOutcome::BadMessage { error, message } => {
        ... return this.close_on_error(error, cx)
    }
    ...
}
...
Err(err) => { ... return this.close_on_error(err, cx) }
```
- `close_on_error` emits `SessionClosedOnConnectionError`, which the manager maps to `SwarmEvent::SessionClosed` and then `on_active_session_dropped`.```458:465:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
fn close_on_error(&mut self, error: EthStreamError, cx: &mut Context<'_>) -> Poll<()> {
    let msg = ActiveSessionMessage::ClosedOnConnectionError { peer_id: self.remote_peer_id, remote_addr: self.remote_addr, error };
    self.terminate_message = Some((self.to_session_manager.inner().clone(), msg));
    self.poll_terminate_message(cx).expect("message is set")
}
```
```457:468:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
ActiveSessionMessage::ClosedOnConnectionError { peer_id, remote_addr, error } => {
    self.remove_active_session(&peer_id);
    Poll::Ready(SessionEvent::SessionClosedOnConnectionError { remote_addr, peer_id, error })
}
```
```164:166:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/swarm.rs
SessionEvent::SessionClosedOnConnectionError { peer_id, remote_addr, error } => {
    self.state.on_session_closed(peer_id);
    Some(SwarmEvent::SessionClosed { peer_id, remote_addr, error: Some(error) })
}
```
```843:862:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
SwarmEvent::SessionClosed { peer_id, remote_addr, error } => {
    let reason = if let Some(ref err) = error {
        self.swarm.state_mut().peers_mut().on_active_session_dropped(&remote_addr, &peer_id, err);
        err.as_disconnected()
    } else { ... };
    ...
}
```
- `on_active_session_dropped` applies `ReputationChangeKind::Dropped`, and only fatal errors are banned. `EthHandshakeError` is generally fatal; `InvalidMessage` is not marked fatal.```586:597:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
pub(crate) fn on_active_session_dropped(
    &mut self,
    remote_addr: &SocketAddr,
    peer_id: &PeerId,
    err: &EthStreamError,
) {
    self.on_connection_failure(remote_addr, peer_id, err, ReputationChangeKind::Dropped)
}
```
```635:658:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if err.is_fatal_protocol_error() {
    ...
    self.ban_peer(*peer_id);
} else {
    ...
}
```
```109:178:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/error.rs
impl SessionError for EthStreamError {
    fn is_fatal_protocol_error(&self) -> bool {
        match self {
            Self::P2PStreamError(_) => { ... }
            Self::EthHandshakeError(err) => {
                match err {
                    EthHandshakeError::NoResponse | EthHandshakeError::InvalidFork(_) => false,
                    _ => true,
                }
            }
            _ => false
        }
    }
}
```

## Actionable Pointers
- Status enforcement: `reth/crates/net/eth-wire/src/ethstream.rs` and `reth/crates/net/network/src/session/active.rs`.
- Version gating: `reth/crates/net/eth-wire-types/src/message.rs`.
- Penalty logic: `reth/crates/net/network/src/error.rs` and `reth/crates/net/network/src/peers.rs`.
