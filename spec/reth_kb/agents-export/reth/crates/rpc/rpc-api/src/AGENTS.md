# src

## Purpose
RPC interface definitions for all namespaces (traits and JSON-RPC method signatures).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports of all server/client traits.
- **Key items**: `servers`, `clients`, `TestingBuildBlockRequestV1`, `TESTING_BUILD_BLOCK_V1`
- **Interactions**: Re-exports `reth_rpc_eth_api` server/client traits.

### `admin.rs`
- **Role**: Admin namespace trait definitions.
- **Key items**: `AdminApi`, `add_peer()`, `remove_peer()`, `peers()`, `node_info()`

### `anvil.rs`
- **Role**: Anvil-compatible namespace trait definitions.
- **Key items**: `AnvilApi`, `anvil_impersonate_account()`, `anvil_mine()`, `anvil_reset()`, `anvil_dump_state()`

### `debug.rs`
- **Role**: Debug namespace trait definitions for raw block/tx access and tracing.
- **Key items**: `DebugApi`, `debug_trace_block()`, `debug_trace_transaction()`, `debug_execution_witness()`

### `engine.rs`
- **Role**: Engine API trait definitions and module adapter for consensus clients.
- **Key items**: `EngineApi`, `EngineEthApi`, `IntoEngineApiRpcModule`, `new_payload_v3()`, `fork_choice_updated_v3()`

### `hardhat.rs`
- **Role**: Hardhat-compatible namespace trait definitions.
- **Key items**: `HardhatApi`, `hardhat_impersonate_account()`, `hardhat_set_balance()`, `hardhat_reset()`

### `mev.rs`
- **Role**: MEV namespace trait definitions for bundle submission/simulation.
- **Key items**: `MevSimApi`, `MevFullApi`, `sim_bundle()`, `send_bundle()`

### `miner.rs`
- **Role**: Miner namespace trait definitions.
- **Key items**: `MinerApi`, `set_extra()`, `set_gas_price()`, `set_gas_limit()`

### `net.rs`
- **Role**: Net namespace trait definitions.
- **Key items**: `NetApi`, `version()`, `peer_count()`, `is_listening()`

### `otterscan.rs`
- **Role**: Otterscan-compatible namespace trait definitions.
- **Key items**: `Otterscan`, `get_header_by_number()`, `get_block_details()`, `trace_transaction()`

### `reth.rs`
- **Role**: Reth-specific namespace trait definitions.
- **Key items**: `RethApi`, `reth_get_balance_changes_in_block()`, `reth_subscribe_chain_notifications()`

### `rpc.rs`
- **Role**: RPC namespace trait definitions.
- **Key items**: `RpcApi`, `rpc_modules()`

### `testing.rs`
- **Role**: Testing namespace trait definitions for block-building RPC.
- **Key items**: `TestingApi`, `TestingBuildBlockRequestV1`, `TESTING_BUILD_BLOCK_V1`, `build_block_v1()`

### `trace.rs`
- **Role**: Trace namespace trait definitions for parity-style tracing.
- **Key items**: `TraceApi`, `trace_call()`, `trace_raw_transaction()`, `trace_filter()`

### `txpool.rs`
- **Role**: Txpool namespace trait definitions.
- **Key items**: `TxPoolApi`, `txpool_status()`, `txpool_inspect()`, `txpool_content()`

### `validation.rs`
- **Role**: Block submission validation namespace trait definitions.
- **Key items**: `BlockSubmissionValidationApi`, `validate_builder_submission_v1()`, `validate_builder_submission_v5()`

### `web3.rs`
- **Role**: Web3 namespace trait definitions.
- **Key items**: `Web3Api`, `client_version()`, `sha3()`

## End-to-end flow (high level)
- Namespace trait definitions are declared with `jsonrpsee` macros.
- The `servers` module re-exports all server-side traits for implementation.
- The optional `clients` module re-exports client-side traits when enabled.
- Downstream crates implement these traits to provide concrete RPC handlers.

## Key APIs (no snippets)
- `AdminApi`, `EngineApi`, `EthApiServer`, `TraceApi`
- `MevSimApi`, `TestingApi`, `BlockSubmissionValidationApi`
