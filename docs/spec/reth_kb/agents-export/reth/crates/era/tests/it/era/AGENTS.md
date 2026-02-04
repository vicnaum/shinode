# era

## Purpose
Integration tests for `.era` (consensus-layer) files: validate genesis handling and roundtrip read/decode/write behavior against downloaded fixtures.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - test module wiring.
  - **Key items**: `genesis`, `roundtrip`
- `genesis.rs` - tests around genesis-era `.era` file structure/edge cases.
  - **Key items**: (see tests in file)
- `roundtrip.rs` - roundtrip tests for `.era` files (read -> decode/decompress -> re-encode -> write -> re-read and compare).
  - **Key items**: `EraReader`, `EraWriter`, `EraFile`

## Relationships
- **Driven by**: `tests/it/main.rs` helper that downloads fixtures into a temp dir and opens them via `EraReader`.
