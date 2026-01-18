# benches

## Purpose
Criterion benchmarks for core sync stages and their performance profiles.

## Contents (one hop)
### Subdirectories
- [x] `setup/` - Benchmark data generation and stage range helpers.

### Files
- `README.md` - Benchmark usage and external DB instructions.
  - **Key items**: `ACCOUNT_HASHING_DB`, `cargo bench --package reth-stages --bench criterion`
- `criterion.rs` - Criterion benchmark runner for multiple stages.
  - **Key items**: `transaction_lookup()`, `account_hashing()`, `senders()`, `merkle()`
  - **Interactions**: Uses `setup` helpers to build test DBs and unwind stages.

## Key APIs (no snippets)
- `run_benches()`, `measure_stage()`
