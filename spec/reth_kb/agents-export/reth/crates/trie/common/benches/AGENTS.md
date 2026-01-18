# benches

## Purpose
Criterion benchmarks for trie common components (hashed state and prefix set performance).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `hashed_state.rs`
- **Role**: Benchmarks parallel construction of `HashedPostState` with alternative iterator strategies.
- **Key items**: `generate_test_data()`, `keccak256_u64()`, `from_par_iter_fold_reduce()`, `from_par_iter_collect_twice()`, `bench_from_parallel_iterator()`
- **Interactions**: Exercises `HashedPostState::from_par_iter` under the `rayon` feature.

### `prefix_set.rs`
- **Role**: Benchmarks prefix-set lookup implementations across varying set sizes.
- **Key items**: `PrefixSetMutAbstraction`, `PrefixSetAbstraction`, `prefix_set_lookups()`, `generate_test_data()`, `prefix_set_bench()`
- **Interactions**: Compares BTreeSet-backed and Vec/cursor-backed prefix set variants.

## End-to-end flow (high level)
- Generate randomized test data at multiple sizes.
- Run Criterion benchmarks for each implementation.
- Compare throughput across strategies for hashed state building and prefix lookups.
