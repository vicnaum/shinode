# segments

## Purpose
Prune segment trait definitions and shared helpers for segment execution.

## Contents (one hop)
### Subdirectories
- [x] `user/` - User-configured prune segments for history, receipts, and tx data.

## Files (detailed)

### `mod.rs`
- **Role**: Segment trait, shared prune helpers, and public re-exports.
- **Key items**: `Segment`, `PruneInput`, `prune_static_files()`
- **Interactions**: Calls `Segment::prune` and handles static file pruning for receipts/bodies.

### `receipts.rs`
- **Role**: Shared receipts pruning logic and checkpoint saving.
- **Key items**: `prune()`, `save_checkpoint()`
- **Interactions**: Handles DB vs static-file receipts destinations.

### `set.rs`
- **Role**: Segment collection builder based on prune modes.
- **Key items**: `SegmentSet`, `from_components()`
- **Interactions**: Builds segment list in pruning order from `PruneModes`.

## Key APIs (no snippets)
- `Segment`, `PruneInput`, `SegmentSet`
