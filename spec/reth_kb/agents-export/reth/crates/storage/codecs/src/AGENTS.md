# src

## Purpose
Core Compact codec trait and utilities plus alloy-specific codec implementations.

## Contents (one hop)
### Subdirectories
- [x] `alloy/` - Compact codecs for alloy consensus/primitives types and transactions.

### Files
- `lib.rs` - Defines the `Compact` trait and core encoding/decoding helpers.
  - **Key items**: `Compact`, `CompactPlaceholder`, `encode_varuint()`, `decode_varuint()`,
    `specialized_to_compact()`
- `private.rs` - Internal re-exports for derive macro expansion.
  - **Key items**: `modular_bitfield`, `bytes::Buf`
- `test_utils.rs` - Test helpers and macros for backwards compatibility checks.
  - **Key items**: `validate_bitflag_backwards_compat!`, `UnusedBits`, `test_decode()`
- `txtype.rs` - Transaction type identifier constants for compact encoding.
  - **Key items**: `COMPACT_IDENTIFIER_LEGACY`, `COMPACT_IDENTIFIER_EIP2930`,
    `COMPACT_EXTENDED_IDENTIFIER_FLAG`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Compact`
- **Modules / Packages**: `alloy`, `txtype`
- **Functions**: `to_compact()`, `from_compact()`, `decode_varuint()`

## Relationships
- **Depends on**: `reth-codecs-derive` for generated `Compact` implementations.
- **Data/control flow**: `Compact` trait is implemented for core primitives and alloy types,
  enabling storage serialization across crates.
