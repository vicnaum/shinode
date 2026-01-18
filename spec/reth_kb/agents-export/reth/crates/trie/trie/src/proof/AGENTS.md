# proof

## Purpose
Merkle proof generation for account and storage tries using trie walkers, hashed cursors, and
hash builder retention.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Proof generators for accounts and storage, producing multiproofs and account proofs.
- **Key items**: `Proof`, `StorageProof`, `account_proof()`, `multiproof()`, `storage_proof()`,
  `storage_multiproof()`, `with_branch_node_masks()`
- **Interactions**: Uses `TrieWalker`, `TrieNodeIter`, and `HashBuilder` to rebuild roots and
  retain proof nodes.
- **Knobs / invariants**: `collect_branch_node_masks` toggles branch mask capture; prefix sets are
  extended with proof targets.

### `trie_node.rs`
- **Role**: Blinded node providers that materialize missing trie nodes by issuing proofs.
- **Key items**: `ProofTrieNodeProviderFactory`, `ProofBlindedAccountProvider`,
  `ProofBlindedStorageProvider`
- **Interactions**: Uses `Proof`/`StorageProof` to fetch nodes and emits `RevealedNode` with masks.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Proof`, `StorageProof`, `ProofTrieNodeProviderFactory`
- **Modules / Packages**: `trie_node`
- **Functions**: `account_proof()`, `multiproof()`, `storage_proof()`, `storage_multiproof()`

## Relationships
- **Depends on**: `TrieCursorFactory` and `HashedCursorFactory` for trie traversal and leaf access.
- **Depends on**: `reth-trie-common` proof types (`MultiProof`, `StorageMultiProof`).
- **Data/control flow**: proof targets expand prefix sets, walker iterates trie nodes, hash builder
  retains proof nodes, and storage proofs are embedded into account proofs.

## End-to-end flow (high level)
- Build a `Proof` with trie and hashed cursor factories.
- Extend prefix sets with targets and walk the trie.
- Use `HashBuilder` retainer to capture proof nodes while recomputing roots.
- Emit account/storage proofs; optionally fetch blinded nodes via proof-based providers.
