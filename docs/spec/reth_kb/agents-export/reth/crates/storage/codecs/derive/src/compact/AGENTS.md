# compact

## Purpose
Code generation helpers for the `Compact` derive macro, including field analysis and bitflag
struct generation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Entry point for `Compact` derive codegen, wiring helpers and field analysis.
- **Key items**: `derive()`, `StructFieldDescriptor`, `FieldTypes`, `get_fields()`,
  `get_bit_size()`
- **Interactions**: Delegates to `flags`, `structs`, `enums`, and `generator`.

### `enums.rs`
- **Role**: Codegen helpers for enum Compact implementations.
- **Key items**: enum variant handling, `generate_from_to` helpers (enum paths)
- **Interactions**: Used by `mod.rs` during derive expansion for enums.

### `flags.rs`
- **Role**: Builds bitflag structs and metadata for compacted fields.
- **Key items**: `generate_flag_struct()`, `bitflag_encoded_bytes()`, `bitflag_unused_bits()`
- **Interactions**: Uses `modular_bitfield` and `get_bit_size()` for flag sizing.
- **Knobs / invariants**: Pads flag structs to full bytes and tracks unused bits for extensibility.

### `generator.rs`
- **Role**: Emits `to_compact`/`from_compact` implementations and optional zstd wrapping.
- **Key items**: `generate_from_to()`, zstd handling, field encode/decode paths
- **Interactions**: Consumes field descriptors and flags generated in `mod.rs`.

### `structs.rs`
- **Role**: Codegen helpers for struct Compact implementations.
- **Key items**: struct field ordering, special-casing `Option`/`Vec` fields, flag extraction
- **Interactions**: Used by `generator.rs` when emitting struct encode/decode logic.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StructFieldDescriptor`, `FieldTypes`
- **Modules / Packages**: `flags`, `generator`, `structs`, `enums`
- **Functions**: `derive()`, `get_fields()`, `get_bit_size()`, `generate_flag_struct()`

## Relationships
- **Depends on**: `syn`/`quote` for AST parsing and code generation.
- **Data/control flow**: field descriptors feed flag generation, then encoder/decoder codegen.

## End-to-end flow (high level)
- Parse derive input into field descriptors.
- Generate flag structs and metadata for compacted fields.
- Emit `to_compact` and `from_compact` implementations (optionally with zstd wrapping).
