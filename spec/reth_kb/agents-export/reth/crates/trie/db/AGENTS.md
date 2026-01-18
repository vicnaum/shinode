# db

## Purpose
`reth-trie-db` crate: database-backed trie integration for roots, proofs, witnesses, and changeset caching.

## Contents (one hop)
### Subdirectories
- [x] `src/` - DB cursor factories, prefix set loading, roots/proofs/witnesses, and changeset cache.
- [x] `tests/` - Integration and fuzz tests for cursors, roots, proofs, walkers, and witnesses.

### Files
- `Cargo.toml` - Manifest for DB integration dependencies and optional features.
  - **Key items**: features `metrics`, `serde`, `test-utils`; deps `reth-trie`, `reth-trie-common`, `reth-db-api`, `reth-storage-api`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseStateRoot`, `DatabaseStorageRoot`, `DatabaseProof`, `DatabaseTrieWitness` - DB adapters for root/proof computation.
- **Modules / Packages**: `changesets`, `trie_cursor`, `hashed_cursor` - cache, trie cursors, and hashed state access.
- **Functions**: `compute_block_trie_changesets()`, `compute_block_trie_updates()` - changeset calculation utilities.

## Relationships
- **Depends on**: `reth-trie`, `reth-trie-common`, `reth-db-api`, `reth-storage-api` - trie logic and DB access layers.
- **Data/control flow**: changesets -> prefix sets -> incremental roots/proofs/witnesses using DB cursor factories.
