# era1

## Purpose
Implements `.era1` (execution-layer) file support: reading/writing the era1 file structure on top of e2store records, and assembling the parsed stream into an `Era1File` (block tuples + accumulator + block index).

## Contents (one hop)
### Subdirectories
- [x] `types/` - compressed execution payload wrappers and `.era1` group/index/id types.

### Files
- `mod.rs` - module wiring for `.era1` primitives.
  - **Key items**: `file`, `types`
- `file.rs` - core `.era1` file model plus streaming reader/writer implementations built on `E2StoreReader`/`E2StoreWriter`.
  - **Key items**: `Era1File`, `Era1Reader<R>`, `Era1Writer<W>`, `BlockTupleIterator<R>`, `Era1File::get_block_by_number()`

## Key APIs (no snippets)
- **Models**: `Era1File`
- **I/O**: `Era1Reader`, `Era1Writer`, `BlockTupleIterator`

## Relationships
- **Builds on**: `e2s` record format and `common::file_ops` trait layer.
- **Produces/consumes**: `types::Era1Group` and compressed execution payload entries.
