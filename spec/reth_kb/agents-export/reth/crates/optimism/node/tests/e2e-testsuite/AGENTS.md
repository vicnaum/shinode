# e2e-testsuite

## Purpose
End-to-end test suite for OP node behavior, sync, and payload handling.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness wiring the e2e test suite modules.
- **Key items**: modules `p2p`, `testsuite`
- **Interactions**: Includes the sync and testsuite modules under the e2e harness.

### `p2p.rs`
- **Role**: Multi-node e2e sync test that exercises optimistic sync, reorgs, and live sync.
- **Key items**: `can_sync`
- **Interactions**: Uses the e2e test utils to configure peers and advance the chain.

### `testsuite.rs`
- **Role**: Testsuite integration that verifies mined block attributes with OP-specific checks.
- **Key items**: `test_testsuite_op_assert_mine_block*`
- **Interactions**: Uses `reth_e2e_test_utils` assertions on OP payload attributes.

## End-to-end flow (high level)
- Wire the e2e modules via `main.rs`.
- Spin up OP nodes and advance a canonical chain in `p2p` tests.
- Run testsuite assertions against mined payload attributes in `testsuite`.

## Key APIs (no snippets)
- E2E tests using `reth_e2e_test_utils`
