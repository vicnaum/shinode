# src

## Purpose
MDBX-backed database implementation for reth, including environment management, metrics, and
static file helpers.

## Contents (one hop)
### Subdirectories
- [x] `implementation/` - Backend-specific implementations (MDBX).
- [x] `static_file/` - Static file cursor and mask helpers.

### Files
- `lib.rs` - Crate entrypoint wiring MDBX implementation and test utilities.
  - **Key items**: `DatabaseEnv`, `DatabaseEnvKind`, `create_db()`, `init_db()`, `open_db()`
- `lockfile.rs` - Storage lockfile helpers.
  - **Key items**: `StorageLock`
- `mdbx.rs` - Convenience helpers for creating/opening MDBX environments.
  - **Key items**: `create_db()`, `init_db_for()`, `open_db_read_only()`
- `metrics.rs` - Database operation and transaction metrics.
  - **Key items**: `DatabaseEnvMetrics`, `TransactionMode`, `TransactionOutcome`
- `utils.rs` - Environment helper utilities.
  - **Key items**: `default_page_size()`, `is_database_empty()`
- `version.rs` - Database version file management.
  - **Key items**: `DB_VERSION`, `check_db_version_file()`, `create_db_version_file()`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseEnv`, `DatabaseArguments`, `DatabaseEnvMetrics`
- **Modules / Packages**: `mdbx`, `static_file`
- **Functions**: `create_db()`, `init_db()`, `open_db()`, `is_database_empty()`

## Relationships
- **Depends on**: `reth-db-api` for database traits and table sets.
- **Depends on**: `reth-libmdbx` for MDBX bindings (feature gated).
- **Data/control flow**: MDBX environment -> transactions -> cursors -> table encode/decode.
