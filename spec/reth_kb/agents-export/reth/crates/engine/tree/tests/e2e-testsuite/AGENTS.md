# e2e-testsuite

## Purpose
End-to-end integration tests for `reth-engine-tree`, exercising Engine API handling and chain progression behaviors (e.g., forkchoice/finalization edge cases).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `fcu_finalized_blocks.rs` - e2e coverage for forkchoiceUpdated interactions involving finalized blocks.
- `main.rs` - test harness entrypoint wiring the e2e test suite.

## Key APIs (no snippets)
- **Files-as-entrypoints**: `main.rs` (suite runner), `fcu_finalized_blocks.rs` (scenario coverage)

## Relationships
- **Used by**: `reth/crates/engine/tree/Cargo.toml` declares this as an integration test target (`[[test]] name = "e2e_testsuite"`).

## Notes
- These tests are intended to validate cross-component behavior (tree + engine handler + persistence/backfill coordination), not individual unit invariants.
