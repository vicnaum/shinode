# static_file

## Purpose
Static file segment access helpers built on top of nippy-jar archives.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Static file utilities and directory scanning for segment jars.
- **Key items**: `iter_static_files()`, `StaticFileCursor`, `StaticFileMask`
- **Interactions**: Uses `NippyJar` and `StaticFileSegment` headers to enumerate ranges.

### `cursor.rs`
- **Role**: Cursor wrapper for reading static file rows and column subsets.
- **Key items**: `StaticFileCursor`, `KeyOrNumber`, `get_one()`, `get_two()`, `get_three()`
- **Interactions**: Uses column selector traits from `mask.rs` and `NippyJarCursor`.

### `mask.rs`
- **Role**: Column selector traits and macro for static file masks.
- **Key items**: `ColumnSelectorOne`, `ColumnSelectorTwo`, `ColumnSelectorThree`,
  `add_static_file_mask!`

### `masks.rs`
- **Role**: Predefined masks for common static file segments.
- **Key items**: `HeaderMask`, `TotalDifficultyMask`, `BlockHashMask`, `ReceiptMask`,
  `TransactionMask`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `StaticFileCursor`, `KeyOrNumber`
- **Modules / Packages**: `mask`, `masks`
- **Functions**: `iter_static_files()`, `get_one()`, `get_two()`

## Relationships
- **Depends on**: `reth_nippy_jar` for static file storage and cursoring.
- **Data/control flow**: column masks select subsets of row data for typed decoding.

## End-to-end flow (high level)
- Scan static file directory for segment jars.
- Create `StaticFileCursor` over a jar segment.
- Use mask selectors to decode one or more columns.
