# db

## Purpose
`reth-db` crate: MDBX-backed implementation of the database abstraction layer with static file
helpers and benchmarks.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Database and serialization benchmarks.
- [x] `src/` - MDBX environment, cursor/tx implementation, metrics, and static file helpers.

### Files
- `Cargo.toml` - Crate manifest for MDBX-backed DB implementation.
  - **Key items**: feature `mdbx`, deps `reth-db-api`, `reth-libmdbx`, `reth-storage-errors`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseEnv`, `DatabaseArguments`, `DbCursorRO`, `DbTx`
- **Modules / Packages**: `mdbx`, `static_file`
- **Functions**: `create_db()`, `init_db()`, `open_db_read_only()`

## Relationships
- **Depends on**: `reth-db-api` for database traits and tables.
- **Depends on**: `reth-libmdbx` for MDBX environment and cursor bindings.
