# src

## Purpose
Shared storage error types for database, provider, and lockfile layers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports.
- **Key items**: `db`, `lockfile`, `provider`, `any`

### `db.rs`
- **Role**: Database error types and log level helpers.
- **Key items**: `DatabaseError`, `DatabaseErrorInfo`, `DatabaseWriteError`, `LogLevel`

### `lockfile.rs`
- **Role**: Storage lockfile error definitions.
- **Key items**: `StorageLockError`

### `provider.rs`
- **Role**: Provider-layer error types and helpers.
- **Key items**: `ProviderError`, `ProviderResult`

### `any.rs`
- **Role**: Cloneable error wrapper for arbitrary errors.
- **Key items**: `AnyError`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseError`, `ProviderError`, `StorageLockError`
