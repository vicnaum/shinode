# src

## Purpose
Static file producer implementation for moving finalized data into static files.

## Contents (one hop)
### Subdirectories
- [x] `segments/` - Segment implementations for static file copying.

### Files
- `lib.rs` - Crate wiring and re-exports for static file producer types.
  - **Key items**: `StaticFileProducer`, `StaticFileProducerInner`
- `static_file_producer.rs` - Producer logic for selecting targets and copying data.
  - **Key items**: `StaticFileProducer`, `StaticFileProducerInner`, `StaticFileProducerResult`
  - **Interactions**: Emits `StaticFileProducerEvent` and updates static file indexes.

## Key APIs (no snippets)
- `StaticFileProducer`, `StaticFileProducerInner`
