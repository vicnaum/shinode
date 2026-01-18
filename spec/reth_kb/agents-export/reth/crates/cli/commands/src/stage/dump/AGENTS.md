# dump

## Purpose
Implements `reth stage dump`: utilities to export ("dump") selected pipeline stage data for a given block range into a new database for debugging and repro workflows.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `execution.rs` - dump logic for the Execution stage.
  - **Key items**: `dump_execution_stage()`
- `hashing_account.rs` - dump logic for the AccountHashing stage.
  - **Key items**: `dump_hashing_account_stage()`
- `hashing_storage.rs` - dump logic for the StorageHashing stage.
  - **Key items**: `dump_hashing_storage_stage()`
- `merkle.rs` - dump logic for the Merkle stage.
  - **Key items**: `dump_merkle_stage()`
- `mod.rs` - clap command definitions + dispatch across dumpable stages; shared setup for the output DB and initial state.
  - **Key items**: `Command<C>`, `Stages`, `StageCommand`, `setup()`

## Key APIs (no snippets)
- **CLI types**: `Command<C>`, `Stages`, `StageCommand`
- **Stage dump entrypoints**: `dump_execution_stage()`, `dump_hashing_storage_stage()`, `dump_hashing_account_stage()`, `dump_merkle_stage()`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/stage/mod.rs` (`reth stage dump` subcommand).
- **Depends on**: DB tooling (`reth-db*`, `reth-db-common`) and stage implementations (`reth-stages*`) to read input state and write a new output DB.
