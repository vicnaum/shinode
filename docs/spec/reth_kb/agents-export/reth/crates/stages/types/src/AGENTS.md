# src

## Purpose
Common stage and pipeline types: stage identifiers, checkpoints, and execution thresholds.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports for stage IDs, checkpoints, and execution controls.
- **Key items**: `StageId`, `StageCheckpoint`, `ExecutionStageThresholds`, `PipelineTarget`
- **Interactions**: Re-exports checkpoint structs from `checkpoints.rs`.

### `id.rs`
- **Role**: Stage ID registry and helpers.
- **Key items**: `StageId`, `StageId::ALL`, `StageId::STATE_REQUIRED`, `get_pre_encoded()`
- **Knobs / invariants**: Stage IDs are stable and used as DB keys.

### `checkpoints.rs`
- **Role**: Checkpoint structures for stages and merkle progress.
- **Key items**: `StageCheckpoint`, `EntitiesCheckpoint`, `MerkleCheckpoint`, `StorageRootMerkleCheckpoint`
- **Knobs / invariants**: Compact encoding for checkpoints; optional inner progress for merkle.

### `execution.rs`
- **Role**: Execution batching thresholds for the execution stage.
- **Key items**: `ExecutionStageThresholds`, `is_end_of_batch()`
- **Knobs / invariants**: Defaults enforce max blocks, changes, gas, and duration.

## End-to-end flow (high level)
- Pipeline code uses `StageId` to identify stages and persist checkpoints.
- Stages write `StageCheckpoint` with optional entity progress.
- Merkle stages persist `MerkleCheckpoint`/`StorageRootMerkleCheckpoint` for resumable hashing.
- Execution uses `ExecutionStageThresholds` to decide when to commit.

## Key APIs (no snippets)
- `StageId`, `StageCheckpoint`, `EntitiesCheckpoint`
- `MerkleCheckpoint`, `StorageRootMerkleCheckpoint`
- `ExecutionStageThresholds`, `PipelineTarget`
