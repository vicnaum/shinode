# init_state

## Purpose
Implements `reth init-state`: initializes a data directory from a JSONL state dump, with an optional `--without-evm` mode that creates a dummy EVM chain up to a provided header before importing state.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - clap command definition and execution logic for importing the JSONL state dump into the database/static files.
  - **Key items**: `InitStateCommand<C>`, `init_from_state_dump()`, `EnvironmentArgs<C>`
- `without_evm.rs` - helpers for `--without-evm`: read an RLP header from file and append dummy blocks/static-file ranges before inserting the first "real" block.
  - **Key items**: `read_header_from_file()`, `setup_without_evm()`, `append_dummy_chain()`, `append_first_block()`

## Key APIs (no snippets)
- **CLI type**: `InitStateCommand<C>`
- **Helpers**: `setup_without_evm()`, `read_header_from_file()`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` via the `init_state` module.
- **Depends on**: `reth-db-common::init` (state dump importer), provider/static-file writer APIs (`reth-provider`, `reth-static-file-types`) to materialize state and required metadata.
