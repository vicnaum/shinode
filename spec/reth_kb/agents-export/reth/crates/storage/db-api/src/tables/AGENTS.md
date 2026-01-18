# tables

## Purpose
Defines database tables, table metadata, and raw table access types.

## Contents (one hop)
### Subdirectories
- [x] `codecs/` - Codec integration and fuzz targets.

### Files
- `mod.rs` - Table definitions and table set utilities.
  - **Key items**: `TableType`, `TableViewer`, `TableSet`, `tables!` macro, `Tables` enum
- `raw.rs` - Raw table wrappers for delayed encoding/decoding.
  - **Key items**: `RawTable`, `RawDupSort`, `RawKey`, `RawValue`, `TableRawRow`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Tables`, `RawTable`, `RawKey`, `RawValue`
- **Modules / Packages**: `codecs`

## Relationships
- **Depends on**: `models` for table value types.
- **Used by**: DB implementations and providers to select tables and iterate.
