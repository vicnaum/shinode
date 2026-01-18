# setup

## Purpose
Benchmark setup utilities for stages: test data generation and stage range helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Benchmark setup helpers for stage execution and unwind flows.
- **Key items**: `StageRange`, `stage_unwind()`, `unwind_hashes()`, `txs_testdata()`
- **Interactions**: Uses `AccountHashingStage`/`StorageHashingStage` for setup.

### `account_hashing.rs`
- **Role**: Prepares account-hashing benchmark data and stage ranges.
- **Key items**: `prepare_account_hashing()`, `generate_testdata_db()`
- **Knobs / invariants**: Optional external DB via `ACCOUNT_HASHING_DB` env var.

### `constants.rs`
- **Role**: Benchmark environment constants.
- **Key items**: `ACCOUNT_HASHING_DB`

## End-to-end flow (high level)
- Detect whether an external DB is provided via `ACCOUNT_HASHING_DB`.
- If missing, generate local test data and stage range.
- Provide helpers to reset/unwind stage state before benchmarking.

## Key APIs (no snippets)
- `prepare_account_hashing()`, `txs_testdata()`, `stage_unwind()`
