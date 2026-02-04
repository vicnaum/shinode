# common

## Purpose
Shared utilities for ERA/E2Store formats: common decoding helpers and generic file-format traits used by both `.era` and `.era1` implementations.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for shared utilities.
- **Key items**: `decode`, `file_ops`
- **Interactions**: re-exported/used by `era`, `era1`, and `e2s` layers.
- **Knobs / invariants**: none.

#### `decode.rs`
- **Role**: Defines a small extension trait for decoding typed values from compressed RLP payloads.
- **Key items**: `DecodeCompressedRlp`, `DecodeCompressedRlp::decode<T>()`
- **Interactions**: used by compressed execution/consensus payload wrappers to expose "decompress + RLP decode" as a single call.
- **Knobs / invariants**: error type is normalized to `E2sError`.

#### `file_ops.rs`
- **Role**: Generic era file format traits and naming helpers shared across file types; abstracts "reader/writer" patterns over `Read+Seek` / `Write`.
- **Key items**: `EraFileFormat`, `EraFileId`, `StreamReader`, `FileReader`, `StreamWriter`, `FileWriter`, `EraFileType`, `format_hash()`
- **Interactions**: implemented by `EraFile`/`Era1File` and their readers/writers; provides standardized filename formatting for `.era`/`.era1`.
- **Knobs / invariants**: `EraFileId::ITEMS_PER_ERA` and `era_count()` enforce the "max 8192 items per file" expectation; `EraFileType` determines extension and naming format.

## Key APIs (no snippets)
- **Format traits**: `EraFileFormat`, `EraFileId`
- **I/O traits**: `StreamReader`, `FileReader`, `StreamWriter`, `FileWriter`
- **File typing/naming**: `EraFileType`, `EraFileId::to_file_name()`, `EraFileType::format_filename()`
- **Decode helper**: `DecodeCompressedRlp`

## Relationships
- **Used by**: `e2s` (core TLV records), `era` (consensus/CL), and `era1` (execution/EL) modules to share I/O abstractions and naming conventions.
