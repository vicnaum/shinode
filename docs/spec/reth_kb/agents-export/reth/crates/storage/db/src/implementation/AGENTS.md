# implementation

## Purpose
Backend-specific database implementations (currently MDBX).

## Contents (one hop)
### Subdirectories
- [x] `mdbx/` - MDBX-backed cursor and transaction implementations.

### Files
- `mod.rs` - Feature-gated module selection for DB backends.
  - **Key items**: `mdbx` (feature gated)

## Key APIs (no snippets)
- **Modules / Packages**: `mdbx`

## Relationships
- **Depends on**: `reth-libmdbx` when the `mdbx` feature is enabled.
