# src

## Purpose
Storage model types shared by database layers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and module re-exports.
- **Key items**: `AccountBeforeTx`, `StoredBlockBodyIndices`, `ClientVersion`

### `accounts.rs`
- **Role**: Account pre-transaction models.
- **Key items**: `AccountBeforeTx`

### `blocks.rs`
- **Role**: Block-related storage models.
- **Key items**: `StoredBlockBodyIndices`, `StoredBlockWithdrawals`,
  `StaticFileBlockWithdrawals`

### `client_version.rs`
- **Role**: Client version model used by storage metadata.
- **Key items**: `ClientVersion`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `AccountBeforeTx`, `StoredBlockBodyIndices`, `ClientVersion`

## Relationships
- **Used by**: `db-api` model definitions and table value types.
