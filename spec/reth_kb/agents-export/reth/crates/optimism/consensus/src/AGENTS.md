# src

## Purpose
Optimism consensus implementation with OP hardfork-specific validation and receipt root handling.

## Contents (one hop)
### Subdirectories
- [x] `validation/` - Hardfork-specific validation helpers and post-exec checks.

### Files
- `lib.rs` - Main Optimism consensus type and validation wiring.
  - **Key items**: `OpBeaconConsensus`, `calculate_receipt_root_no_memo_optimism`
- `error.rs` - Optimism consensus errors.
  - **Key items**: `OpConsensusError`
- `proof.rs` - Receipt root calculation helpers with Regolith/Canyon handling.
  - **Key items**: `calculate_receipt_root_optimism()`, `calculate_receipt_root_no_memo_optimism()`

## Key APIs (no snippets)
- `OpBeaconConsensus`, `OpConsensusError`
