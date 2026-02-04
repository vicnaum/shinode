# src

## Purpose
Stateless execution and validation helpers that verify blocks using witness data instead of a full database.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint, public exports, and `StatelessInput` serialization helper.
- **Key items**: `StatelessInput`, `ExecutionWitness`, `stateless_validation()`, `stateless_validation_with_trie()`
- **Interactions**: Re-exports `StatelessTrie` and `UncompressedPublicKey`.

### `recover_block.rs`
- **Role**: Signature verification and sender recovery using provided public keys.
- **Key items**: `UncompressedPublicKey`, `recover_block_with_public_keys()`
- **Knobs / invariants**: Enforces Homestead signature normalization; requires matching tx/key counts.

### `trie.rs`
- **Role**: Stateless trie abstraction and sparse trie implementation backed by witness data.
- **Key items**: `StatelessTrie`, `StatelessSparseTrie`, `verify_execution_witness()`, `calculate_state_root()`
- **Interactions**: Uses `SparseStateTrie` and validates witness completeness for account/storage lookups.
- **Knobs / invariants**: Errors on incomplete witnesses; pre-state root must match.

### `validation.rs`
- **Role**: Stateless validation pipeline orchestration and error types.
- **Key items**: `StatelessValidationError`, `stateless_validation()`, `stateless_validation_with_trie()`
- **Interactions**: Executes blocks via `reth-evm`, validates consensus, and checks state roots.
- **Knobs / invariants**: Enforces `BLOCKHASH` ancestor limit (256) and contiguous ancestor headers.

### `witness_db.rs`
- **Role**: `revm::Database` implementation backed by stateless trie + witness bytecode/hashes.
- **Key items**: `WitnessDatabase`
- **Knobs / invariants**: Missing bytecode/hash entries produce errors; assumes contiguous ancestor hashes.

## End-to-end flow (high level)
- Deserialize `StatelessInput` containing block, witness, and chain config.
- Recover senders using `recover_block_with_public_keys`.
- Build a `StatelessTrie` from witness data and verify pre-state root.
- Initialize `WitnessDatabase` with trie, bytecode map, and ancestor hashes.
- Execute the block with `reth-evm` and validate consensus/post-exec checks.
- Recompute the post-state root and compare to the block header.

## Key APIs (no snippets)
- `stateless_validation()`, `stateless_validation_with_trie()`
- `StatelessTrie`, `StatelessSparseTrie`, `WitnessDatabase`
- `StatelessValidationError`, `UncompressedPublicKey`
