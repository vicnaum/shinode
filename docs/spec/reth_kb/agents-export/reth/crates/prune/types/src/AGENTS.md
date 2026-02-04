# src

## Purpose
Common prune configuration and runtime types: segments, modes, checkpoints, and events.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Crate entrypoint and re-exports for prune types and helpers.
  - **Key items**: `PruneModes`, `PruneMode`, `PruneSegment`, `PrunerOutput`, `ReceiptsLogPruneConfig`
- `segment.rs` - Prune segment identifiers and purpose enums.
  - **Key items**: `PruneSegment`, `PrunePurpose`, `PruneSegmentError`
  - **Knobs / invariants**: Segment enum order is stable for DB encoding.
- `mode.rs` - Prune mode semantics and target block computation.
  - **Key items**: `PruneMode`, `prune_target_block()`, `should_prune()`
- `target.rs` - Top-level prune configuration and minimum distance rules.
  - **Key items**: `PruneModes`, `MINIMUM_PRUNING_DISTANCE`, `UnwindTargetPrunedError`
- `pruner.rs` - Runtime output and progress types for pruner runs.
  - **Key items**: `PrunerOutput`, `SegmentOutput`, `PruneProgress`, `PruneInterruptReason`
- `checkpoint.rs` - Persisted pruning checkpoint structure.
  - **Key items**: `PruneCheckpoint`
- `event.rs` - Pruner lifecycle events.
  - **Key items**: `PrunerEvent`

## Key APIs (no snippets)
- `PruneModes`, `PruneMode`, `PruneSegment`
- `PruneCheckpoint`, `PrunerOutput`, `PrunerEvent`
