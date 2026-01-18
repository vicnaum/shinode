# header

## Purpose
Header wrapper types, mutable header helpers, and header test utilities.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and re-exports for header types.
- **Key items**: `SealedHeader`, `SealedHeaderFor`, `HeaderMut`
- **Interactions**: Exposes `test_utils` under test/features.

### `sealed.rs`
- **Role**: Sealed header wrapper with lazy hash caching.
- **Key items**: `SealedHeader`, `SealedHeaderFor`, `hash()`, `seal_slow()`
- **Knobs / invariants**: Hash computed lazily; equality/hash are based on cached hash.

### `header_mut.rs`
- **Role**: Mutable header trait for testing and mocks.
- **Key items**: `HeaderMut`, `set_parent_hash()`, `set_timestamp()`

### `test_utils.rs`
- **Role**: Header test helpers and proptest strategies.
- **Key items**: `generate_valid_header()`, `valid_header_strategy()`, `TestHeader`

## End-to-end flow (high level)
- Wrap headers in `SealedHeader` to cache hashes.
- Use `HeaderMut` in tests to tweak header fields.
- Generate fork-valid headers for testing via `test_utils`.

## Key APIs (no snippets)
- `SealedHeader`, `HeaderMut`
