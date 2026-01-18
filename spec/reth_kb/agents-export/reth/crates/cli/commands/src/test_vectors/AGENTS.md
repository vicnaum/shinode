# test_vectors

## Purpose
Implements `reth test-vectors`: CLI tooling to generate and validate small "micro" test vectors for DB tables and `Compact`-encoded types (primarily for codec/DB compatibility checks).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - clap command and subcommand dispatch (`Tables` vs `Compact`).
  - **Key items**: `Command`, `Subcommands`
- `tables.rs` - generates JSON vectors for selected DB tables (including dupsort tables), using proptest-based generation and stable ordering.
  - **Key items**: `generate_vectors()`, `generate_table_vector()`, `generate_dupsort_vector()`, `save_to_file()`, `VECTORS_FOLDER`
- `compact.rs` - generates/reads vectors for many `Compact` types (codecs), using proptest/arbitrary and a macro-driven registry.
  - **Key items**: `compact_types!`, `GENERATE_VECTORS`, `READ_VECTORS`, `VECTORS_FOLDER`, `VECTOR_SIZE`

## Key APIs (no snippets)
- **CLI types**: `Command`, `Subcommands`
- **Generators**: `tables::generate_vectors()`, `compact::generate_vectors()`, `compact::read_vectors()`

## Relationships
- **Feature-gated by**: `reth-cli-commands` `arbitrary` feature (enables the `test_vectors` module).
- **Writes to**: `testdata/micro/*` folders (generated vector artifacts; not part of normal node operation).
