# common

## Purpose
`reth-consensus-common` crate: shared, chain-spec-aware validation helpers for Ethereum blocks/headers/bodies (pre-execution / stateless checks) used by consensus implementations.

## Contents (one hop)
### Subdirectories
- [x] `src/` - validation helper functions and constants.

### Files
- `Cargo.toml` - crate manifest (std/no-std support and dependency wiring).
  - **Key items**: `features.std`

## Key APIs (no snippets)
- **Validation helpers**: `validate_block_pre_execution()`, `validate_body_against_header()`, `validate_header_gas()`, `post_merge_hardfork_fields()`

## Relationships
- **Used by**: consensus engines and validation call sites that want common Ethereum consensus checks.
