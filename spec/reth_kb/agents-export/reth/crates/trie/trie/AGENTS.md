# trie

## Purpose
`reth-trie` crate: Merkle Patricia Trie implementation with root computation, cursors, proofs,
witness generation, and verification tooling.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks for hashing, proof generation, and trie roots.
- [x] `src/` - Core trie logic: root calculation, cursors, proofs, witnesses, verification.
- [x] `testdata/` - (skip: static JSON fixtures for proof tests)

### Files
- `Cargo.toml` - Crate manifest for trie implementation and feature flags.
  - **Key items**: features `metrics`, `serde`, `test-utils`; benches `hash_post_state`,
    `trie_root`, `proof_v2`; deps `reth-trie-common`, `reth-trie-sparse`, `alloy-trie`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StateRoot`, `StorageRoot`, `TrieWalker`, `TrieNodeIter`,
  `TrieWitness`, `Proof`, `ProofCalculator`
- **Modules / Packages**: `trie_cursor`, `hashed_cursor`, `proof`, `proof_v2`, `witness`
- **Functions**: `root_with_updates()`, `multiproof()`, `storage_proof()`

## Relationships
- **Depends on**: `reth-trie-common` for hash builder, nodes, and prefix sets.
- **Depends on**: `reth-trie-sparse` for sparse trie integration in witness generation.
- **Depends on**: `alloy-trie`/`alloy-rlp` for trie node encoding and proofs.
- **Data/control flow**: cursors and walkers feed node iterators, which drive hash builders to
  compute roots and updates.
