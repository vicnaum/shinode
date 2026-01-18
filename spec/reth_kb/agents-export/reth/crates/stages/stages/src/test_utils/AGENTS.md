# test_utils

## Purpose
Test helpers for stage implementations: test DB harness, stage runners, and macros.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and exports for stage test helpers.
- **Key items**: `TestStageDB`, `TestStages`, `StageTestRunner`, `TEST_STAGE_ID`

### `macros.rs`
- **Role**: Shared test suite macros for stage execute/unwind flows.
- **Key items**: `stage_test_suite!`, `stage_test_suite_ext!`

### `runner.rs`
- **Role**: Generic test runner traits for stage execution and unwind checks.
- **Key items**: `StageTestRunner`, `ExecuteStageTestRunner`, `UnwindStageTestRunner`, `TestRunnerError`
- **Interactions**: Spawns async tasks that call `Stage::execute`/`Stage::unwind`.

### `set.rs`
- **Role**: StageSet implementation backed by `TestStage` outputs.
- **Key items**: `TestStages`

### `test_db.rs`
- **Role**: Test database harness with helpers for inserting headers/blocks and querying tables.
- **Key items**: `TestStageDB`, `StorageKind`, `insert_headers()`, `insert_blocks()`
- **Knobs / invariants**: Uses temp MDBX/static files and optional RocksDB for tests.

## End-to-end flow (high level)
- Build a `TestStageDB` with temp static files and DB env.
- Implement `StageTestRunner` to seed data and validate results.
- Use macros to exercise execute/unwind flows consistently across stages.

## Key APIs (no snippets)
- `TestStageDB`, `StageTestRunner`, `ExecuteStageTestRunner`
- `stage_test_suite!`, `TestStages`
