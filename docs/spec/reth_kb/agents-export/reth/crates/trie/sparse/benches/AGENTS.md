# benches

## Purpose
Criterion benchmarks for sparse trie operations and root computation performance.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `rlp_node.rs`
- **Role**: Benchmarks `SerialSparseTrie::update_rlp_node_level` across trie depths and update
  percentages after initial population.
- **Key items**: `update_rlp_node_level()`, `Criterion`, `TestRunner`, `DefaultTrieNodeProvider`,
  `SerialSparseTrie`
- **Interactions**: Uses `SerialSparseTrie` update and root APIs from the sparse trie crate.
- **Knobs / invariants**: Fixed dataset size (100_000), sampled update percentages, per-depth
  iteration.

### `root.rs`
- **Role**: Compares root computation using `HashBuilder` versus `SparseTrie`, and measures
  repeated update scenarios.
- **Key items**: `calculate_root_from_leaves()`, `calculate_root_from_leaves_repeated()`,
  `generate_test_data()`, `SparseTrie::<SerialSparseTrie>`, `HashBuilder`, `TrieNodeIter`
- **Interactions**: Uses `reth_trie` walkers and cursors for the hash-builder baseline.
- **Knobs / invariants**: Dataset sizes and update sizes vary; some runs are skipped under
  `codspeed` to avoid long benchmarks.

## End-to-end flow (high level)
- Generate randomized account/storage inputs.
- Populate sparse trie or hash builder and compute initial roots.
- Apply batches of updates and recompute roots.
- Record timings for each configuration.
