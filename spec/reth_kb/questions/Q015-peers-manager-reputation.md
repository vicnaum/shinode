# Q015 - PeersManager and Reputation

Status: verified

## Question
How does PeersManager work and how is peer reputation scored or updated?

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q015):
How does PeersManager work and how is peer reputation scored or updated?
I want the main types, event flow, the reputation model, and where penalties/rewards are applied.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe the event flow from network events into reputation changes.
3. Explain how bans/penalties are enforced.
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
Based on the `context-network.xml` pack, here is the architectural breakdown of the `PeersManager` and the reputation system in Reth.

### 1. Summary
*   **Central State Machine**: `PeersManager` is the state machine responsible for tracking all known peers, their connection state, and their reputation. It does not handle I/O directly but emits `PeerAction`s.
*   **Integer-Based Reputation**: Reputation is a signed integer (`i32`). Peers start at `0`. Bad behavior subtracts points; time spent connected adds points (decaying penalties).
*   **Weighted Offenses**: Different offenses (Bad Message, Timeout, Failed to Connect) have specific weights defined in `ReputationChangeWeights`.
*   **Threshold Banning**: If a peer's reputation drops below `BANNED_REPUTATION`, they are disconnected and banned.
*   **Backoff Mechanism**: For non-fatal errors (like timeouts), peers are placed in a "backoff" state (temporary timeout) rather than being immediately banned.
*   **Trusted Peers**: Trusted peers (configured via CLI/Config) are exempt from most reputation slashing and are prioritized for connection slots.
*   **BanList Integration**: The manager checks a `BanList` (IPs and PeerIds) before allowing incoming connections or dialing out.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/peers.rs` | `PeersManager` | The core struct managing the collection of peers, slots, and reputation logic. |
| `crates/net/network-types/src/peers/mod.rs` | `Peer` | Represents the state of a single peer (address, reputation score, connection state). |
| `crates/net/network-types/src/peers/reputation.rs` | `ReputationChangeKind` | Enum defining all possible reasons for reputation changes (e.g., `BadMessage`, `Timeout`). |
| `crates/net/network-api/src/test_utils/peers_manager.rs` | `PeersHandle` | A handle used by other parts of the system (like Sync or RPC) to report reputation changes manually. |

---

### 3. Algorithm Flow: Reputation & Banning

#### Step 1: Triggering a Reputation Change
A component (e.g., the Sync Pipeline or Session Manager) detects an issue.
*   **Example**: `StateFetcher` receives a bad block header.
*   **Action**: It calls `peers_handle.reputation_change(peer_id, ReputationChangeKind::BadMessage)`.

#### Step 2: Applying the Change
The `PeersManager` processes the change in `apply_reputation_change`.
*   **File**: `crates/net/network/src/peers.rs`
*   **Logic**:
    1.  It retrieves the `Peer` struct.
    2.  It calculates the numeric penalty using `reputation_weights.change(kind)`.
    3.  **Trusted Peer Exception**: If the peer is `Trusted` or `Static`, severe penalties (like `Dropped` or `BadAnnouncement`) are ignored, and others are capped at `MAX_TRUSTED_PEER_REPUTATION_CHANGE` to prevent banning.

#### Step 3: Updating the Score
The score is updated in `Peer::apply_reputation`.
*   **File**: `crates/net/network-types/src/peers/mod.rs`
*   **Logic**: `self.reputation = self.reputation.saturating_add(reputation_change)`.

#### Step 4: Determining the Outcome
The `apply_reputation` method returns a `ReputationChangeOutcome`.
*   If `reputation < BANNED_REPUTATION` (approx -51,200): Returns `DisconnectAndBan`.
*   If currently banned but score improved: Returns `Unban`.
*   Otherwise: Returns `None`.

#### Step 5: Enforcement (`PeersManager`)
Based on the outcome, `PeersManager` queues a `PeerAction`.
*   **Disconnect**: `PeerAction::Disconnect` is queued, which `NetworkManager` picks up to kill the session.
*   **Ban**: `ban_list.ban_peer_until` is called. Future connection attempts from this PeerID/IP will be rejected in `on_incoming_pending_session` or `add_and_connect`.

#### Step 6: Recovery (The "Tick")
`PeersManager` runs a `tick()` method on every poll.
*   **Logic**: For every peer currently connected (`is_connected()`), if their reputation is below 0, it increments the score based on seconds elapsed. This allows peers to recover from minor infractions by staying useful.

---

### 4. Key Abstractions & Types

#### `ReputationChangeWeights`
*   **File**: `crates/net/network-types/src/peers/reputation.rs`
*   **Description**: Configuration struct mapping abstract offenses to concrete integer penalties.
*   **Defaults**:
    *   `BadMessage`: -16,384 (`16 * REPUTATION_UNIT`)
    *   `Timeout`: -4,096
    *   `FailedToConnect`: -25,600

#### `PeerConnectionState`
*   **File**: `crates/net/network-types/src/peers/state.rs`
*   **Description**: Enum tracking if a peer is `Idle`, `PendingOut`, `In` (connected via inbound), or `Out` (connected via outbound). Used to manage slot limits.

#### `PeerAction`
*   **File**: `crates/net/network/src/peers.rs`
*   **Description**: Events emitted by `PeersManager::poll` to instruct the `NetworkManager` to perform I/O.
*   **Variants**: `Connect`, `Disconnect`, `BanPeer`, `DiscoveryBanIp`.

#### `BackoffKind`
*   **File**: `crates/net/network-types/src/backoff.rs`
*   **Description**: Differentiates between `Low`, `Medium`, and `High` severity backoffs. Used when a peer disconnects non-fatally (e.g., `TooManyPeers`) to determine how long to wait before redialing.

---

### 5. Where to Look Next
1.  **`crates/net/network/src/peers.rs` -> `apply_reputation_change`**: This is the central logic for deciding if a peer survives an error.
2.  **`crates/net/network/src/swarm.rs` -> `on_session_closed`**: To see how the network stack reports session drops to the `PeersManager`.
3.  **`crates/net/network/src/fetch/mod.rs`**: To see how the downloaders report `BadResponse` which eventually triggers reputation changes.

---

### 6. Suggested Follow-up Questions
*   "How does the `TrustedPeersResolver` work to keep DNS-based trusted peer addresses up to date?"
*   "What is the difference between `soft_remove_node` and `remove_node` in `Discv4Service`?"
*   "How does `NetworkState` use `PeerAction` to trigger actual socket disconnections?"

## Verification
- PeersManager buffers `PeerAction` and returns actions on `poll`, acting as a state machine rather than doing I/O directly.```41:64:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
/// Maintains the state of _all_ the peers known to the network.
/// From this type, connections to peers are established or disconnected, see [`PeerAction`].
...
queued_actions: VecDeque<PeerAction>,
```
```1038:1064:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<PeerAction> {
    if let Some(action) = self.queued_actions.pop_front() { return Poll::Ready(action) }
    ...
}
```
- Reputation is `i32` with default 0, and the banned threshold is `BANNED_REPUTATION` (50 * -1024 = -51200).```3:11:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-types/src/peers/reputation.rs
pub const DEFAULT_REPUTATION: Reputation = 0;
const REPUTATION_UNIT: i32 = -1024;
pub const BANNED_REPUTATION: i32 = 50 * REPUTATION_UNIT;
```
- Weighted reputation changes come from `ReputationChangeWeights::change`.```111:168:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-types/src/peers/reputation.rs
pub struct ReputationChangeWeights { ... }
pub fn change(&self, kind: ReputationChangeKind) -> ReputationChange { ... }
```
- `Peer::apply_reputation` uses `saturating_add` and returns a ban/disconnect outcome.```85:110:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-types/src/peers/mod.rs
self.reputation = previous.saturating_add(reputation);
...
return ReputationChangeOutcome::DisconnectAndBan
```
- `PeersManager::apply_reputation_change` handles trusted/static peers with exemptions and caps, then enforces Ban/Disconnect outcomes.```466:516:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if peer.is_trusted() || peer.is_static() { ... return ... }
if reputation_change < MAX_TRUSTED_PEER_REPUTATION_CHANGE { reputation_change = MAX_TRUSTED_PEER_REPUTATION_CHANGE; }
...
match outcome { ReputationChangeOutcome::Ban => self.ban_peer(*peer_id), ... }
```
- Tick-based reputation recovery only increments connected peers whose reputation is below default.```438:457:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
for peer in self.peers.iter_mut().filter(|(_, peer)| peer.state.is_connected()) {
    if peer.1.reputation < DEFAULT_REPUTATION { peer.1.reputation += secs_since_last_tick; }
}
```
- Backoff is applied on non-fatal connection failures when `SessionError::should_backoff` returns a severity.```626:716:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if let Some(kind) = err.should_backoff() { ... backoff_until = Some(...) }
```
```83:106:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/error.rs
fn should_backoff(&self) -> Option<BackoffKind>;
```
- Trusted/static peers are prioritized for outbound slots in `best_unconnected`.```936:969:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
/// Peers that are trusted or static are prioritized ...
if maybe_better.1.is_trusted() || maybe_better.1.is_static() { return Some(...) }
```
- BanList checks guard inbound and outbound paths.```248:260:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if self.ban_list.is_banned_ip(&addr) { return Err(InboundConnectionError::IpBanned) }
```
```879:888:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if self.ban_list.is_banned(&peer_id, &ip_addr) { return }
```
- PeersHandle exposes `reputation_change` to send commands to the manager.```20:50:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/test_utils/peers_manager.rs
pub struct PeersHandle { ... }
pub fn reputation_change(&self, peer_id: PeerId, kind: ReputationChangeKind) {
    self.send(PeerCommand::ReputationChange(peer_id, kind));
}
```

## Corrections / Caveats
- Trusted/static peers are exempt only for specific change kinds (Dropped, BadAnnouncement, Timeout, AlreadySeenTransaction) and otherwise have slashing capped; not "most" reputation changes.
- "PeersHandle used by Sync/RPC" is not shown in this pack; `PeersHandle` is defined in `network-api` test_utils and provides a command channel.
- PeerAction variants listed are not exhaustive; see `PeerAction` in `peers.rs` for all actions.
- Backoff is tied to connection/session errors via `SessionError::should_backoff`; it is not applied to all reputation changes.

## Actionable Pointers
- Core reputation flow and enforcement: `reth/crates/net/network/src/peers.rs`.
- Reputation weights/constants: `reth/crates/net/network-types/src/peers/reputation.rs`.
- Peer state changes and ban outcomes: `reth/crates/net/network-types/src/peers/mod.rs`.
- Backoff severity model: `reth/crates/net/network-types/src/backoff.rs`.
