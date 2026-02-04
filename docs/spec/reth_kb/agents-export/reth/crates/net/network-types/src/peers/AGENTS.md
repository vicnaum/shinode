# peers

## Purpose
Peer-related shared types and configuration: describes peer identity/addressing, connection state, reputation scoring and outcomes, and configuration knobs for peering/backoff/ban lists.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Peer model root: defines `Peer` and re-exports peer config/state/reputation/kind/address types.
- **Key items**: `Peer`, `apply_reputation()`, `ReputationChangeOutcome`, `ConnectionsConfig`, `PeersConfig`

### `addr.rs`
- **Role**: Socket addressing for a peer (TCP + optional UDP) used by dialing and discovery integration.
- **Key items**: `PeerAddr`, `PeerAddr::tcp()`, `udp()`, `new_with_ports()`

### `kind.rs`
- **Role**: Classification of peers by trust/source (basic/static/trusted) with convenience predicates.
- **Key items**: `PeerKind`, `is_trusted()`, `is_static()`, `is_basic()`

### `state.rs`
- **Role**: Tracks current connection lifecycle state to a peer (idle/connected/pending/disconnecting).
- **Key items**: `PeerConnectionState`, `disconnect()`, `is_connected()`, `is_pending_out()`

### `reputation.rs`
- **Role**: Reputation scoring constants, change kinds/weights, and computed outcomes (ban/unban/disconnect) used to police peers and backoff decisions.
- **Key items**: `Reputation`, `ReputationChangeKind`, `ReputationChangeWeights`, `ReputationChangeOutcome`, `BANNED_REPUTATION`, `FAILED_TO_CONNECT_REPUTATION_CHANGE`, `MAX_TRUSTED_PEER_REPUTATION_CHANGE`
- **Knobs / invariants**: Banning threshold and per-kind weights are configurable; `BadProtocol` is maximal penalty.

### `config.rs`
- **Role**: Peering configuration for `PeersManager`: connection limits, backoff durations, ban durations/lists, trusted peers, and IP filtering.
- **Key items**: `PeersConfig`, `ConnectionsConfig`, `PeerBackoffDurations`, `DEFAULT_MAX_COUNT_PEERS_OUTBOUND`, `DEFAULT_MAX_COUNT_PEERS_INBOUND`, `DEFAULT_MAX_COUNT_CONCURRENT_OUTBOUND_DIALS`, `INBOUND_IP_THROTTLE_DURATION`
- **Knobs / invariants**: Backoff duration scales with counter and is capped by `max`; trusted peers are exempt from some drop/backoff behavior.

## Key APIs (no snippets)
- `Peer`, `PeerAddr`, `PeerKind`, `PeerConnectionState`
- `PeersConfig`, `ConnectionsConfig`, `PeerBackoffDurations`
- `ReputationChangeKind`, `ReputationChangeWeights`
