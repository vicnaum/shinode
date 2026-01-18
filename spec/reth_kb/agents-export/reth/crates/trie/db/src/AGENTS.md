# src

## Purpose
Database-backed trie integrations: cursor factories, prefix set loading, root computation, proofs, witnesses, and changeset caching.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `changesets.rs`
- **Role**: Computes trie changesets/updates for blocks and caches them for reorg-safe lookups.
- **Key items**: `compute_block_trie_changesets()`, `compute_block_trie_updates()`, `ChangesetCache`, `get_or_compute_range()`, `evict()`
- **Interactions**: Uses `DatabaseTrieCursorFactory`, `InMemoryTrieCursorFactory`, and `StateRoot::overlay_root_from_nodes_with_updates`.
- **Knobs / invariants**: Cache eviction is explicit; range accumulation is newest-to-oldest so older values win.

### `hashed_cursor.rs`
- **Role**: Database cursor wrappers implementing hashed account/storage cursor traits.
- **Key items**: `DatabaseHashedCursorFactory`, `DatabaseHashedAccountCursor`, `DatabaseHashedStorageCursor`, `is_storage_empty()`, `set_hashed_address()`
- **Interactions**: Reads `HashedAccounts`/`HashedStorages` tables via `reth_db_api` cursors.

### `lib.rs`
- **Role**: Crate entrypoint wiring and re-exports for DB integration traits and cursor factories.
- **Key items**: `DatabaseStateRoot`, `DatabaseHashedPostState`, `DatabaseStorageRoot`, `DatabaseProof`, `DatabaseTrieCursorFactory`, `DatabaseTrieWitness`

### `prefix_set.rs`
- **Role**: Loads account/storage prefix sets from changesets over block ranges.
- **Key items**: `PrefixSetLoader`, `load()`, `load_prefix_sets_with_provider()`, `TriePrefixSets`, `PrefixSetMut`
- **Interactions**: Reads `AccountChangeSets`/`StorageChangeSets` and hashes keys via `KeyHasher`.

### `proof.rs`
- **Role**: Database-specific proof builders that overlay trie input on top of DB state.
- **Key items**: `DatabaseProof`, `DatabaseStorageProof`, `overlay_account_proof()`, `overlay_multiproof()`, `overlay_storage_proof()`, `overlay_storage_multiproof()`
- **Interactions**: Uses `Proof`, `StorageProof`, and `InMemoryTrieCursorFactory` with `TrieInput`.

### `state.rs`
- **Role**: Database-backed state root computation and hashed post-state reverts ingestion.
- **Key items**: `DatabaseStateRoot`, `DatabaseHashedPostState`, `incremental_root()`, `overlay_root()`, `overlay_root_from_nodes_with_updates()`, `from_reverts()`
- **Interactions**: Uses `load_prefix_sets_with_provider`, `HashedPostStateSorted`, and `TrieInputSorted`.
- **Knobs / invariants**: `from_reverts` keeps first occurrence per key and respects range bounds.

### `storage.rs`
- **Role**: Database-backed storage root computation and hashed storage reverts.
- **Key items**: `DatabaseStorageRoot`, `DatabaseHashedStorage`, `overlay_root()`, `from_reverts()`
- **Interactions**: Uses `StorageRoot` with DB cursor factories and `StorageChangeSets` tables.

### `trie_cursor.rs`
- **Role**: Database trie cursor factories for account/storage tries, plus sorted update writes.
- **Key items**: `DatabaseTrieCursorFactory`, `DatabaseAccountTrieCursor`, `DatabaseStorageTrieCursor`, `write_storage_trie_updates_sorted()`
- **Interactions**: Uses `StorageTrieEntry`, `StoredNibbles`, and `StoredNibblesSubKey` for table IO.
- **Knobs / invariants**: `write_storage_trie_updates_sorted` deletes duplicates when `is_deleted` and skips empty nibbles.

### `witness.rs`
- **Role**: Database-specific trie witness generation for a target state overlay.
- **Key items**: `DatabaseTrieWitness`, `overlay_witness()`, `TrieWitness`
- **Interactions**: Builds overlay factories with `TrieInput` and `HashedPostStateCursorFactory`.

## End-to-end flow (high level)
- Load prefix sets from changesets for a block range.
- Build `HashedPostStateSorted` from reverts via `DatabaseHashedPostState`.
- Instantiate `StateRoot`/`StorageRoot` with DB cursor factories and compute roots or updates.
- Overlay cached nodes with `TrieInputSorted` for incremental roots or proof generation.
- Produce account/storage proofs and multiproofs from DB-backed cursors.
- Generate witness maps for target hashed state overlays.
- Cache per-block trie changesets for reorg-safe lookups and explicit eviction.
