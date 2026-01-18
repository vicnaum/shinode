# benches

## Purpose
Benchmarks for MDBX cursor and transaction performance.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `cursor.rs`
- **Role**: Cursor performance benchmarks.
- **Key items**: benchmark groups for read/write cursor ops

### `transaction.rs`
- **Role**: Transaction-level performance benchmarks.
- **Key items**: begin/commit/put loops

### `utils.rs`
- **Role**: Benchmark helpers and dataset generation.
- **Key items**: env setup and data generators
