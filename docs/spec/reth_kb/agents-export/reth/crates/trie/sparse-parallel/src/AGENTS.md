# src

## Purpose
Parallel sparse trie implementation that splits the trie into upper and lower subtries to
parallelize reveals and hash updates, with update tracking and optional metrics.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint that re-exports the parallel sparse trie and wires internal modules.
- **Key items**: `trie`, `lower`, `metrics` (feature gated)
- **Interactions**: Re-exports `ParallelSparseTrie` and related types from `trie.rs`.

### `lower.rs`
- **Role**: Manages the lower subtrie lifecycle (blind vs revealed) and allocation reuse.
- **Key items**: `LowerSparseSubtrie`, `reveal()`, `clear()`, `take_revealed()`,
  `take_revealed_if()`, `shrink_nodes_to()`, `shrink_values_to()`
- **Interactions**: Used by `ParallelSparseTrie` to store and recycle lower subtries.
- **Knobs / invariants**: `reveal()` may shorten the stored root path; `clear()` preserves a
  cleared subtrie for reuse.

### `metrics.rs`
- **Role**: Metrics collection for parallel sparse trie hash updates.
- **Key items**: `ParallelSparseTrieMetrics`, `subtries_updated`, `subtrie_hash_update_latency`,
  `subtrie_upper_hash_latency`
- **Interactions**: Used by `ParallelSparseTrie` when the `metrics` feature is enabled.

### `trie.rs`
- **Role**: Core parallel sparse trie implementation, including subtrie partitioning, leaf
  updates, parallel reveal and hashing, and update tracking.
- **Key items**: `ParallelSparseTrie`, `ParallelismThresholds`, `UPPER_TRIE_MAX_DEPTH`,
  `NUM_LOWER_SUBTRIES`, `SparseSubtrie`, `SparseSubtrieType`, `LeafUpdateStep`,
  `SparseSubtrieBuffers`, `SparseTrieUpdatesAction`, `RlpNodePathStackItem`
- **Interactions**: Uses `SparseNode`/`SparseTrieInterface` from `reth-trie-sparse`,
  `PrefixSet` and `BranchNodeMasks` from `reth-trie-common`, and `TrieNodeProvider`
  for revealing blinded nodes.
- **Knobs / invariants**: `ParallelismThresholds` gates parallel execution; paths shorter than
  `UPPER_TRIE_MAX_DEPTH` live in the upper subtrie; prefix sets drive incremental hashing.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ParallelSparseTrie`, `ParallelismThresholds`,
  `SparseSubtrie`, `SparseSubtrieType`, `LowerSparseSubtrie`
- **Modules / Packages**: `trie`, `lower`, `metrics`
- **Functions**: `with_parallelism_thresholds()`, `reveal_nodes()`, `update_leaf()`,
  `remove_leaf()`, `root()`

## Relationships
- **Depends on**: `reth-trie-sparse` for sparse node types and shared interfaces.
- **Depends on**: `reth-trie-common` for masks, RLP nodes, and prefix sets.
- **Depends on**: `rayon` (optional) to parallelize reveal and hash updates under `std`.
- **Data/control flow**: proof nodes are partitioned into upper/lower subtries and revealed,
  then updates are applied to subtrie maps and prefix sets.
- **Data/control flow**: hash updates run bottom-up on lower subtries (optionally in parallel),
  then the upper subtrie hash is updated to produce the final root.
- **Data/control flow**: update actions emitted by subtries are folded into `SparseTrieUpdates`.

## End-to-end flow (high level)
- Initialize `ParallelSparseTrie` (empty or from a root node) and configure thresholds.
- Reveal proof nodes, updating branch masks and routing nodes to upper/lower subtries.
- Apply leaf updates or removals, revealing blinded nodes via providers as needed.
- Recompute lower subtrie hashes (parallel when thresholds are met), then update upper hashes.
- Return the root hash and optional update sets for persistence.
