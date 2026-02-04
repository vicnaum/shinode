# rocksdb

## Purpose
RocksDB-backed history storage provider and helpers.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: RocksDB provider module entrypoint and exports.
- **Key items**: `RocksDBProvider`, `RocksDBBuilder`, `RocksTx`

### `provider.rs`
- **Role**: RocksDB provider implementation.
- **Key items**: read/write APIs for history storage

### `metrics.rs`
- **Role**: RocksDB metrics helpers.
- **Key items**: metrics structs and recording

### `invariants.rs`
- **Role**: RocksDB invariants and validation helpers.
- **Key items**: invariant checks
