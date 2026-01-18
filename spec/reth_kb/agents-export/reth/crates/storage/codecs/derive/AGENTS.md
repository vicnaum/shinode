# derive

## Purpose
`reth-codecs-derive` proc-macro crate for deriving `Compact` codecs and optional zstd/test helpers.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Proc-macro entrypoints and codegen helpers.

### Files
- `Cargo.toml` - Proc-macro crate manifest.
  - **Key items**: `proc-macro = true`, deps `syn`, `quote`, `proc-macro2`

## Key APIs (no snippets)
- **Modules / Packages**: `compact`, `arbitrary`
- **Functions**: `derive()`, `derive_zstd()`, `add_arbitrary_tests`, `generate_tests`

## Relationships
- **Depends on**: `syn` and `quote` for parsing and code generation.
