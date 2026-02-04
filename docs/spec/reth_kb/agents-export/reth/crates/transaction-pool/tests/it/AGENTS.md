# it

## Purpose
Integration tests for transaction pool behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring and feature gating.
- **Key items**: `blobs`, `evict`, `listeners`, `pending`, `best`

### `best.rs`
- **Role**: Sanity checks for best-transaction iteration.
- **Key items**: `test_best_transactions()`

### `blobs.rs`
- **Role**: Blob transaction exclusivity tests.
- **Key items**: `blobs_exclusive()`

### `evict.rs`
- **Role**: Pool eviction behavior under size limits.
- **Key items**: `only_blobs_eviction()`, `mixed_eviction()`

### `listeners.rs`
- **Role**: Listener and event stream behavior tests.
- **Key items**: `txpool_listener_by_hash()`, `txpool_listener_replace_event()`

### `pending.rs`
- **Role**: Pending transaction stream tests.
- **Key items**: `txpool_new_pending_txs()`

## End-to-end flow (high level)
- Insert mock transactions and assert subpool transitions.
- Verify listener event streams for pending/queued/replaced/discarded events.
- Validate eviction behavior with varied pool limits.

## Key APIs (no snippets)
- `TransactionPool`, `pending_transactions_listener()`
