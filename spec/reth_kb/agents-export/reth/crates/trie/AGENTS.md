# trie

## Purpose
Trie subsystem crates: common data types, database-backed access, sparse trie variants, and parallelized root/proof computation.

## Contents (one hop)
### Subdirectories
- [x] `common/` - Shared trie types: hashed state overlays, proofs, updates, prefix sets, and root helpers.
- [x] `db/` - Database-backed trie integration: cursor factories, roots/proofs, and changeset caching.
- [x] `parallel/` - Parallel state root/proof computation with worker pools and metrics.
- [x] `sparse/` - Sparse MPT implementation with lazy reveal, state trie wrapper, and update tracking.
- [x] `sparse-parallel/` - Parallel sparse MPT using upper/lower subtries and update thresholds.
- [x] `trie/` - Merkle trie core: root computation, cursors, proofs, witnesses, and verification.

### Files
- (none)

## Key APIs (no snippets)
- **Modules / Packages**: `common`, `db`, `sparse`, `parallel`, `trie` - coordinated crates for trie storage, computation, and proofs.

## Relationships
- **Depends on**: `common` types are shared by the other trie crates in this subtree.
