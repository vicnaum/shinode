# benches

## Purpose
Criterion benchmarks for MDBX table operations and serialization paths.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `criterion.rs`
- **Role**: Main benchmark harness for table operations and serialization.
- **Key items**: `db()`, `serialization()`, `measure_table_db()`, `measure_table_serialization()`
- **Interactions**: Uses `utils.rs` helpers to load test vectors and open temp DBs.

### `get.rs`
- **Role**: Benchmarks point lookup performance for MDBX tables.
- **Key items**: `bench_get`, `DbCursorRO`, table-specific loops

### `hash_keys.rs`
- **Role**: Benchmarks hashed key generation for table workloads.
- **Key items**: hashing helpers, dataset generation

### `put.rs`
- **Role**: Benchmarks insert/write performance for MDBX tables.
- **Key items**: `DbTxMut`, `cursor_write`, insert loops

### `utils.rs`
- **Role**: Benchmark data generation and table vector loading.
- **Key items**: `load_vectors()`, test data builders, `BENCH_DB_PATH`

### `README.md`
- **Role**: Notes on running DB benchmarks and codec benchmarks.
- **Key items**: `cargo bench --features bench`

## End-to-end flow (high level)
- Generate test vectors and open a temp database.
- Run serialization and CRUD benchmarks across core tables.
- Record timing per table and operation.
