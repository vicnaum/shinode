# era1

## Purpose
Integration tests for `.era1` (execution-layer) files: validate genesis handling and full roundtrip read/decode/write behavior against downloaded fixtures (mainnet/sepolia samples).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - test module wiring.
  - **Key items**: `genesis`, `roundtrip`
- `genesis.rs` - tests around genesis-era `.era1` file structure/edge cases.
  - **Key items**: (see tests in file)
- `roundtrip.rs` - roundtrip tests for `.era1` files (read -> decompress/decode -> re-encode/recompress -> write -> re-read and compare).
  - **Key items**: `Era1Reader`, `Era1Writer`, `BlockTuple`, `CompressedHeader`, `CompressedBody`, `CompressedReceipts`

## Relationships
- **Driven by**: `tests/it/main.rs` helper that downloads fixtures into a temp dir and opens them via `Era1Reader`.
