# src

## Purpose
Merkle Patricia Trie core: root calculation, cursors and walkers, proof/witness generation, and
verification utilities.

## Contents (one hop)
### Subdirectories
- [x] `hashed_cursor/` - Hashed state cursor traits, overlays, metrics, and mocks.
- [x] `proof/` - Proof generators and proof-based node providers.
- [x] `proof_v2/` - Leaf-only proof calculator with deferred value encoding.
- [x] `trie_cursor/` - Trie cursor traits and implementations (overlay, depth-first, metrics).

### Files
- `changesets.rs` - Computes trie changesets (old node values) from trie updates.
  - **Key items**: `compute_trie_changesets()`, `compute_storage_changesets()`,
    `compute_wiped_storage_changesets()`, `storage_trie_wiped_changeset_iter()`
- `forward_cursor.rs` - Forward-only in-memory cursor over sorted entries.
  - **Key items**: `ForwardInMemoryCursor`, `seek()`, `first_after()`, `BINARY_SEARCH_THRESHOLD`
- `lib.rs` - Module wiring and re-exports for trie components and progress types.
  - **Key items**: `StateRoot`, `StorageRoot`, `TrieType`, `StateRootProgress`, `StorageRootProgress`
- `metrics.rs` - Metrics for root computation, walkers, and node iterators.
  - **Key items**: `StateRootMetrics`, `TrieRootMetrics`, `WalkerMetrics`, `TrieNodeIterMetrics`
- `mock.rs` - Shared mock helpers for cursor tests.
  - **Key items**: `KeyVisit`, `KeyVisitType`
- `node_iter.rs` - Iterator that merges trie walkers with hashed cursors for hash building.
  - **Key items**: `TrieNodeIter`, `TrieElement`, `TrieBranchNode`, `try_next()`
- `progress.rs` - Intermediate checkpoint types for resumable root computation.
  - **Key items**: `StateRootProgress`, `StorageRootProgress`, `IntermediateStateRootState`,
    `IntermediateRootState`
- `stats.rs` - Root calculation stats and tracker.
  - **Key items**: `TrieStats`, `TrieTracker`
- `test_utils.rs` - Test helpers for computing trie roots via `triehash`.
  - **Key items**: `state_root()`, `storage_root()`, `state_root_prehashed()`,
    `storage_root_prehashed()`
- `trie.rs` - State and storage root computation pipelines with thresholds and updates.
  - **Key items**: `StateRoot`, `StorageRoot`, `TrieType`, `root_with_updates()`,
    `root_with_progress()`, `with_threshold()`
- `verify.rs` - Verifies trie tables against hashed state tables, reporting inconsistencies.
  - **Key items**: `Verifier`, `Output`, `StateRootBranchNodesIter`
- `walker.rs` - Trie traversal with prefix-set skipping and removal tracking.
  - **Key items**: `TrieWalker`, `next_unprocessed_key()`, `with_deletions_retained()`
- `witness.rs` - Builds state transition witnesses using multiproofs and sparse trie updates.
  - **Key items**: `TrieWitness`, `WitnessTrieNodeProviderFactory`, `compute()`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StateRoot`, `StorageRoot`, `TrieWalker`, `TrieNodeIter`,
  `TrieCursorFactory`, `HashedCursorFactory`, `Verifier`, `TrieWitness`
- **Modules / Packages**: `changesets`, `proof`, `proof_v2`, `trie_cursor`, `hashed_cursor`
- **Functions**: `compute_trie_changesets()`, `root_with_updates()`, `multiproof()`,
  `storage_proof()`

## Relationships
- **Depends on**: `reth-trie-common` for hash builder, node types, and prefix sets.
- **Depends on**: `reth-trie-sparse` for sparse trie interfaces used in witness generation.
- **Data/control flow**: trie cursors and walkers feed `TrieNodeIter`, which drives `HashBuilder`
  for root computation and updates.
- **Data/control flow**: proofs and witnesses reuse cursor factories to fetch nodes and build
  proof node collections.
- **Data/control flow**: verification recomputes branch nodes from hashed state and compares
  against trie tables.
