# hash_builder

## Purpose
Persistable hash-builder utilities that capture and restore trie hash builder state for reuse.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for hash builder helpers.
- **Key items**: `HashBuilderState`, `alloy_trie::hash_builder::*`
- **Interactions**: Re-exported by `src/lib.rs` for crate consumers.

### `state.rs`
- **Role**: Serializable snapshot of `HashBuilder` internals for storing/resuming trie computation.
- **Key items**: `HashBuilderState`, `From<HashBuilderState> for HashBuilder`, `From<HashBuilder> for HashBuilderState`, `Compact` impl
- **Interactions**: Uses `TrieMask`, `RlpNode`, `HashBuilderValue` and `reth_codecs` for compact encoding.
- **Knobs / invariants**: Preserves stack/mask ordering; `stored_in_database` flag is round-tripped in compact form.

## End-to-end flow (high level)
- Build a `HashBuilder` while computing trie updates.
- Convert to `HashBuilderState` to snapshot the builder state.
- Encode/decode via `reth_codecs::Compact` when persisting or loading.
- Rehydrate into `HashBuilder` to resume hashing or proof generation.
