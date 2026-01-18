# segments

## Purpose
Static file segment implementations for copying data from the database.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Segment trait definition and module wiring.
- **Key items**: `Segment`, `StaticFileSegment`

### `receipts.rs`
- **Role**: Segment that copies receipts into static files.
- **Key items**: `Receipts`
- **Interactions**: Uses `StaticFileProviderFactory` and receipt cursors to append data.

## Key APIs (no snippets)
- `Segment`, `Receipts`
