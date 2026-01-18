# tests

## Purpose
Integration and property tests for database-backed trie cursors, roots, proofs, walkers, and witnesses.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `fuzz_in_memory_nodes.rs`
- **Role**: Property tests comparing in-memory overlay roots against expected roots for accounts and storage.
- **Key items**: `fuzz_in_memory_account_nodes`, `fuzz_in_memory_storage_nodes`, `InMemoryTrieCursorFactory`, `StateRoot::from_tx`, `StorageRoot::from_tx_hashed`
- **Interactions**: Extends `TrieUpdates` with in-memory nodes and validates expected roots.

### `post_state.rs`
- **Role**: Validates hashed cursor ordering and precedence between DB and post-state overlays.
- **Key items**: `assert_account_cursor_order`, `assert_storage_cursor_order`, `storage_is_empty`, `storage_cursor_correct_order`, `zero_value_storage_entries_are_discarded`, `wiped_storage_is_discarded`
- **Interactions**: Uses `HashedPostStateCursorFactory` with `DatabaseHashedCursorFactory`.

### `proof.rs`
- **Role**: Verifies account and storage proofs against known genesis/testspec fixtures.
- **Key items**: `testspec_proofs`, `testspec_empty_storage_proof`, `mainnet_genesis_account_proof`, `holesky_deposit_contract_proof`, `DatabaseProof`
- **Interactions**: Compares computed proofs to expected byte sequences and calls `verify()`.

### `trie.rs`
- **Role**: End-to-end root computation tests (full, incremental, extension-node scenarios).
- **Key items**: `incremental_vs_full_root`, `arbitrary_state_root`, `arbitrary_state_root_with_progress`, `account_and_storage_trie`, `storage_root_regression`, `fuzz_state_root_incremental`
- **Interactions**: Exercises `StateRoot`, `StorageRoot`, `TrieUpdates`, and `TriePrefixSets`.

### `walker.rs`
- **Role**: Tests trie walker traversal ordering and prefix-set behavior for account/storage tries.
- **Key items**: `walk_nodes_with_common_prefix`, `cursor_rootnode_with_changesets`, `TrieWalker`, `PrefixSetMut`
- **Interactions**: Uses `DatabaseAccountTrieCursor` and `DatabaseStorageTrieCursor`.

### `witness.rs`
- **Role**: Validates trie witness generation for empty roots and destroyed storage.
- **Key items**: `includes_empty_node_preimage`, `includes_nodes_for_destroyed_storage_nodes`, `correctly_decodes_branch_node_values`, `DatabaseTrieWitness`
- **Interactions**: Cross-checks witness contents against `Proof` multiproofs.

## End-to-end flow (high level)
- Build test databases/providers and seed hashed accounts/storage.
- Compute roots/updates with prefix sets and compare against expected roots.
- Generate proofs and witnesses from DB-backed factories.
- Assert ordering, deletion, and witness completeness invariants.
- Use proptest fuzzing to validate cursor/overlay behavior.
