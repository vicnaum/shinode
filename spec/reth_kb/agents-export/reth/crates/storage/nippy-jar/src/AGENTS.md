# src

## Purpose
Immutable columnar storage format (NippyJar) with cursor, writer, and consistency checks.

## Contents (one hop)
### Subdirectories
- [x] `compression/` - Compression backends and traits.

### Files
- `lib.rs` - Core NippyJar type and helpers.
  - **Key items**: `NippyJar`, `NippyJarHeader`, `Compressors`, `NippyJarError`
- `cursor.rs` - Read cursor for NippyJar rows.
  - **Key items**: `NippyJarCursor`
- `writer.rs` - Builder/writer for NippyJar files.
  - **Key items**: `NippyJarWriter`
- `consistency.rs` - Consistency checker for jar contents.
  - **Key items**: `NippyJarChecker`
- `error.rs` - NippyJar error types.
  - **Key items**: `NippyJarError`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `NippyJar`, `NippyJarCursor`, `NippyJarWriter`

## Relationships
- **Used by**: static file providers and storage segments.
