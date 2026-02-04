# benches

## Purpose
Criterion benchmarks for trie hashing, proof generation, and root calculation performance.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `hash_post_state.rs`
- **Role**: Benchmarks sequential versus parallel hashing of `HashedPostState` from bundle state.
- **Key items**: `hash_post_state()`, `from_bundle_state_seq()`, `generate_test_data()`,
  `HashedPostState::from_bundle_state()`
- **Interactions**: Uses `revm_database::BundleBuilder` to generate bundle state inputs.
- **Knobs / invariants**: Dataset sizes scale from 100 to 10_000; codspeed skips larger sizes.

### `proof_v2.rs`
- **Role**: Benchmarks legacy storage multiproof generation versus proof_v2 implementation.
- **Key items**: `bench_proof_algos()`, `generate_test_data()`, `StorageProof`,
  `StorageProofCalculator`
- **Interactions**: Uses mock cursor factories built from `HashedPostState`.
- **Knobs / invariants**: Dataset size and number of targets vary per benchmark case.

### `trie_root.rs`
- **Role**: Benchmarks receipt root calculation using `triehash::ordered_trie_root` vs
  `HashBuilder`.
- **Key items**: `trie_root_benchmark()`, `trie_hash_ordered_trie_root()`, `hash_builder_root()`
- **Interactions**: Encodes receipts via `Encodable2718` before hashing.

## End-to-end flow (high level)
- Generate randomized dataset inputs.
- Run alternative implementations for each operation (hashing, proof, root).
- Record timing across dataset sizes and target counts.
