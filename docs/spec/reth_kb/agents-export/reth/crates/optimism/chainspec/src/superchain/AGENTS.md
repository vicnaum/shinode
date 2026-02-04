# superchain

## Purpose
Superchain registry support: generate chain specs from bundled superchain configs and genesis files.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring for superchain registry support.
  - **Key items**: re-exports from `chain_specs`
- `chain_specs.rs` - Generated list of superchain (name, environment) pairs.
  - **Key items**: `create_superchain_specs!` invocation
- `configs.rs` - Reads and decompresses genesis/config data from bundled tar archive.
  - **Key items**: `read_superchain_genesis()`
- `chain_metadata.rs` - Metadata structs and conversion to genesis chain config.
  - **Key items**: `ChainMetadata`, `to_genesis_chain_config()`
- `chain_spec_macro.rs` - Macros to create chain specs and superchain enums.
  - **Key items**: `create_chain_spec!`, `create_superchain_specs!`

## Key APIs (no snippets)
- `Superchain` (generated enum), `read_superchain_genesis()`
