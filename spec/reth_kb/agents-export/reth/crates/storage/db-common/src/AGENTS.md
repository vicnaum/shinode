# src

## Purpose
Shared storage initialization and DB tooling utilities.

## Contents (one hop)
### Subdirectories
- [x] `db_tool/` - CLI/tooling helpers for table inspection.

### Files
- `lib.rs` - Crate entrypoint.
  - **Key items**: `init`, `db_tool` exports
- `init.rs` - Genesis and storage initialization routines.
  - **Key items**: `init_genesis()`, `init_genesis_with_settings()`, `InitStorageError`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `InitStorageError`, `StorageSettings`
- **Modules / Packages**: `init`, `db_tool`

## Relationships
- **Depends on**: `reth_provider` and `reth_db_api` for storage read/write.
