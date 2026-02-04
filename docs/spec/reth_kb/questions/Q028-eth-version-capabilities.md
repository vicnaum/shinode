# Q028 - EthVersion + Capabilities Negotiation and Access

Status: verified

## Question
Where is the negotiated EthVersion/capabilities stored and how do callers access it per peer?
I want the flow from handshake -> session state -> NetworkEvent/SessionInfo, and how this is used to select eth/69 vs eth/70 requests.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q028):
Where is the negotiated EthVersion/capabilities stored and how do callers access it per peer?
I want the flow from handshake -> session state -> NetworkEvent/SessionInfo, and how this is used to select eth/69 vs eth/70 requests.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Show where EthVersion is negotiated and stored.
3. Show how callers obtain the version/capabilities for a peer session.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Negotiation flow (step-by-step)
4. Where the data is stored (types/fields)
5. How callers access it (NetworkEvent/SessionInfo paths)
6. "Where to look next"
7. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-network.xml` pack, here is the architectural map of how `EthVersion` and capabilities are negotiated, stored, and accessed in Reth.

### 1. Summary
*   **Negotiation in Handshake**: The version is negotiated during the P2P handshake (`Hello` message) via `SharedCapabilities`. The highest shared version of the "eth" protocol is selected.
*   **Storage in Session Handles**: Once established, the negotiated `EthVersion` and `Capabilities` are stored in the `ActiveSessionHandle` (held by the `NetworkManager`) and the `ActiveSession` (the worker task managing the connection).
*   **Exposure via Events**: When a session starts, the `NetworkManager` emits a `NetworkEvent::ActivePeerSession`, which contains a `SessionInfo` struct carrying the version and capabilities. Callers (like the Sync pipeline) consume this event.
*   **Version-Agnostic Callers**: Callers typically send version-agnostic requests (e.g., `GetReceipts`). The `ActiveSession` task uses the stored `EthVersion` to automatically upgrade requests (e.g., to `GetReceipts70`) before putting bytes on the wire.
*   **Direct API Access**: External components can query peer details via `NetworkHandle::get_peer_info_by_id`, which retrieves the version from the active session map.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/eth-wire/src/capability.rs` | `SharedCapabilities::try_new` | The logic that intersects local and remote capabilities to find the highest shared version. |
| `crates/net/network/src/session/handle.rs` | `ActiveSessionHandle` | The struct held by `NetworkManager` storing metadata for every active connection. |
| `crates/net/network/src/session/active.rs` | `ActiveSession` | The background task that holds the connection state and applies version-specific logic. |
| `crates/net/network-api/src/events.rs` | `SessionInfo` | The public struct emitted in network events containing `eth_version` and `capabilities`. |
| `crates/net/eth-wire-types/src/message.rs` | `EthMessage::map_versioned` | The function that transforms a generic message into a version-specific one (e.g. eth/69 vs eth/70). |

---

### 3. Negotiation Flow

#### Step 1: P2P Hello Handshake
During the P2P handshake, capabilities are exchanged.
*   **File**: `crates/net/eth-wire/src/p2pstream.rs`
*   **Function**: `UnauthedP2PStream::handshake` calls `SharedCapabilities::try_new`.
*   **Logic**: It iterates `local_protocols` and `peer_capabilities`. For the "eth" protocol, it selects the numerically highest version supported by both parties.

#### Step 2: Authentication Wrapper
The handshake result is bubbled up to the session authentication logic.
*   **File**: `crates/net/network/src/session/mod.rs`
*   **Function**: `authenticate_stream`
*   **Logic**: `let eth_version = p2p_stream.shared_capabilities().eth_version()?`. This confirms the negotiated version (e.g., `Eth68`).

#### Step 3: Session Creation
The `SessionManager` creates the session objects with the negotiated version.
*   **File**: `crates/net/network/src/session/mod.rs`
*   **Function**: `poll` (handling `PendingSessionEvent::Established`)
*   **Logic**: It instantiates `ActiveSession` (the worker) and `ActiveSessionHandle` (the state handle), passing `version` and `capabilities` to both.

---

### 4. Storage of Version/Capabilities

The data is stored in two places: the manager's state and the worker's state.

#### A. In `ActiveSessionHandle` (Owned by `NetworkManager`)
This allows the manager to answer API queries about peers.
*   **File**: `crates/net/network/src/session/handle.rs`
*   **Fields**:
    ```rust
    pub struct ActiveSessionHandle<N: NetworkPrimitives> {
        pub(crate) version: EthVersion,
        pub(crate) capabilities: Arc<Capabilities>,
        // ...
    }
    ```

#### B. In `ActiveSession` (The Worker Task)
This allows the worker to format messages correctly before sending.
*   **File**: `crates/net/network/src/session/active.rs`
*   **Access**: `self.conn.version()` delegates to the underlying `EthStreamInner`.
*   **Fields**:
    ```rust
    pub(crate) struct ActiveSession<N: NetworkPrimitives> {
        pub(crate) conn: EthRlpxConnection<N>, // Holds version inside
        pub(crate) remote_capabilities: Arc<Capabilities>,
        // ...
    }
    ```

---

### 5. Access and Usage

#### A. Caller Access via Events (`SessionInfo`)
When a peer connects, the network emits an event.
*   **File**: `crates/net/network/src/manager.rs`
*   **Flow**: `SwarmEvent::SessionEstablished` -> `NetworkEvent::ActivePeerSession`.
*   **Data**: The event contains `SessionInfo` (defined in `crates/net/network-api/src/events.rs`), which exposes:
    *   `pub version: EthVersion`
    *   `pub capabilities: Arc<Capabilities>`

#### B. Caller Access via API
A component can query a specific peer.
*   **File**: `crates/net/network/src/network.rs`
*   **Function**: `NetworkHandle::get_peer_info_by_id`.
*   **Flow**: Sends message to Manager -> Manager looks up `ActiveSessionHandle` -> Returns `PeerInfo`.

#### C. Internal Usage: Request Upgrading (`map_versioned`)
This is how Reth handles `eth/69` vs `eth/70` transparently. Callers send a generic `GetReceipts` request. The session layer upgrades it if the peer supports a newer version.

*   **File**: `crates/net/network/src/session/active.rs`
*   **Function**: `on_internal_peer_request`
*   **Code**:
    ```rust
    let msg = request.create_request_message(request_id).map_versioned(self.conn.version());
    ```
*   **Logic (`map_versioned`)**:
    *   **File**: `crates/net/eth-wire-types/src/message.rs`
    *   **Implementation**:
        ```rust
        pub fn map_versioned(self, version: EthVersion) -> Self {
            if version >= EthVersion::Eth70 {
                // Upgrades generic GetReceipts to GetReceipts70
                return match self {
                    Self::GetReceipts(pair) => { ... Self::GetReceipts70(...) }
                    other => other,
                }
            }
            self
        }
        ```

---

### 6. Where to Look Next
1.  **`crates/net/eth-wire-types/src/message.rs`**: Look at `map_versioned` to see exactly which message types change based on the version.
2.  **`crates/net/eth-wire/src/capability.rs`**: Look at `shared_capability_offsets` to see the logic for how Reth selects the highest version when multiple are available.
3.  **`crates/net/network/src/session/active.rs`**: Look at `on_incoming_message` to see how incoming messages are handled differently based on `self.conn.version()` (e.g., `Receipts` vs `Receipts69`).

---

### 7. Suggested Follow-up Questions
*   "How does `EthStreamInner` enforce that `Status` messages are not sent after the handshake is complete?"
*   "What happens if a peer announces `eth/68` in the Hello message but sends an `eth/69` specific message ID?"
*   "How does `TransactionsManager` use `capabilities` to filter which transaction types (EIP-4844) are broadcast to specific peers?"

## Verification
- `SharedCapabilities::try_new` is called during the p2p hello handshake, and capability intersection picks the highest shared version per capability name.```156:169:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/p2pstream.rs
let capability_res =
    SharedCapabilities::try_new(hello.protocols, their_hello.capabilities.clone());
```
```285:296:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire/src/capability.rs
// If multiple versions are shared of the same (equal name) capability, the numerically
// highest wins, others are ignored
if shared_capabilities
    .get(&peer_capability.name)
    .is_none_or(|v| peer_capability.version > v.version)
{
    shared_capabilities.insert(
        peer_capability.name.clone(),
        ProtoVersion { version: peer_capability.version, messages },
    );
}
```
- `authenticate_stream` reads the negotiated eth version from `shared_capabilities()` and applies it to the status handshake.```1103:1117:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
let eth_version = match p2p_stream.shared_capabilities().eth_version() { ... };
status.set_eth_version(eth_version);
```
- `ActiveSessionHandle` stores negotiated `EthVersion` and `Capabilities`, and `peer_info` exposes both to callers.```55:150:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/handle.rs
pub struct ActiveSessionHandle<N: NetworkPrimitives> {
    pub(crate) version: EthVersion,
    pub(crate) capabilities: Arc<Capabilities>,
    // ...
}
...
pub(crate) fn peer_info(&self, record: &NodeRecord, kind: PeerKind) -> PeerInfo {
    PeerInfo { capabilities: self.capabilities.clone(), eth_version: self.version, ... }
}
```
- `SessionManager` constructs `ActiveSession` and `ActiveSessionHandle` with the negotiated version and capabilities, and emits `SessionEstablished` with both.```528:611:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
let messages = PeerRequestSender::new(peer_id, to_session_tx);
let version = conn.version();
let session = ActiveSession { remote_capabilities: Arc::clone(&capabilities), ... };
let handle = ActiveSessionHandle { version, capabilities: Arc::clone(&capabilities), ... };
...
Poll::Ready(SessionEvent::SessionEstablished { version, capabilities, messages, ... })
```
- `SwarmEvent::SessionEstablished` carries `version` and `capabilities`, and `NetworkManager` forwards them in `SessionInfo` via `NetworkEvent::ActivePeerSession`.```380:388:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/swarm.rs
SessionEstablished { capabilities: Arc<Capabilities>, version: EthVersion, messages: PeerRequestSender<PeerRequest<N>>, ... }
```
```777:831:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
SwarmEvent::SessionEstablished { capabilities, version, messages, ... } => {
    let session_info = SessionInfo { capabilities, version, ... };
    self.event_sender.notify(NetworkEvent::ActivePeerSession { info: session_info, messages });
}
```
- Callers can query per-peer info via `NetworkHandle::get_peer_by_id`; the manager builds this from `ActiveSessionHandle` using `peer_info`.```323:326:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/network.rs
let _ = self.manager().send(NetworkHandleMessage::GetPeerInfoById(peer_id, tx));
```
```1019:1026:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
fn get_peer_info_by_id(&self, peer_id: PeerId) -> Option<PeerInfo> {
    self.swarm.sessions().active_sessions().get(&peer_id).and_then(|session| {
        self.swarm.state().peers().peer_by_id(peer_id).map(|(record, kind)| session.peer_info(&record, kind))
    })
}
```
- `ActiveSession` uses `conn.version()` to map outgoing requests and uses the version to choose the inbound receipts response variant.```317:323:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
let msg = request.create_request_message(request_id).map_versioned(self.conn.version());
```
```266:270:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/active.rs
EthMessage::GetReceipts(req) => {
    if self.conn.version() >= EthVersion::Eth69 {
        on_request!(req, Receipts69, GetReceipts69)
    } else {
        on_request!(req, Receipts, GetReceipts)
    }
}
```

## Corrections / Caveats
- The request upgrade path is only for eth/70. For eth/69, callers must explicitly send `PeerRequest::GetReceipts69` to accept a `Receipts69` response; `map_versioned` does not upcast for eth/69.
- The external API method is `NetworkHandle::get_peer_by_id` (not `get_peer_info_by_id`). The manager resolves this using `ActiveSessionHandle::peer_info`.

## Actionable Pointers
- Capability negotiation and eth version selection: `reth/crates/net/eth-wire/src/p2pstream.rs` and `reth/crates/net/eth-wire/src/capability.rs`.
- Session storage and exposure: `reth/crates/net/network/src/session/mod.rs`, `reth/crates/net/network/src/session/handle.rs`, `reth/crates/net/network/src/manager.rs`.
- Request mapping by version: `reth/crates/net/network/src/session/active.rs` and `reth/crates/net/eth-wire-types/src/message.rs`.
