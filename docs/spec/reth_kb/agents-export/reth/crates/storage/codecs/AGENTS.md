# codecs

## Purpose
`reth-codecs` crate: Compact codec trait, derive macros, and alloy type implementations for
storage serialization.

## Contents (one hop)
### Subdirectories
- [x] `derive/` - Proc-macro derive helpers for Compact and zstd wrappers.
- [x] `src/` - Compact trait, core utilities, and alloy codecs.
- [x] `testdata/` - (skip: JSON fixtures for compact codec tests)

### Files
- `Cargo.toml` - Crate manifest for Compact codecs and feature flags.
  - **Key items**: features `alloy`, `op`, `serde`, `test-utils`; deps `reth-codecs-derive`,
    optional `reth-zstd-compressors`
- `README.md` - Overview of Compact codec design and derive macro usage.
  - **Key items**: `Compact`, `#[reth_codec]`, `#[add_arbitrary_tests]`, backwards compatibility

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Compact`, `CompactPlaceholder`
- **Modules / Packages**: `derive`, `alloy`
- **Functions**: `to_compact()`, `from_compact()`, `decode_varuint()`

## Relationships
- **Depends on**: `reth-codecs-derive` for proc-macro codegen.
- **Depends on**: alloy types for consensus, transactions, and trie codecs.
