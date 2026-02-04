# pool

## Purpose
Internal transaction pool implementation: subpools, ordering, event listeners, and update logic.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Pool internals entrypoint, `PoolInner` implementation, and re-exports.
- **Key items**: `PoolInner`, `PENDING_TX_LISTENER_BUFFER_SIZE`, `NEW_TX_LISTENER_BUFFER_SIZE`
- **Interactions**: Wraps `TxPool`, manages listeners and blob store access.

### `txpool.rs`
- **Role**: Core pool data structure with pending/queued/basefee/blob subpools.
- **Key items**: `TxPool`, `AllTransactions`, `PendingFees`
- **Knobs / invariants**: Enforces subpool limits and ordering; uses base fee and blob fee.

### `state.rs`
- **Role**: Transaction state flags and subpool classification.
- **Key items**: `TxState`, `SubPool`, `determine_queued_reason()`
- **Knobs / invariants**: Bitflag thresholds map to subpool selection.

### `pending.rs`
- **Role**: Pending subpool storing gapless, executable transactions.
- **Key items**: `PendingPool`, `PendingTransaction`
- **Interactions**: Feeds `BestTransactions` iterator for block production.

### `parked.rs`
- **Role**: Parked subpools for queued/basefee transactions.
- **Key items**: `ParkedPool`, `ParkedOrd`, `BasefeeOrd`, `QueuedOrd`
- **Knobs / invariants**: Enforces sender slot limits and truncation rules.

### `best.rs`
- **Role**: Iterators for best-transaction selection with fee constraints.
- **Key items**: `BestTransactions`, `BestTransactionsWithFees`

### `blob.rs`
- **Role**: Blob transaction subpool and prioritization helpers.
- **Key items**: `BlobTransactions`, `BlobOrd`, `blob_tx_priority()`, `fee_delta()`
- **Knobs / invariants**: Blob transactions must be gapless.

### `events.rs`
- **Role**: Pool event types for transaction lifecycle changes.
- **Key items**: `FullTransactionEvent`, `TransactionEvent`, `NewTransactionEvent`

### `listener.rs`
- **Role**: Event broadcast streams for per-tx and all-tx listeners.
- **Key items**: `TransactionEvents`, `AllTransactionsEvents`, `PoolEventBroadcast`
- **Knobs / invariants**: Event channel buffer sizes and final event termination.

### `size.rs`
- **Role**: Size accounting utility for pool memory tracking.
- **Key items**: `SizeTracker`

### `update.rs`
- **Role**: Update bookkeeping for pool promotions/demotions.
- **Key items**: `PoolUpdate`, `Destination`, `UpdateOutcome`

## End-to-end flow (high level)
- `PoolInner` validates and inserts transactions into `TxPool`.
- `TxPool` classifies transactions into subpools using `TxState` and fees.
- `PendingPool` yields best transactions via `BestTransactions`.
- Updates move transactions across subpools and emit events to listeners.

## Key APIs (no snippets)
- `PoolInner`, `TxPool`, `PendingPool`, `ParkedPool`
- `BestTransactions`, `BlobTransactions`
- `TransactionEvents`, `FullTransactionEvent`
