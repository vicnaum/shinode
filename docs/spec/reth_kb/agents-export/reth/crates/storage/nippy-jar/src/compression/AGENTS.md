# compression

## Purpose
Compression backends and traits for NippyJar columnar storage.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Compression trait and enum wrapper.
- **Key items**: `Compression`, `Compressors`

### `lz4.rs`
- **Role**: LZ4 compression backend.
- **Key items**: `Lz4`

### `zstd.rs`
- **Role**: Zstd compression backend and dictionary support.
- **Key items**: `Zstd`, `Decompressor`, `DecoderDictionary`
