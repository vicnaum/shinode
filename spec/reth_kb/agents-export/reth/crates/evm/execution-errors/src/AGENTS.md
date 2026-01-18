# src

## Purpose
Implements `reth-execution-errors`: shared error types used by block execution and state/trie computations (including state root/proof/trie witness error families), and re-exports common EVM execution/validation errors.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports `alloy_evm::block` execution/validation error types and exposes the trie/state-root related error module.
- **Key items**: `BlockExecutionError`, `BlockValidationError`, `InternalBlockExecutionError`

#### `trie.rs`
- **Role**: Error types for state root, storage root, state proofs, sparse trie operations, and trie witness building.
- **Key items**: `StateRootError`, `StorageRootError`, `StateProofError`, `SparseStateTrieError`/`SparseStateTrieErrorKind`, `SparseTrieError`/`SparseTrieErrorKind`, `TrieWitnessError`
- **Interactions**: bridges errors into `reth_storage_errors::{DatabaseError, ProviderError}` where applicable.

## Key APIs (no snippets)
- Error families: `StateRootError`, `TrieWitnessError`, `SparseStateTrieError`
