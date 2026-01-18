# types

## Purpose
Data types for `.era1` (execution-layer) files: compressed execution block components and the `.era1` grouping/index/id structures used by readers/writers.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for `.era1` types.
- **Key items**: `execution`, `group`
- **Interactions**: imported by `era1::file` for assembling/parsing complete `.era1` files.
- **Knobs / invariants**: none.

#### `execution.rs`
- **Role**: Execution-layer record wrappers for `.era1`: snappy-framed RLP codecs for block headers/bodies/receipts, plus total difficulty and accumulator records.
- **Key items**: `CompressedHeader`, `CompressedBody`, `CompressedReceipts`, `TotalDifficulty`, `Accumulator`, `BlockTuple`, `SnappyRlpCodec<T>`, `MAX_BLOCKS_PER_ERA1`
- **Interactions**: implements `DecodeCompressedRlp` for compressed types; converts to/from `e2s::types::Entry` using record IDs like `COMPRESSED_HEADER`/`COMPRESSED_BODY`/`COMPRESSED_RECEIPTS`.
- **Knobs / invariants**: `.era1` size/structure constraints depend on accumulator/index; `MAX_BLOCKS_PER_ERA1` (8192) caps per-file block tuples.

#### `group.rs`
- **Role**: Defines `.era1` content grouping and identifiers: `Era1Group` (block tuples + accumulator + block index + extra entries), `BlockIndex` (offset index), and `Era1Id` (file naming metadata).
- **Key items**: `Era1Group`, `BlockIndex`, `Era1Id`, `BLOCK_INDEX`
- **Interactions**: `BlockIndex` implements `IndexEntry`; `Era1Id` implements `EraFileId` for standardized naming.
- **Knobs / invariants**: block index offsets length must match block count; file naming is derived from start block and (optional) hash.

## Key APIs (no snippets)
- **Compressed execution payloads**: `CompressedHeader`, `CompressedBody`, `CompressedReceipts`
- **Block tuple/metadata**: `BlockTuple`, `TotalDifficulty`, `Accumulator`
- **Grouping/indexing**: `Era1Group`, `BlockIndex`
- **File identity**: `Era1Id`

## Relationships
- **Used by**: `era1::file` to parse/assemble `.era1` streams and to enforce structure ordering (block tuples -> extras -> accumulator -> block index).
