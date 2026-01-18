# session

## Purpose
Session lifecycle management for the network: authenticates peers via RLPx/ECIES, maintains pending and active sessions, dispatches/receives protocol messages, enforces timeouts, and tracks per-peer block range announcements (eth/69).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Session manager and handshake orchestration: accepts inbound/outbound connections, performs RLPx + eth handshakes, transitions pending -> active sessions, and emits `SessionEvent`s for the rest of the network stack.
- **Key items**: `SessionManager`, `SessionId`, `SessionEvent`, `PendingSessionHandshakeError`, `ExceedsSessionLimit`, `pending_session_with_timeout()`, `start_pending_incoming_session()`, `start_pending_outbound_session()`, `authenticate_stream()`
- **Interactions**: Spawns `ActiveSession` tasks (`active.rs`), wraps connections using `EthRlpxConnection` (`conn.rs`), uses `SessionCounter` (`counter.rs`) and handle/events (`handle.rs`), propagates `BlockRangeInfo` (`types.rs`) for eth/69 range updates.
- **Knobs / invariants**: Enforces session limits, handshake timeouts, and protocol breach timeouts; optionally negotiates extra RLPx subprotocols; caps concurrent graceful disconnects.

### `active.rs`
- **Role**: Active session state machine that drives a single peer connection: handles inbound messages, internal requests, outgoing responses/broadcasts, and disconnect/error paths with backpressure and timeout logic.
- **Key items**: `ActiveSession`, `QueuedOutgoingMessages`, `OutgoingMessage`, `RANGE_UPDATE_INTERVAL`, `MAX_QUEUED_OUTGOING_RESPONSES`, `InflightRequest`, `ReceivedRequest`, `calculate_new_timeout()`
- **Interactions**: Sends `ActiveSessionMessage` back to `SessionManager`; consumes `PeerRequest`/`PeerMessage`; uses `EthRlpxConnection` to read/write `EthMessage`.
- **Knobs / invariants**: Updates internal request timeout via RTT sampling; triggers protocol-breach errors after prolonged timeouts; throttles reads when queued responses exceed limits; emits `BlockRangeUpdate` periodically for eth/69+.

### `conn.rs`
- **Role**: Connection wrapper that abstracts ETH-only vs ETH+satellite (multiplexed) streams and exposes a uniform `Stream`/`Sink` interface.
- **Key items**: `EthRlpxConnection`, `EthPeerConnection`, `EthSatelliteConnection`, `start_send_broadcast()`, `start_send_raw()`
- **Interactions**: Used by `SessionManager` and `ActiveSession` to send/receive `EthMessage` across ECIES/P2P streams.
- **Knobs / invariants**: Boxes underlying streams to keep type size manageable.

### `counter.rs`
- **Role**: Tracks pending/active inbound/outbound session counts and enforces configured limits.
- **Key items**: `SessionCounter`, `ensure_pending_inbound()`, `ensure_pending_outbound()`, `inc_active()`, `dec_active()`
- **Interactions**: Used by `SessionManager` when accepting/dialing sessions.

### `handle.rs`
- **Role**: Session handle types and command/event enums for pending and active session coordination.
- **Key items**: `PendingSessionHandle`, `ActiveSessionHandle`, `PendingSessionEvent`, `SessionCommand`, `ActiveSessionMessage`
- **Interactions**: `ActiveSessionHandle::disconnect()` sends commands to the session task; `peer_info()` builds `PeerInfo`.

### `types.rs`
- **Role**: Shared range information for eth/69 block-range announcements.
- **Key items**: `BlockRangeInfo`, `BlockRangeInfo::update()`, `BlockRangeInfo::to_message()`
- **Interactions**: Used by `SessionManager` and `ActiveSession` to advertise and track range updates.

## End-to-end flow (high level)
- `SessionManager` accepts inbound TCP connections or dials outbound peers, enforcing session limits via `SessionCounter`.
- Pending sessions perform ECIES auth, hello exchange, and eth status handshake; extra subprotocols are negotiated as needed.
- On success, the manager creates an `EthRlpxConnection`, spawns an `ActiveSession`, and emits `SessionEstablished`.
- `ActiveSession` multiplexes manager commands, internal requests, and wire messages, buffering outbound traffic with backpressure.
- Incoming requests from the peer are forwarded as `PeerMessage` events and resolved via response channels.
- Timeouts and protocol breaches trigger disconnects; graceful disconnects are capped by `DisconnectionsCounter`.
- For eth/69+, range updates are periodically sent and tracked via `BlockRangeInfo`.

## Key APIs (no snippets)
- `SessionManager`, `SessionEvent`, `SessionCommand`
- `ActiveSession`, `ActiveSessionHandle`, `PendingSessionHandle`
- `EthRlpxConnection`, `BlockRangeInfo`
