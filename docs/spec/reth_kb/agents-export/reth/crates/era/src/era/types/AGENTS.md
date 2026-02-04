# types

## Purpose
Data types for `.era` (consensus-layer) files: compressed beacon block/state record wrappers and the `.era` content grouping/index/id types used by readers/writers.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Module wiring for `.era` types.
- **Key items**: `consensus`, `group`
- **Interactions**: imported by `era::file` for assembling/parsing complete `.era` files.
- **Knobs / invariants**: none.

#### `consensus.rs`
- **Role**: Snappy-compressed SSZ payload wrappers for post-merge consensus data (signed beacon blocks and beacon states), with bounded decompression for safety.
- **Key items**: `CompressedSignedBeaconBlock`, `CompressedBeaconState`, `COMPRESSED_SIGNED_BEACON_BLOCK`, `COMPRESSED_BEACON_STATE`, `decompress()`, `from_ssz()`
- **Interactions**: converted to/from `e2s::types::Entry` when reading/writing `.era` files; external SSZ decoding into concrete beacon types is intentionally out-of-scope (caller responsibility).
- **Knobs / invariants**: bounded decompression limits (e.g. max decompressed beacon state size) to avoid pathological inputs.

#### `group.rs`
- **Role**: Defines `.era` file content grouping and identifiers: `EraGroup` (blocks + era-state + indices + extra entries), `SlotIndex` (offset index), and `EraId` (file naming metadata).
- **Key items**: `EraGroup`, `SlotIndex`, `EraId`, `SLOTS_PER_HISTORICAL_ROOT`
- **Interactions**: `SlotIndex` implements `IndexEntry` for e2store; `EraId` implements `EraFileId` for standardized naming.
- **Knobs / invariants**: genesis era is represented by absence of `slot_index` and empty block list; `slot_range()` drives ID construction.

## Key APIs (no snippets)
- **Compressed consensus payloads**: `CompressedSignedBeaconBlock`, `CompressedBeaconState`
- **Grouping/indexing**: `EraGroup`, `SlotIndex`
- **File identity**: `EraId`

## Relationships
- **Used by**: `era::file` to parse/assemble `.era` streams and to enforce structure ordering (blocks -> state -> indices).
