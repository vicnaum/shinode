# validation

## Purpose
Optimism-specific consensus validation helpers for hardforks and post-execution checks.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Shared validation helpers and receipt/root verification.
  - **Key items**: `validate_body_against_header_op()`, `validate_block_post_execution()`
- `canyon.rs` - Canyon hardfork checks for empty withdrawals.
  - **Key items**: `ensure_empty_withdrawals_root()`, `ensure_empty_shanghai_withdrawals()`
- `isthmus.rs` - Isthmus hardfork checks for L2 withdrawals storage root.
  - **Key items**: `ensure_withdrawals_storage_root_is_some()`, `verify_withdrawals_root()`

## Key APIs (no snippets)
- `validate_block_post_execution`
