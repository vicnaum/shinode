# src

## Purpose
Stage implementations and built-in stage sets used by the sync pipeline.

## Contents (one hop)
### Subdirectories
- [x] `stages/` - Concrete stage implementations (download, execute, hash, prune, finish).
- [x] `test_utils/` - Stage test harness, runners, and macros.

### Files
- `lib.rs` - Crate entrypoint and re-exports for pipeline and stages.
  - **Key items**: `Pipeline`, `Stage`, `StageSet`, `stages`, `sets`
- `prelude.rs` - Re-exports of common stage set types.
  - **Key items**: `DefaultStages`, `OnlineStages`, `OfflineStages`
- `sets.rs` - Built-in stage sets and wiring for default sync order.
  - **Key items**: `DefaultStages`, `OnlineStages`, `OfflineStages`, `HashingStages`
  - **Interactions**: Constructs `StageSetBuilder` with downloader, consensus, and config inputs.

## Key APIs (no snippets)
- `DefaultStages`, `OnlineStages`, `OfflineStages`
- `Pipeline`, `StageSet`
