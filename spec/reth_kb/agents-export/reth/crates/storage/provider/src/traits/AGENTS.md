# traits

## Purpose
Provider trait definitions and factory abstractions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Trait re-exports and module wiring.
- **Key items**: `FullProvider`, `StaticFileProviderFactory`, `RocksDBProviderFactory`

### `full.rs`
- **Role**: Full provider trait combining common provider capabilities.
- **Key items**: `FullProvider`

### `rocksdb_provider.rs`
- **Role**: RocksDB provider factory trait.
- **Key items**: `RocksDBProviderFactory`

### `static_file_provider.rs`
- **Role**: Static file provider factory trait.
- **Key items**: `StaticFileProviderFactory`
