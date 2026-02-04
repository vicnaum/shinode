# src

## Purpose
Stage API definitions: stage trait and inputs/outputs, pipeline orchestration, metrics, and errors.

## Contents (one hop)
### Subdirectories
- [x] `metrics/` - Metric events and listener for stage progress gauges.
- [x] `pipeline/` - Pipeline orchestration, control flow, and stage set builders.

### Files
- `lib.rs` - Module wiring and public re-exports for stages and pipeline.
  - **Key items**: `Stage`, `Pipeline`, `StageId`, `StageCheckpoint`
- `error.rs` - Stage/pipeline error types and helpers.
  - **Key items**: `StageError`, `PipelineError`, `BlockErrorKind`
- `stage.rs` - Stage trait definitions and execution/unwind inputs.
  - **Key items**: `ExecInput`, `ExecOutput`, `UnwindInput`, `UnwindOutput`, `Stage`, `StageExt`
  - **Knobs / invariants**: Range helpers enforce transaction/blocks thresholds.
- `util.rs` - Small option helpers for min/max aggregation.
  - **Key items**: `opt::max()`, `opt::min()`
- `test_utils.rs` - Test stage implementation used by stage tests.
  - **Key items**: `TestStage`

## Key APIs (no snippets)
- `Stage`, `ExecInput`, `ExecOutput`, `UnwindInput`, `UnwindOutput`
- `StageError`, `PipelineError`, `BlockErrorKind`
