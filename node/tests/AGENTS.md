# tests

## Purpose
Integration test entrypoints for validating end-to-end node behavior (RPC contract and reorg/rollback
handling). Most tests are currently placeholders and ignored.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `test_strategy.rs` - Ignored integration tests documenting planned coverage.
  - **Key items**: `rpc_contract_probe`, `reorg_rollback_integration`

## Key APIs (no snippets)
- **Functions**: `rpc_contract_probe()`, `reorg_rollback_integration()` - TODO tests (currently `#[ignore]`).

## Relationships
- **Depends on**: `node/src/rpc` and `node/src/sync/historical` once the contract expands.
- **Used by**: `cargo test` (after un-ignoring tests).

## Files (detailed)

### `test_strategy.rs`
- **Role**: Placeholder integration tests that define expected RPC and reorg coverage without executing in CI yet.
- **Key items**: `rpc_contract_probe`, `reorg_rollback_integration`, `#[ignore]`
- **Interactions**: Intended to exercise the RPC surface and follow-mode rollback logic once enabled.
- **Knobs / invariants**: Tests are `#[ignore]` until the API surface stabilizes.

## End-to-end flow (high level)
- `cargo test` discovers the tests but skips them due to `#[ignore]`.
- When un-ignored, the RPC probe should validate method contracts and the reorg test should validate rollback behavior.

## Notes
- Tests are intentionally ignored until the relevant surfaces are stable.
