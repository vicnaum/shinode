# src

## Purpose
Shared trie data structures and helpers: hashed state overlays, prefix sets, proofs, update buffers, and root computation utilities.

## Contents (one hop)
### Subdirectories
- [x] `hash_builder/` - Hash builder state snapshots and compact encoding helpers.

### Files
- `account.rs` - Re-exports the trie account type and validates conversions from genesis/account inputs.
  - **Key items**: `TrieAccount`, `GenesisAccount`, `Account`
  - **Interactions**: Tests call `root::storage_root_unhashed` for storage-root derivation.
- `added_removed_keys.rs` - Tracks added/removed keys across account and storage tries.
  - **Key items**: `MultiAddedRemovedKeys`, `update_with_state()`, `touch_accounts()`, `AddedRemovedKeys`
  - **Interactions**: Consumes `HashedPostState` updates to update removal sets.
- `constants.rs` - Defines trie size constants for RLP encoding.
  - **Key items**: `TRIE_ACCOUNT_RLP_MAX_SIZE`, `account_rlp_max_size()`
- `hashed_state.rs` - In-memory hashed state overlays, sorted forms, and chunking utilities.
  - **Key items**: `HashedPostState`, `HashedStorage`, `HashedPostStateSorted`, `HashedStorageSorted`, `ChunkedHashedPostState`
  - **Interactions**: Builds `TriePrefixSetsMut`, consumes `MultiProofTargets`, and uses `MultiAddedRemovedKeys`.
  - **Knobs / invariants**: `wiped` flag semantics; chunking preserves wipe -> storage updates -> account ordering.
- `input.rs` - Aggregates trie updates, hashed state, and prefix sets into computation inputs.
  - **Key items**: `TrieInput`, `TrieInputSorted`, `from_state()`, `from_blocks()`, `append_cached_ref()`
  - **Interactions**: Composes `TrieUpdates`, `HashedPostState`, and `TriePrefixSetsMut`.
- `key.rs` - Key hashing trait and Keccak implementation.
  - **Key items**: `KeyHasher`, `KeccakKeyHasher`, `hash_key()`
- `lib.rs` - Crate entrypoint with module wiring and re-exports.
  - **Key items**: `TrieInput`, `TrieAccount`, `KeccakKeyHasher`, `BranchNodeCompact`, `HashBuilder`, `TrieMask`
  - **Interactions**: Re-exports alloy-trie primitives for downstream crates.
- `nibbles.rs` - Stored nibble wrappers and compact encoding helpers.
  - **Key items**: `Nibbles`, `StoredNibbles`, `StoredNibblesSubKey`, `Compact` impls
- `prefix_set.rs` - Mutable and immutable prefix-set containers for trie invalidation.
  - **Key items**: `TriePrefixSetsMut`, `TriePrefixSets`, `PrefixSetMut`, `PrefixSet`, `freeze()`
  - **Knobs / invariants**: `PrefixSet::contains()` maintains a cursor for sequential lookups; `all` flag overrides keys.
- `proofs.rs` - Merkle proof and multiproof structures with verification helpers.
  - **Key items**: `MultiProofTargets`, `MultiProof`, `DecodedMultiProof`, `AccountProof`, `StorageProof`, `StorageMultiProof`
  - **Interactions**: Uses `BranchNodeMasksMap`, `TrieAccount`, and `verify_proof` from `alloy_trie`.
- `root.rs` - Re-exports root computation helpers from `alloy_trie`.
  - **Key items**: `state_root`, `state_root_unhashed`, `state_root_unsorted`, `storage_root`, `storage_root_unhashed`, `storage_root_unsorted`
- `storage.rs` - Storage trie table entries and changeset value wrappers.
  - **Key items**: `StorageTrieEntry`, `TrieChangeSetsEntry`, `StoredNibblesSubKey`, `BranchNodeCompact`
  - **Interactions**: Implements `ValueWithSubKey` for DB changeset usage.
- `subnode.rs` - Stored subnode representation for trie walker state.
  - **Key items**: `StoredSubNode`, `key`, `nibble`, `node`
  - **Interactions**: Uses `BranchNodeCompact` for persisted node snapshots.
- `trie.rs` - Sparse trie mask types and proof node descriptors.
  - **Key items**: `BranchNodeMasks`, `BranchNodeMasksMap`, `ProofTrieNode`
- `updates.rs` - Trie update buffers and sorted variants, including batch merge utilities.
  - **Key items**: `TrieUpdates`, `StorageTrieUpdates`, `TrieUpdatesSorted`, `StorageTrieUpdatesSorted`, `merge_batch()`, `finalize()`
  - **Interactions**: Consumes `HashBuilder` output and compacts changes for storage tries.
  - **Knobs / invariants**: `is_deleted` wipes prior storage nodes; empty-nibble paths are filtered.
- `utils.rs` - Sorted merge helpers used by hashed state and update buffers.
  - **Key items**: `kway_merge_sorted()`, `extend_sorted_vec()`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `HashedPostState`, `TrieUpdates`, `MultiProof`, `TrieInput`, `TriePrefixSetsMut` - core data carriers for trie computation and proofs.
- **Modules / Packages**: `hash_builder`, `prefix_set`, `updates` - state snapshots, invalidation sets, and update buffers.
- **Functions**: `state_root()`, `storage_root_unhashed()`, `kway_merge_sorted()`, `extend_sorted_vec()` - root helpers and sorted-merge utilities.

## Relationships
- **Depends on**: `alloy-trie`, `alloy-primitives`, `reth-primitives-traits` - trie node encoding, hash types, and account traits.
- **Data/control flow**: `HashedPostState` + `TrieUpdates` + `TriePrefixSetsMut` compose a `TrieInput`.
- **Data/control flow**: `TrieInput`/`TrieInputSorted` feed root/proof computation in higher-level trie crates.
- **Data/control flow**: `MultiProofTargets` -> `MultiProof`/`AccountProof` -> `verify()` for proof validation.
- **Data/control flow**: `HashBuilder`/`HashBuilderState` snapshots feed `TrieUpdates::finalize()`.
