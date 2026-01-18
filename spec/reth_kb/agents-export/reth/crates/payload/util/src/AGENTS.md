# src

## Purpose
Utilities for selecting and composing transaction iterators during payload building.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and re-exports for payload transaction iterators.
- **Key items**: `PayloadTransactions`, `BestPayloadTransactions`, `PayloadTransactionsChain`

### `traits.rs`
- **Role**: Core iterator trait and adapters for txpool best transactions.
- **Key items**: `PayloadTransactions`, `NoopPayloadTransactions`, `BestPayloadTransactions`
- **Interactions**: Adapts `ValidPoolTransaction` iterators from the txpool.
- **Knobs / invariants**: `mark_invalid()` must prevent dependent txs from reappearing.

### `transaction.rs`
- **Role**: Fixed and chained transaction iterators with gas budgeting.
- **Key items**: `PayloadTransactionsFixed`, `PayloadTransactionsChain`
- **Knobs / invariants**: `PayloadTransactionsChain` enforces per-segment gas limits and propagates invalidation.

## End-to-end flow (high level)
- Build a `PayloadTransactions` iterator from txpool best transactions.
- Optionally prepend fixed transactions (e.g., sequencer-specified).
- Chain iterators with gas budgets to control block composition order.

## Key APIs (no snippets)
- `PayloadTransactions`, `BestPayloadTransactions`
- `PayloadTransactionsFixed`, `PayloadTransactionsChain`
