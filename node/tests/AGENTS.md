# tests

## Purpose

Integration test scaffolding for the Stateless History Node. Currently contains placeholder stubs for planned integration tests (RPC contract and reorg/rollback handling).

## Contents (one hop)

### Subdirectories
- (none)

### Files
- `test_strategy.rs` - Integration test stubs for v0.1 strategy (~20 lines). Two ignored/TODO tests.
  - **Key items**: `rpc_contract_probe` (ignored - planned RPC contract test), `reorg_rollback_integration` (ignored - planned reorg rollback test)

## Key APIs (no snippets)

- **Functions**: `rpc_contract_probe()`, `reorg_rollback_integration()` - TODO tests (currently `#[ignore]`).

## Relationships

- **Depends on**: Will depend on `node/src/rpc`, `node/src/sync/historical`, and `node/src/storage` once tests are implemented.
- **Used by**: `cargo test` (skipped due to `#[ignore]`).

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
