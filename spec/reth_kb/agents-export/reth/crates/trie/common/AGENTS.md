# common

## Purpose
`reth-trie-common` crate with shared trie types, hashing utilities, proofs, and update buffers used across trie implementations.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Criterion benchmarks for hashed state construction and prefix set lookups.
- [x] `src/` - Core trie common types: hashed state overlays, prefix sets, proofs, updates, and root helpers.

### Files
- `Cargo.toml` - Crate manifest defining features, dependencies, and bench setup.
  - **Key items**: features `serde`, `serde-bincode-compat`, `eip1186`, `reth-codec`, `rayon`, `test-utils`; benches `prefix_set`, `hashed_state`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `TrieInput`, `HashedPostState`, `TrieUpdates`, `MultiProof`, `PrefixSetMut` - shared primitives for trie computation and proof handling.
- **Modules / Packages**: `hash_builder`, `updates`, `proofs` - state snapshots, update buffers, and proof helpers.
- **Functions**: `state_root()`, `storage_root()` - root helpers re-exported from `alloy_trie`.

## Relationships
- **Depends on**: `alloy-trie`, `alloy-primitives`, `alloy-rlp` - trie node/proof encoding and hash types.
- **Depends on**: `reth-primitives-traits`, `revm-database` - account types and bundle state hashing inputs.
- **Data/control flow**: `TrieInput` aggregates `HashedPostState`, `TrieUpdates`, and prefix sets for higher-level trie computations.
