# it

## Purpose
Integration tests for OP node builder, custom genesis, tx priority, and RPC behavior.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module harness wiring the integration test modules.
- **Key items**: modules `builder`, `priority`, `rpc`, `custom_genesis`
- **Interactions**: Aggregates the module tests under `tests/it`.

### `builder.rs`
- **Role**: Verifies OP node builder setup and custom precompile wiring.
- **Key items**: `test_basic_setup`, `test_setup_custom_precompiles`
- **Interactions**: Exercises node builder configuration and EVM factory customization.

### `custom_genesis.rs`
- **Role**: Validates custom genesis block number support.
- **Key items**: `test_op_node_custom_genesis_number`
- **Interactions**: Asserts stage checkpoints and provider queries after init.

### `priority.rs`
- **Role**: Tests custom transaction priority integration with payload builder.
- **Key items**: `CustomTxPriority`, `test_custom_block_priority_config`
- **Interactions**: Influences payload assembly ordering.

### `rpc.rs`
- **Role**: Checks admin RPC external IP reporting.
- **Key items**: `test_admin_external_ip`
- **Interactions**: Verifies network config propagation to RPC.

## End-to-end flow (high level)
- Build node configurations via the builder tests.
- Initialize custom genesis and validate checkpoints.
- Exercise custom transaction priority in payload builder tests.
- Validate RPC admin info reflects network configuration.

## Key APIs (no snippets)
- `OpNode` builder tests
