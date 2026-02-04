# storage

## Purpose
Storage crates for database access, providers, APIs, and static file formats.

## Contents (one hop)
### Subdirectories
- [x] `codecs/` - Compact codecs and derive helpers.
- [x] `db/` - MDBX-backed DB implementation.
- [x] `db-api/` - Database abstraction traits and table definitions.
- [x] `db-common/` - Shared DB init and tooling helpers.
- [x] `db-models/` - Storage model structs.
- [x] `errors/` - Shared storage error types.
- [x] `libmdbx-rs/` - Rust MDBX wrapper and FFI.
- [x] `nippy-jar/` - Immutable columnar storage format.
- [x] `provider/` - Provider traits and implementations.
- [x] `rpc-provider/` - RPC-backed provider implementation.
- [x] `storage-api/` - Storage access trait definitions.
- [x] `zstd-compressors/` - Zstd dictionary compressors.

## Relationships
- **Used by**: higher-level chain, sync, and RPC components.
