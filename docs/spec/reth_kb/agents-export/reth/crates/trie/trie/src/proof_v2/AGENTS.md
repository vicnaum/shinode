# proof_v2

## Purpose
Leaf-only proof calculator that generates lexicographically ordered proof nodes with cursor
reuse and deferred value encoding.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core proof calculator that builds proofs from leaf data and cached branch nodes.
- **Key items**: `ProofCalculator`, `StorageProofCalculator`, `storage_proof()`,
  `TargetsCursor`, `PATH_ALL_ZEROS`
- **Interactions**: Uses `TrieCursor` for branch nodes and `HashedCursor` for leaf data, emitting
  `ProofTrieNode`s in depth-first order.
- **Knobs / invariants**: Targets must be sorted; calculator resets after each call and reuses
  internal buffers.

### `node.rs`
- **Role**: Internal branch/child node representations and RLP conversion helpers.
- **Key items**: `ProofTrieBranchChild`, `ProofTrieBranch`, `trim_nibbles_prefix()`
- **Interactions**: Converts deferred leaf encoders into RLP nodes and proof nodes.

### `target.rs`
- **Role**: Proof target representation and sub-trie grouping helpers.
- **Key items**: `Target`, `SubTrieTargets`, `iter_sub_trie_targets()`
- **Interactions**: Chunks targets by sub-trie prefix to drive proof calculation.

### `value.rs`
- **Role**: Deferred leaf value encoding for storage slots and accounts.
- **Key items**: `LeafValueEncoder`, `DeferredValueEncoder`, `StorageValueEncoder`,
  `SyncAccountValueEncoder`
- **Interactions**: Integrates with `ProofCalculator` to encode values late, allowing storage
  root computation at encode time.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProofCalculator`, `StorageProofCalculator`, `Target`,
  `LeafValueEncoder`, `DeferredValueEncoder`
- **Modules / Packages**: `node`, `target`, `value`
- **Functions**: `storage_proof()`, `iter_sub_trie_targets()`, `deferred_encoder()`

## Relationships
- **Depends on**: `TrieCursor` and `HashedCursor` for branch/leaf data.
- **Depends on**: `reth-trie-common` proof node types and masks.
- **Data/control flow**: targets are grouped by sub-trie, leaves are encoded lazily, branch nodes
  are assembled and retained into proof output.

## End-to-end flow (high level)
- Build a `ProofCalculator` with trie and hashed cursors.
- Sort and chunk targets by sub-trie prefix.
- Walk leaf data, building branch stacks and encoding deferred leaf values.
- Emit proof nodes in lexicographic order and reset internal buffers for reuse.
