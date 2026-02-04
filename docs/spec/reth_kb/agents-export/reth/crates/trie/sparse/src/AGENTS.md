# src

## Purpose
Core sparse trie implementation and state-trie wrapper used for lazy node reveal, update tracking,
and root computation. Defines provider traits, sparse trie data structures, and optional metrics.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint that wires modules, re-exports main types, and exposes sparse trie
  error types.
- **Key items**: `state`, `trie`, `traits`, `provider`, `errors`, `SparseTrieErrorKind`
- **Interactions**: Re-exports public APIs defined in `state.rs`, `trie.rs`, and `traits.rs`.

### `metrics.rs`
- **Role**: Metrics collection for sparse state trie multiproof reveals.
- **Key items**: `SparseStateTrieMetrics`, `SparseStateTrieInnerMetrics`, `record()`,
  `increment_*_nodes()`
- **Interactions**: Used by `SparseStateTrie` when the `metrics` feature is enabled.
- **Knobs / invariants**: Counters are accumulated and recorded into histograms via `record()`.

### `provider.rs`
- **Role**: Traits and default implementations for retrieving blinded trie nodes on demand.
- **Key items**: `TrieNodeProviderFactory`, `TrieNodeProvider`, `RevealedNode`,
  `DefaultTrieNodeProviderFactory`, `DefaultTrieNodeProvider`, `pad_path_to_key()`
- **Interactions**: Providers are used by `SparseTrieInterface` operations in `trie.rs` and
  reveal helpers in `state.rs`.

### `state.rs`
- **Role**: Sparse state trie that combines account and storage sparse tries with multiproof and
  witness reveal helpers.
- **Key items**: `SparseStateTrie`, `ClearedSparseStateTrie`, `reveal_multiproof()`,
  `reveal_decoded_multiproof()`, `reveal_witness()`, `root()`, `root_with_updates()`,
  `update_account_leaf()`, `update_storage_leaf()`, `update_account()`, `update_account_storage_root()`,
  `wipe_storage()`, `storage_root()`
- **Interactions**: Uses `SparseTrie` from `trie.rs`, `TrieNodeProviderFactory` from `provider.rs`,
  and `TrieAccount`/`MultiProof` types from `reth-trie-common`.
- **Knobs / invariants**: `retain_updates` controls update tracking; storage tries and revealed
  path sets are pooled for allocation reuse.

### `traits.rs`
- **Role**: Defines the sparse trie interface and shared update/result types.
- **Key items**: `SparseTrieInterface`, `SparseTrieUpdates`, `LeafLookup`, `LeafLookupError`
- **Interactions**: Implemented by `SerialSparseTrie` in `trie.rs` and used by `SparseTrie` wrapper.

### `trie.rs`
- **Role**: Core sparse trie implementation, node types, and RLP/hash update logic.
- **Key items**: `SparseTrie`, `SerialSparseTrie`, `SparseNode`, `SparseNodeType`,
  `RlpNodeBuffers`, `root_with_updates()`, `update_leaf()`, `remove_leaf()`, `find_leaf()`,
  `update_rlp_node_level()`, `rlp_node()`
- **Interactions**: Pulls blinded nodes via `TrieNodeProvider`, uses `PrefixSet` and
  `BranchNodeMasks` from `reth-trie-common`, emits `SparseTrieUpdates`.
- **Knobs / invariants**: Root node is always present; leaf values are stored in a separate map;
  `SPARSE_TRIE_SUBTRIE_HASHES_LEVEL` controls subtrie hash refresh depth.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `SparseTrie`, `SerialSparseTrie`, `SparseStateTrie`,
  `SparseTrieInterface`, `SparseTrieUpdates`, `TrieNodeProvider`, `TrieNodeProviderFactory`,
  `RevealedNode`
- **Modules / Packages**: `state`, `trie`, `traits`, `provider`, `metrics`
- **Functions**: `reveal_multiproof()`, `reveal_witness()`, `root_with_updates()`,
  `update_leaf()`, `remove_leaf()`, `update_rlp_node_level()`, `pad_path_to_key()`

## Relationships
- **Depends on**: `reth-trie-common` for node types, masks, and proof nodes.
- **Depends on**: `alloy-trie` and `alloy-rlp` for decoding and RLP encoding.
- **Depends on**: `reth-execution-errors` for sparse trie error types.
- **Data/control flow**: `MultiProof`/witness data is decoded into `ProofTrieNode`s and revealed
  into `SparseTrie` instances.
- **Data/control flow**: Leaf updates/removals use `TrieNodeProvider` to fetch blinded nodes
  and update `PrefixSet` for dirty hashing.
- **Data/control flow**: `SerialSparseTrie` recomputes RLP nodes and hashes, optionally capturing
  `SparseTrieUpdates` for DB persistence.
- **Data/control flow**: `SparseStateTrie` merges account and storage trie roots into account RLP,
  updates account leaves, and returns root and updates.

## End-to-end flow (high level)
- Initialize `SparseStateTrie` or `SparseTrie` in blind or revealed mode.
- Reveal account/storage paths using multiproofs or witness data.
- Apply account and storage leaf updates or removals, resolving blinded nodes via providers.
- Recompute subtrie hashes and root RLP nodes, optionally collecting update sets.
- Return root hashes (and updates) for persistence; clear or reuse tries for the next run.
