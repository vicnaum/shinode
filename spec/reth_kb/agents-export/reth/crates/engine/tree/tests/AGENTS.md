# tests

## Purpose
Integration test area for the `reth-engine-tree` crate.

## Contents (one hop)
### Subdirectories
- [x] `e2e-testsuite/` - End-to-end Engine API test suite (forkchoice/newPayload scenarios, finalized-block edge cases).

### Files
- (none)

## Key APIs (no snippets)
- **Test targets**: `e2e-testsuite/main.rs` (suite entrypoint)

## Relationships
- **Related crate**: `reth/crates/engine/tree/Cargo.toml` registers `tests/e2e-testsuite/main.rs` as an integration test target.
