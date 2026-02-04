# src

## Purpose
Static file type definitions: segments, headers, compression, and producer events.

## Contents (one hop)
### Subdirectories
- [x] `snapshots/` - (skip: test snapshot fixtures).

### Files
- `lib.rs` - Crate entrypoint and shared static-file types.
  - **Key items**: `StaticFileSegment`, `SegmentHeader`, `StaticFileTargets`, `HighestStaticFiles`, `DEFAULT_BLOCKS_PER_STATIC_FILE`
- `segment.rs` - Static file segment enum and segment header/offset types.
  - **Key items**: `StaticFileSegment`, `SegmentHeader`, `SegmentRangeInclusive`, `SegmentConfig`
  - **Knobs / invariants**: Segment order is significant; filename format must stay in sync.
- `event.rs` - Producer lifecycle events.
  - **Key items**: `StaticFileProducerEvent`
- `compression.rs` - Compression type enum for static files.
  - **Key items**: `Compression`

## Key APIs (no snippets)
- `StaticFileSegment`, `SegmentHeader`, `SegmentRangeInclusive`
- `StaticFileTargets`, `StaticFileProducerEvent`
