# e2s

## Purpose
Core e2store (`.e2s`) primitives: TLV record encoding/decoding, buffered readers/writers, and shared error types used as the foundation for `.era` and `.era1` file formats.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for the core e2store building blocks.
- **Key items**: `error`, `file`, `types`
- **Interactions**: consumed by `era`/`era1` readers and writers as the underlying record format.
- **Knobs / invariants**: none.

#### `error.rs`
- **Role**: Unified error enum for e2store operations (I/O, SSZ record parsing, snappy compression/decompression, RLP).
- **Key items**: `E2sError`
- **Interactions**: used across all modules to normalize failures while reading/writing and compressing/decompressing entries.
- **Knobs / invariants**: distinguishes "format invariant" errors like `ReservedNotZero`.

#### `types.rs`
- **Role**: Defines the e2store TLV record layout (`Header` + `Entry`), special record IDs (version/index), and an `IndexEntry` helper for serializing offset indices.
- **Key items**: `Header`, `Entry`, `Version`, `IndexEntry`, `VERSION`, `SLOT_INDEX`
- **Interactions**: `Entry::read()` / `Entry::write()` are used by `E2StoreReader`/`E2StoreWriter`; `IndexEntry` is implemented by `SlotIndex`/`BlockIndex` for `.era`/`.era1`.
- **Knobs / invariants**: header `reserved` must be zero; `IndexEntry::from_entry()` validates size and count layout.

#### `file.rs`
- **Role**: Buffered file reader/writer for sequences of e2store `Entry` records, including version handling.
- **Key items**: `E2StoreReader<R>`, `E2StoreWriter<W>`, `read_version()`, `read_next_entry()`, `write_version()`, `write_entry()`
- **Interactions**: `EraReader`/`EraWriter` and `Era1Reader`/`Era1Writer` build on top of this abstraction.
- **Knobs / invariants**: writer automatically inserts the version record before the first non-version entry if not written explicitly.

## Key APIs (no snippets)
- **Records**: `Header`, `Entry`, `Version`
- **I/O**: `E2StoreReader`, `E2StoreWriter`
- **Errors**: `E2sError`
- **Index helper**: `IndexEntry`

## Relationships
- **Used by**: `.era` and `.era1` implementations as the low-level record format and I/O layer.
