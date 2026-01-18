# fuzz

## Purpose
Fuzzing harnesses and curated inputs for table encode/decode round-trips.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Fuzz target macro definitions for encode/decode and compress/decompress.
- **Key items**: `impl_fuzzer_with_input!`, `impl_fuzzer_key!`, `impl_fuzzer_value_with_input!`

### `inputs.rs`
- **Role**: Curated fuzzer input types.
- **Key items**: `IntegerListInput`

## Key APIs (no snippets)
- **Modules / Packages**: `inputs`
