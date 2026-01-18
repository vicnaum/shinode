# src

## Purpose
Provider traits and implementations for accessing chain state, history, and static files.

## Contents (one hop)
### Subdirectories
- [x] `changesets_utils/` - Changeset utilities and iterators.
- [x] `providers/` - Database, static file, state, and RocksDB providers.
- [x] `test_utils/` - Test helpers and mocks.
- [x] `traits/` - Provider trait definitions and factories.
- [x] `writer/` - Write-path tests.

### Files
- `lib.rs` - Crate entrypoint and re-exports.
  - **Key items**: provider exports, `to_range()`
- `changeset_walker.rs` - Changeset iteration over static files.
  - **Key items**: `StaticFileAccountChangesetWalker`
- `either_writer.rs` - Abstraction over writers with different destinations.
  - **Key items**: `EitherWriter` and destination types

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ProviderFactory`, `DatabaseProvider`, `StaticFileWriter`
