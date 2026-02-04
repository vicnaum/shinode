# providers

## Purpose
Concrete provider implementations backed by database, static files, or RocksDB.

## Contents (one hop)
### Subdirectories
- [x] `database/` - Database-backed provider factory and DB providers.
- [x] `rocksdb/` - RocksDB-backed history providers.
- [x] `state/` - Latest/historical/overlay state providers.
- [x] `static_file/` - Static file provider and writer.

### Files
- `mod.rs` - Provider module wiring and exports.
  - **Key items**: `BlockchainProvider`, `ConsistentProvider`, `StaticFileProvider`
- `blockchain_provider.rs` - High-level blockchain provider implementation.
- `consistent.rs` - Consistent provider wrapper.
- `consistent_view.rs` - Consistent DB view utilities.
  - **Key items**: `ConsistentDbView`, `ConsistentViewError`
- `rocksdb_stub.rs` - Stub module for non-RocksDB builds.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `BlockchainProvider`, `ProviderFactory`
