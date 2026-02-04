# Q016 - Connection Limits, Dialing, and Backoff

Status: verified

## Question
How are connection limits, dialing, and backoff controls implemented?

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q016):
How are connection limits, dialing, and backoff controls implemented?
I want the main types, how inbound/outbound slots are tracked, how peers are selected for dialing, and how backoff durations are computed/applied.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe the control flow for inbound accept, outbound dial scheduling, and backoff release.
3. Explain how trusted/static peers affect slot allocation or dialing priority.
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
Based on the `context-network.xml` pack, here is the implementation analysis of connection limits, dialing logic, and backoff controls in Reth.

### 1. Summary
*   **Central Control**: The `PeersManager` is the authoritative state machine for deciding *who* to connect to and *when*. It manages connection slots via `ConnectionInfo`.
*   **Slot Tracking**: Inbound and outbound slots are tracked separately. There are limits for "pending" (handshaking) connections and "active" (established) sessions.
*   **Dialing Strategy**: Dialing is interval-based (`refill_slots_interval`). The strategy prioritizes **Trusted** and **Static** peers first, followed by peers with the highest **Reputation** score.
*   **Backoff Logic**: Backoffs are deterministic based on error severity (`BackoffKind`). "Severe" errors increment a counter on the peer, causing the backoff duration to scale linearly (duration * counter) up to a max cap.
*   **Trusted Bypass**: Trusted peers have special privileges; they are prioritized during dialing and can sometimes bypass inbound connection limits if the node is in a specific restricted mode.
*   **Lazy Cleanup**: Backed-off peers are stored in a `HashMap` and released via a periodic tick (`release_interval`) in the `PeersManager`, rather than individual timers.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/peers.rs` | `PeersManager` | The core state machine managing limits, backoffs, and dial selection. |
| `crates/net/network/src/peers.rs` | `ConnectionInfo` | Tracks integer counts of In/Out/Pending connections against configured limits. |
| `crates/net/network-types/src/peers/config.rs` | `PeerBackoffDurations` | Defines base durations for Low, Medium, and High severity backoffs. |
| `crates/net/network-types/src/backoff.rs` | `BackoffKind` | Enum classifying error severity (Low, Medium, High). |
| `crates/net/network/src/session/mod.rs` | `SessionManager` | Executes the physical dial logic based on `PeersManager` commands. |

---

### 3. Algorithm Flow

#### A. Inbound Connection Acceptance
When a new TCP stream arrives at the `Swarm`, it asks the `PeersManager` if it can accept.

1.  **Trigger**: `Swarm` calls `sessions.on_incoming`.
2.  **Check**: `PeersManager::on_incoming_pending_session` is called.
3.  **Validation**:
    *   **IP Filter**: Checks `IpFilter` and `BanList`.
    *   **Capacity**: Calls `connection_info.has_in_capacity()`.
    *   **Trusted Exception**: If at capacity, it checks if there are `idle` trusted peers. If so, it allows the connection (up to a secondary limit) to ensure trusted peers aren't locked out by random peers (`crates/net/network/src/peers.rs`).
4.  **Throttling**: Applies `throttle_incoming_ip` to rate-limit the specific IP temporarily.
5.  **Result**: If successful, `inc_pending_in()` is called, and the session starts.

#### B. Outbound Dialing (Refill Slots)
The node proactively fills outbound slots via the `PeersManager::poll` loop.

1.  **Tick**: `refill_slots_interval` ticks.
2.  **Capacity Check**: Calls `fill_outbound_slots`. Checks `connection_info.has_out_capacity()` (pending + active < max).
3.  **Selection**: Calls `best_unconnected()` to pick a peer.
    *   **Filter**: Must not be `backed_off`, `banned`, or already `connected`.
    *   **Priority 1**: If a **Trusted** or **Static** peer is found, return immediately.
    *   **Priority 2**: Sort remaining peers by **Reputation** (highest first).
4.  **Action**: Updates peer state to `PendingOut`, increments `num_pending_out`, and queues a `PeerAction::Connect`.
5.  **Execution**: `NetworkManager` receives the action and calls `dial_outbound` on the `SessionManager`.

#### C. Backoff Controls
When a connection fails (e.g., handshake timeout or TCP error):

1.  **Classification**: The error is mapped to a `BackoffKind` (Low, Medium, High).
    *   *Low*: Timouts, already connected.
    *   *Medium*: Malformed messages, logic errors.
    *   *High*: Protocol breaches, useless peer.
2.  **Calculation**: `PeersManager` calculates the duration.
    *   If `kind.is_severe()` (Medium/High), increment `peer.severe_backoff_counter`.
    *   Duration = `base_duration * counter`.
    *   Max Duration is capped by `backoff_durations.max`.
3.  **Application**: The peer is marked `backed_off = true` and added to `backed_off_peers` map with an expiry timestamp.
4.  **Release**:
    *   `PeersManager::poll` checks `release_interval` (default ~30s).
    *   Iterates `backed_off_peers`. If `now > expiry`, sets `peer.backed_off = false` and removes from map.

---

### 4. Key Abstractions & Types

#### `ConnectionInfo`
*   **Location**: `crates/net/network/src/peers.rs`
*   **Purpose**: Tracks usage against `ConnectionsConfig`.
*   **Logic**: Maintains four counters: `num_inbound`, `num_outbound`, `num_pending_in`, `num_pending_out`.
    *   *Note*: Pending outbound connections count towards the total outbound limit to prevent over-dialing.

#### `PeerBackoffDurations`
*   **Location**: `crates/net/network-types/src/peers/config.rs`
*   **Defaults**:
    *   `Low`: 30s (e.g., timeout).
    *   `Medium`: 3m (e.g., logic error).
    *   `High`: 15m (e.g., protocol violation).
    *   `Max`: 1h.

#### `PeerConnectionState`
*   **Location**: `crates/net/network-types/src/peers/state.rs`
*   **States**:
    *   `Idle`: Ready to be dialed or backoff.
    *   `PendingOut`: Currently dialing.
    *   `In`/`Out`: Active session established.
    *   `DisconnectingIn`/`DisconnectingOut`: Teardown phase.

#### `ReputationChangeWeights`
*   **Location**: `crates/net/network-types/src/peers/reputation.rs`
*   **Logic**: Defines how much to penalize a peer for specific errors (e.g., `BadMessage` = -16k). Used in conjunction with backoff for connection failures.

---

### 5. Where to Look Next
1.  **`crates/net/network/src/peers.rs` - `fill_outbound_slots`**: The exact logic for prioritizing trusted peers over high-reputation peers.
2.  **`crates/net/network/src/peers.rs` - `on_connection_failure`**: How errors are converted into backoff timestamps.
3.  **`crates/net/network/src/session/mod.rs` - `dial_outbound`**: The physical act of creating the TCP socket, which consumes the "slots" allocated by `PeersManager`.

### 6. Suggested Follow-up Questions
*   "How does the `TrustedPeersResolver` update the IP addresses of trusted peers if they are defined by DNS names?"
*   "What happens to the reputation of a peer if it is backed off multiple times in a row?"
*   "How does `NetworkConnectionState` (Active vs Hibernate) affect the `fill_outbound_slots` logic?"

## Verification
- Connection limits and pending/active counts live in `ConnectionInfo`, with outbound capacity gated by both pending-dial and active limits.```1111:1138:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
pub struct ConnectionInfo { ... }
const fn has_out_capacity(&self) -> bool {
    self.num_pending_out < self.config.max_concurrent_outbound_dials &&
        self.num_outbound < self.config.max_outbound
}
```
- Inbound accept checks IP filter and ban list, then applies capacity and a special trusted-peers allowance, and increments pending inbound.```245:297:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if !self.ip_filter.is_allowed(&addr) { ... }
if self.ban_list.is_banned_ip(&addr) { ... }
if !self.connection_info.has_in_capacity() { ... trusted_peer_ids ... }
...
self.connection_info.inc_pending_in();
```
- Dial scheduling runs on `refill_slots_interval`, using `fill_outbound_slots` and `best_unconnected` (trusted/static first, then highest reputation).```1090:1092:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
while self.refill_slots_interval.poll_tick(cx).is_ready() {
    self.fill_outbound_slots();
}
```
```936:1004:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
fn best_unconnected(&mut self) -> Option<(PeerId, &mut Peer)> { ... }
fn fill_outbound_slots(&mut self) { ... PeerAction::Connect { ... } }
```
- Backoff durations are defined by `PeerBackoffDurations` and capped by `max`, with linear scaling via the backoff counter.```30:70:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-types/src/peers/config.rs
pub struct PeerBackoffDurations { pub low: Duration, pub medium: Duration, pub high: Duration, pub max: Duration }
pub fn backoff_until(&self, kind: BackoffKind, backoff_counter: u8) -> Instant {
    let backoff_time = self.backoff(kind);
    let backoff_time = backoff_time + backoff_time * backoff_counter as u32;
    now + backoff_time.min(self.max)
}
```
- Non-fatal connection failures apply backoff based on `SessionError::should_backoff`, with severe kinds incrementing `severe_backoff_counter`.```626:716:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if let Some(kind) = err.should_backoff() { ... if kind.is_severe() { peer.severe_backoff_counter = ... } ... }
```
```83:106:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/error.rs
fn should_backoff(&self) -> Option<BackoffKind>;
```
- Backoff release and ban evictions happen on `release_interval` ticks.```1066:1087:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if self.release_interval.poll_tick(cx).is_ready() { ... self.backed_off_peers.retain(...); }
```
- Session dialing happens in `SessionManager::dial_outbound`, and Swarm routes connect actions to it.```306:345:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/session/mod.rs
pub fn dial_outbound(&mut self, remote_addr: SocketAddr, remote_peer_id: PeerId) { ... }
```
```228:233:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/swarm.rs
StateAction::Connect { remote_addr, peer_id } => {
    self.dial_outbound(remote_addr, peer_id);
}
```

## Corrections / Caveats
- Outbound capacity does not use "pending + active < max"; it separately enforces `max_concurrent_outbound_dials` and `max_outbound`.
- Backoff duration is `base * (1 + counter)` capped by `max`, not `base * counter`.
- The "trusted bypass inbound limits" logic allows extra pending inbound slots when trusted peers are configured; it does not check the incoming peer's trust at that stage.
- The specific error-to-BackoffKind mapping lives in `SessionError::should_backoff` and is more detailed than the summary. See `network/src/error.rs`.
- The exact mapping from `PeerAction::Connect` to `StateAction::Connect` is handled in network state/manager code outside these snippets.

## Actionable Pointers
- Slot counters and limits: `reth/crates/net/network/src/peers.rs` (`ConnectionInfo`).
- Backoff durations and scaling: `reth/crates/net/network-types/src/peers/config.rs`.
- Dial selection priority: `reth/crates/net/network/src/peers.rs` (`best_unconnected`, `fill_outbound_slots`).
- Outbound dial execution: `reth/crates/net/network/src/session/mod.rs` and `reth/crates/net/network/src/swarm.rs`.
