# pipeline

## Purpose
Staged-sync pipeline orchestration: stage execution loop, control flow, events, and stage set builders.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Pipeline core implementation and run loop.
- **Key items**: `Pipeline`, `PipelineFut`, `PipelineWithResult`, `PipelineEvent`
- **Interactions**: Drives `Stage` execution and emits pipeline events/metrics.
- **Knobs / invariants**: `fail_on_unwind` stops on unexpected unwind; honors `max_block`.

### `builder.rs`
- **Role**: Builder for assembling pipeline stages and settings.
- **Key items**: `PipelineBuilder`, `add_stage()`, `add_stages()`, `with_max_block()`

### `ctrl.rs`
- **Role**: Control flow enum for pipeline outcomes.
- **Key items**: `ControlFlow`

### `event.rs`
- **Role**: Event types emitted during pipeline execution.
- **Key items**: `PipelineEvent`, `PipelineStagesProgress`

### `progress.rs`
- **Role**: Tracks minimum/maximum block progress across stages.
- **Key items**: `PipelineProgress`, `next_ctrl()`

### `set.rs`
- **Role**: Stage set abstraction and ordering helpers.
- **Key items**: `StageSet`, `StageSetBuilder`
- **Knobs / invariants**: Maintains stage ordering and enable/disable flags.

## End-to-end flow (high level)
- Build a `Pipeline` with `PipelineBuilder` and `StageSet` helpers.
- Run `Pipeline::run`/`run_as_fut` to execute stages sequentially.
- Each stage emits `PipelineEvent` and updates `PipelineProgress`.
- Control flow (`Continue`/`NoProgress`/`Unwind`) guides the loop behavior.

## Key APIs (no snippets)
- `Pipeline`, `PipelineBuilder`, `StageSetBuilder`
- `PipelineEvent`, `ControlFlow`, `PipelineProgress`
