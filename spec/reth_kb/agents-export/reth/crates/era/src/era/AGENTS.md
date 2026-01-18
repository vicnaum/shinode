# era

## Purpose
Implements `.era` (consensus-layer) file support: reading/writing the era file structure on top of e2store records, and assembling the parsed stream into an `EraFile` (blocks + era-state + indices).

## Contents (one hop)
### Subdirectories
- [x] `types/` - compressed consensus payload wrappers and `.era` group/index/id types.

### Files
- `mod.rs` - module wiring for `.era` primitives.
  - **Key items**: `file`, `types`
- `file.rs` - core `.era` file model plus streaming reader/writer implementations built on `E2StoreReader`/`E2StoreWriter`.
  - **Key items**: `EraFile`, `EraReader<R>`, `EraWriter<W>`, `BeaconBlockIterator<R>`, `EraReader::read_and_assemble()`

## Key APIs (no snippets)
- **Models**: `EraFile`
- **I/O**: `EraReader`, `EraWriter`, `BeaconBlockIterator`

## Relationships
- **Builds on**: `e2s` record format and `common::file_ops` trait layer.
- **Produces/consumes**: `types::EraGroup` and compressed consensus payload entries.
