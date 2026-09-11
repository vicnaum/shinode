# tests

## Purpose

Integration tests for the Stateless History Node, including Base protocol and receipt-codec compatibility checks, plus placeholder stubs for planned RPC contract and reorg/rollback tests.

## Contents (one hop)

### Subdirectories
- `fixtures/` - Pinned Base mainnet protocol values and execution bootnodes.

### Files
- `chain_cli.rs` - Binary-level help checks, Base sync gating, Base/legacy Ethereum maintenance, directory overrides, and mismatch rejection before cache/log creation.
- `base_protocol.rs` - Offline Base Beryl fork-ID, transaction-type, and OP receipt bincode tests; ignored, bounded live P2P block/receipt verification. Run the live test explicitly and set `SHINODE_BASE_PROBE_OUTPUT` to retain JSON evidence. Does not enable production Base ingestion.
- `test_strategy.rs` - Integration test stubs for v0.1 strategy (~20 lines). Two ignored/TODO tests.
  - **Key items**: `rpc_contract_probe` (ignored - planned RPC contract test), `reorg_rollback_integration` (ignored - planned reorg rollback test)

## Key APIs (no snippets)

- **Functions**: `rpc_contract_probe()`, `reorg_rollback_integration()` - TODO tests (currently `#[ignore]`).

## Relationships

- **Depends on**: Will depend on `node/src/rpc`, `node/src/sync/historical`, and `node/src/storage` once tests are implemented.
- **Used by**: `cargo test`; network-dependent and placeholder tests require explicit opt-in.

## Files (detailed)

### `test_strategy.rs`
- **Role**: Placeholder integration tests that define expected RPC and reorg coverage without executing in CI yet.
- **Key items**: `rpc_contract_probe`, `reorg_rollback_integration`, `#[ignore]`
- **Interactions**: Intended to exercise the RPC surface and follow-mode rollback logic once enabled.
- **Knobs / invariants**: Tests are `#[ignore]` until the API surface stabilizes.

## End-to-end flow (high level)

- `cargo test` runs offline Base compatibility tests and skips the live probe and placeholders.
- Explicitly run `cargo test --test base_protocol live_base_protocol_probe -- --ignored` to check Base discovery, handshake, and a recent block's transaction/receipt roots.
- When implemented, the RPC contract and reorg placeholders should validate their respective behaviors.

## Notes

- The Base live test is ignored because it requires network access; the two strategy placeholders remain ignored until implemented.
