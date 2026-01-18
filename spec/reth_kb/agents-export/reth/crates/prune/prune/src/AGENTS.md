# src

## Purpose
Core prune engine: pruner runtime, segment wiring, and DB pruning helpers.

## Contents (one hop)
### Subdirectories
- [x] `segments/` - Segment trait definitions and user/static prune segments.

### Files
- `lib.rs` - Crate wiring and re-exports for the prune engine.
  - **Key items**: `Pruner`, `PrunerBuilder`, `PruneLimiter`, `PrunerError`
- `builder.rs` - Builder for configuring and constructing a `Pruner`.
  - **Key items**: `PrunerBuilder`, `build()`, `build_with_provider_factory()`
  - **Knobs / invariants**: `block_interval`, `delete_limit`, `timeout`, `segments`
- `pruner.rs` - Main pruner runtime, run loop, and events.
  - **Key items**: `Pruner`, `run_with_provider()`, `PrunerResult`
  - **Interactions**: Emits `PrunerEvent`, uses `PruneLimiter` and per-segment pruning.
- `limiter.rs` - Per-run limits for pruning by time or deleted entries.
  - **Key items**: `PruneLimiter`, `progress()`, `interrupt_reason()`
- `error.rs` - Error types for prune execution.
  - **Key items**: `PrunerError`
- `db_ext.rs` - DB transaction pruning extensions and helpers.
  - **Key items**: `DbTxPruneExt`, `prune_table_with_range()`, `prune_dupsort_table_with_range()`
- `metrics.rs` - Metrics for prune duration and per-segment progress.
  - **Key items**: `Metrics`, `PrunerSegmentMetrics`

## Key APIs (no snippets)
- `Pruner`, `PrunerBuilder`, `PruneLimiter`
- `Segment`, `PrunerEvent`
