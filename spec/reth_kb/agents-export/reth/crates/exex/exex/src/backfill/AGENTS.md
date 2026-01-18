# backfill

## Purpose
Backfill utilities for ExEx notifications: execute historical block ranges (single-block or batched) using an `EvmConfig` + provider, producing `Chain`/execution outputs to "catch up" an ExEx to a target head.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - module wiring and public re-exports for backfill jobs and factories.
  - **Key items**: `BackfillJobFactory`, `BackfillJob`, `SingleBlockBackfillJob`, `StreamBackfillJob`
- `factory.rs` - backfill job factory that captures `evm_config`, `provider`, prune modes, thresholds, and stream parallelism.
  - **Key items**: `BackfillJobFactory::new()`, `backfill()`, `new_from_components()`
- `job.rs` - core backfill iterators: `BackfillJob` (batched execution yielding `Chain`) and `SingleBlockBackfillJob` (yields `(RecoveredBlock, BlockExecutionOutput)`), with threshold-driven batching.
  - **Key items**: `BackfillJob`, `BackfillJob::into_stream()`, `SingleBlockBackfillJob`, `execute_block()`
- `stream.rs` - async stream adapter that executes backfill jobs in parallel using `spawn_blocking`, while yielding results in-order.
  - **Key items**: `StreamBackfillJob`, `with_parallelism()`, `with_batch_size()`
- `test_utils.rs` (tests) - helpers for constructing test chains/specs and executing blocks into a test provider factory.
