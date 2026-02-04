# src

## Purpose
Proc-macro implementation for deriving `Compact` codecs and generating arbitrary/roundtrip tests.

## Contents (one hop)
### Subdirectories
- [x] `compact/` - `Compact` derive codegen helpers and flag struct generation.

### Files
- `arbitrary.rs` - Generates optional proptest roundtrip tests for `Compact`/RLP encodings.
  - **Key items**: `maybe_generate_tests()`, proptest config handling, `add_arbitrary_tests`
- `lib.rs` - Proc-macro entrypoints for `Compact`, `CompactZstd`, and test generation helpers.
  - **Key items**: `derive()`, `derive_zstd()`, `add_arbitrary_tests`, `generate_tests`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ZstdConfig`
- **Modules / Packages**: `compact`, `arbitrary`
- **Functions**: `derive()`, `derive_zstd()`, `add_arbitrary_tests`, `generate_tests`

## Relationships
- **Depends on**: `syn` and `quote` for macro parsing and code generation.
- **Data/control flow**: `Compact` derive dispatches into `compact` helpers; test generation uses
  `arbitrary` to emit proptest modules.
