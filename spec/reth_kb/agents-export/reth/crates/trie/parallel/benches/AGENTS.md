# benches

## Purpose
Criterion benchmarks comparing single-threaded and parallel state root computation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `root.rs`
- **Role**: Benchmarks state root calculation on synthetic data sets with varying sizes.
- **Key items**: `calculate_state_root()`, `generate_test_data()`, `ParallelStateRoot`, `StateRoot::root()`, `TrieInput::from_state()`
- **Interactions**: Compares sequential `StateRoot` vs `ParallelStateRoot` over overlay providers.

## End-to-end flow (high level)
- Generate randomized hashed account/storage state for a target size.
- Seed DB state and write trie updates.
- Benchmark sequential state root calculation.
- Benchmark parallel state root calculation using `ParallelStateRoot`.
