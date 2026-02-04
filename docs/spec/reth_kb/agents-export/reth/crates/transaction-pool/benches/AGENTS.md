# benches

## Purpose
Criterion benchmarks for transaction pool insertion, ordering, eviction, and updates.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `canonical_state_change.rs`
- **Role**: Benchmarks pool updates on canonical state changes.
- **Key items**: `canonical_state_change_bench()`, `fill_pool()`

### `insertion.rs`
- **Role**: Benchmarks individual and batch transaction insertion.
- **Key items**: `txpool_insertion()`, `txpool_batch_insertion()`

### `priority.rs`
- **Role**: Benchmarks blob priority and fee delta calculations.
- **Key items**: `blob_tx_priority()`, `fee_delta()`

### `reorder.rs`
- **Role**: Benchmarks reordering strategies for base-fee changes.
- **Key items**: `BenchTxPool`, `txpool_reordering_bench()`

### `truncate.rs`
- **Role**: Benchmarks subpool truncation behavior.
- **Key items**: `truncate_pending()`, `truncate_blob()`, `benchmark_pools()`

## End-to-end flow (high level)
- Generate mock transaction sets and configure pool limits.
- Run Criterion benchmarks for insertion, ordering, and eviction paths.

## Key APIs (no snippets)
- `canonical_state_change_bench()`, `txpool_insertion()`, `txpool_reordering()`
