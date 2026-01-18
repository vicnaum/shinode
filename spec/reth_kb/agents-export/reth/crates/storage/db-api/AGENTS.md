# db-api

## Purpose
Database abstraction traits, table definitions, and codec integrations for reth storage.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Core traits, models, table definitions, and utilities.

### Files
- `Cargo.toml` - Crate manifest for DB API.
  - **Key items**: features for codecs, fuzzing, and table definitions.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Database`, `DbTx`, `DbCursorRO`, `Table`, `Tables`
- **Modules / Packages**: `models`, `tables`

## Relationships
- **Used by**: `reth-db` and provider crates to implement storage backends.
