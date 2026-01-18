# static_file

## Purpose
Static file provider, writer, and manager for immutable segment storage.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Static file provider entrypoint and exports.
- **Key items**: `StaticFileProvider`, `StaticFileWriter`, `StaticFileAccess`

### `jar.rs`
- **Role**: Jar-level helpers for static files.
- **Key items**: jar reader/writer helpers

### `manager.rs`
- **Role**: Static file manager for segment lifecycle.
- **Key items**: manager struct and segment registry

### `metrics.rs`
- **Role**: Metrics for static file operations.
- **Key items**: metric recording helpers

### `writer.rs`
- **Role**: Static file writer implementation.
- **Key items**: `StaticFileWriter`
