# mdbx

## Purpose
MDBX-backed implementations of database cursors and transactions for `reth-db`.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: MDBX environment wrapper and configuration for database open/init.
- **Key items**: `DatabaseEnvKind`, `DatabaseArguments`, size constants (`KILOBYTE`, `GIGABYTE`)
- **Interactions**: Wires `cursor` and `tx` modules and exports MDBX env helpers.
- **Knobs / invariants**: `DEFAULT_MAX_READERS`, sync mode, geometry sizing.

### `cursor.rs`
- **Role**: Cursor wrappers implementing `DbCursorRO/RW` and dup-sort cursors.
- **Key items**: `Cursor`, `CursorRO`, `CursorRW`, `decode()`, `compress_to_buf_or_ref!`
- **Interactions**: Uses table `Encode/Decode` and metrics hooks for operations.

### `tx.rs`
- **Role**: Transaction wrapper for MDBX read/write transactions.
- **Key items**: `Tx`, `get_dbi()`, `new_cursor()`, `commit()`, metrics handler types
- **Interactions**: Produces MDBX cursors and records transaction metrics.
- **Knobs / invariants**: Long transaction logging threshold and commit latency recording.

### `utils.rs`
- **Role**: Decode helpers for key/value pairs and values.
- **Key items**: `decoder()`, `decode_value()`, `decode_one()`
- **Interactions**: Shared by cursor implementations for decoding.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `DatabaseArguments`, `DatabaseEnvKind`, `Tx`, `Cursor`
- **Modules / Packages**: `cursor`, `tx`
- **Functions**: `decoder()`, `get_dbi()`, `new_cursor()`

## Relationships
- **Depends on**: `reth_libmdbx` for MDBX bindings and environment types.
- **Depends on**: `reth-db-api` for cursor/transaction traits and table codecs.
- **Data/control flow**: MDBX cursors decode table entries and surface them via `DbCursor` traits.

## End-to-end flow (high level)
- Open MDBX environment and configure geometry/sync options.
- Create transactions and cursors for tables.
- Encode/decode entries and record metrics for operations.
