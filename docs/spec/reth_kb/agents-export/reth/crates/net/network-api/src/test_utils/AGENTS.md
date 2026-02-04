# test_utils

## Purpose
Test/integration helpers for `reth-network-api`: provides a lightweight handle interface for interacting with a peer manager in tests (add/remove peers, reputation changes, and peer queries).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module glue that exposes the peers-manager testing API types.
- **Key items**: `PeersHandle`, `PeersHandleProvider`, `PeerCommand`

### `peers_manager.rs`
- **Role**: Defines `PeersHandle` and command types used to drive `PeersManager` behavior from tests (manual peer set edits and lookups).
- **Key items**: `PeersHandleProvider`, `PeersHandle`, `PeerCommand`, `PeersHandle::add_peer()`, `remove_peer()`, `reputation_change()`, `peer_by_id()`, `all_peers()`
