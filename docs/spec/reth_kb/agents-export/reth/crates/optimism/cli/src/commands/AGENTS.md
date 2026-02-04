# commands

## Purpose
Optimism CLI commands for node launch, imports, init-state, and test vectors.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - CLI command enum wiring for op-reth.
  - **Key items**: `Commands`
- `import.rs` - Import pre-Bedrock OP blocks from file.
  - **Key items**: `ImportOpCommand`
- `import_receipts.rs` - Import OP receipts from file into static files.
  - **Key items**: `ImportReceiptsOpCommand`, `import_receipts_from_reader()`
- `init_state.rs` - Initialize state from dump with optional OVM scaffolding.
  - **Key items**: `InitStateCommandOp`
- `test_vectors.rs` - (feature `dev`) Generate OP test vectors.
  - **Key items**: `Command`, `Subcommands`

## Key APIs (no snippets)
- `Commands`, `ImportOpCommand`, `ImportReceiptsOpCommand`
