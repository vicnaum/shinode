# src

## Purpose
Database abstraction layer: traits for transactions, cursors, tables, and table models.

## Contents (one hop)
### Subdirectories
- [x] `models/` - Database model types and key/value encodings.
- [x] `tables/` - Table definitions and raw table wrappers.

### Files
- `lib.rs` - Crate entrypoint wiring modules and re-exports.
  - **Key items**: `Database`, `DbTx`, `DbCursorRO`, `Tables`, `DbTxUnwindExt`
- `common.rs` - Shared result types for cursor/iterator operations.
  - **Key items**: `PairResult`, `IterPairResult`, `ValueOnlyResult`
- `cursor.rs` - Cursor traits and walker iterators.
  - **Key items**: `DbCursorRO`, `DbCursorRW`, `DbDupCursorRO`, `Walker`, `RangeWalker`
- `database.rs` - Core `Database` trait.
  - **Key items**: `Database::tx()`, `Database::tx_mut()`, `view()`, `update()`
- `database_metrics.rs` - Metrics reporting hooks for database backends.
  - **Key items**: `DatabaseMetrics`, `report_metrics()`
- `mock.rs` - Mock database and cursor implementations for tests.
  - **Key items**: `DatabaseMock`, `TxMock`, `CursorMock`
- `table.rs` - Table and codec traits.
  - **Key items**: `Table`, `DupSort`, `Encode`, `Decode`, `Compress`, `Decompress`
- `transaction.rs` - Transaction traits and cursor type aliases.
  - **Key items**: `DbTx`, `DbTxMut`, `CursorTy`, `DupCursorTy`
- `unwind.rs` - Transaction unwind helpers.
  - **Key items**: `DbTxUnwindExt`
- `scale.rs` - SCALE codec integration for select types.
  - **Key items**: `ScaleValue`
- `utils.rs` - Arbitrary helpers for fixed-size types.
  - **Key items**: `impl_fixed_arbitrary!`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Database`, `DbTx`, `DbTxMut`, `DbCursorRO`, `Table`
- **Modules / Packages**: `cursor`, `table`, `tables`, `models`

## Relationships
- **Used by**: `reth-db` and storage providers to implement database backends.
