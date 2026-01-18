# src

## Purpose
Rust wrapper around libmdbx with safe environment, transaction, and cursor APIs.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports.
- **Key items**: `Environment`, `Transaction`, `Cursor`, `Database`, `Error`

### `environment.rs`
- **Role**: MDBX environment builder and runtime handles.
- **Key items**: `Environment`, `EnvironmentBuilder`, `Geometry`, `Stat`, `Info`

### `transaction.rs`
- **Role**: Transaction wrappers for RO/RW operations.
- **Key items**: `Transaction`, `TransactionKind`, `RO`, `RW`, `CommitLatency`

### `cursor.rs`
- **Role**: Cursor wrappers and iterators.
- **Key items**: `Cursor`, `Iter`, `IterDup`

### `database.rs`
- **Role**: Database handle wrappers and DBI helpers.
- **Key items**: `Database`, DBI helpers

### `flags.rs`
- **Role**: MDBX flag types and bitflags.
- **Key items**: env/txn/db flags

### `codec.rs`
- **Role**: Encoding helpers for MDBX values.
- **Key items**: codec helpers and traits

### `error.rs`
- **Role**: Error types for MDBX wrapper.
- **Key items**: `Error`, `Result`

### `txn_manager.rs`
- **Role**: Transaction manager utilities.
- **Key items**: read-transaction tracking

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Environment`, `Transaction`, `Cursor`, `Database`

## Relationships
- **Depends on**: `mdbx-sys` for FFI bindings.
