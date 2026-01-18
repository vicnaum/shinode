# sparse

## Purpose
`reth-trie-sparse` crate: sparse MPT implementation with lazy node reveal, account/storage state
trie wrapper, update tracking, and optional metrics.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for RLP node updates and root calculation.
- [x] `src/` - Core sparse trie types, state trie wrapper, providers, and metrics.

### Files
- `Cargo.toml` - Crate manifest for sparse trie implementation and feature flags.
  - **Key items**: features `std`, `metrics`, `test-utils`, `arbitrary`; benches `root`, `rlp_node`;
    deps `reth-trie-common`, `alloy-trie`, `alloy-rlp`, `smallvec`, optional `rayon`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `SparseTrie`, `SerialSparseTrie`, `SparseStateTrie`,
  `SparseTrieInterface`, `TrieNodeProvider`, `TrieNodeProviderFactory`, `SparseTrieUpdates`
- **Modules / Packages**: `state`, `trie`, `traits`, `provider`
- **Functions**: `reveal_multiproof()`, `reveal_witness()`, `root_with_updates()`,
  `update_leaf()`, `remove_leaf()`

## Relationships
- **Depends on**: `reth-trie-common`, `alloy-trie`, `alloy-rlp` for trie node encoding/decoding.
- **Depends on**: `reth-execution-errors` for sparse trie error types.
- **Data/control flow**: multiproof and witness data is decoded, revealed into sparse tries, and
  updated leaves yield new roots and update sets for persistence.
