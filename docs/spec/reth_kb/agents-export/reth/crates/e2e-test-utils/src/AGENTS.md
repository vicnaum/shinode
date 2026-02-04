# src

## Purpose
Implements `reth-e2e-test-utils`: utilities for spinning up one or more in-process test nodes, wiring RPC/Engine API clients, producing payloads/blocks, and running higher-level e2e "testsuite" action sequences.

## Contents (one hop)
### Subdirectories
- [x] `testsuite/` - e2e framework (environment model, setup logic, and action catalog).

### Files
- `lib.rs` - crate entrypoint: re-exports major helpers and defines setup functions/type aliases used across tests.
  - **Key items**: `setup()`, `setup_engine()`, `setup_engine_with_connection()`, `E2ETestSetupBuilder`, `NodeBuilderHelper`, `NodeHelperType`
- `node.rs` - per-node helper wrapper for driving a running `FullNode`: payload building/submission, forkchoice updates, canonical-state assertions, and network/RPC contexts.
  - **Key items**: `NodeTestContext`, `advance()`, `advance_block()`, `build_and_submit_payload()`, `update_forkchoice()`
- `payload.rs` - payload builder helper that subscribes to payload events, generates attributes, and waits for built payloads.
  - **Key items**: `PayloadTestContext<T>`, `new_payload()`, `expect_attr_event()`, `wait_for_built_payload()`
- `network.rs` - network helper for peer operations and session establishment assertions.
  - **Key items**: `NetworkTestContext<Network>`, `add_peer()`, `next_session_established()`, `record()`
- `rpc.rs` - RPC helper for injecting txs and reading raw transactions via debug API.
  - **Key items**: `RpcTestContext`, `inject_tx()`, `envelope_by_hash()`
- `setup_builder.rs` - builder for launching configurable test node networks (node config/tree config modifiers; optional interconnection).
  - **Key items**: `E2ETestSetupBuilder<N, F>`, `with_tree_config_modifier()`, `with_node_config_modifier()`, `build()`
- `setup_import.rs` - helpers to import an RLP chain into per-node temporary datadirs before launching nodes (Ethereum-only today).
  - **Key items**: `ChainImportResult`, `setup_engine_with_chain_import()`, `load_forkchoice_state()`
- `test_rlp_utils.rs` - utilities to generate simple test blocks and write them to an RLP file; helpers for generating FCU JSON payloads.
  - **Key items**: `generate_test_blocks()`, `write_blocks_to_rlp()`, `create_fcu_json()`
- `transaction.rs` - transaction builder/signing helpers for tests (transfers, deployments, blob txs, EIP-7702 set-code, etc.).
  - **Key items**: `TransactionTestContext`, `transfer_tx_bytes()`, `deploy_tx_bytes()`, `tx_with_blobs_bytes()`
- `wallet.rs` - deterministic wallet generator from a test mnemonic (derivation-path based).
  - **Key items**: `Wallet`, `wallet_gen()`, `with_chain_id()`

## Key APIs (no snippets)
- **Setup**: `E2ETestSetupBuilder`, `testsuite::Setup`, `setup_engine_with_connection()`
- **Node wrapper**: `NodeTestContext`
- **Testsuite**: `testsuite::Environment`, `testsuite::actions::Action`

## Relationships
- **Used by**: workspace crates that define `tests/e2e-testsuite/*` binaries and want consistent node setup + action-based test flows.
- **Depends on**: node builder/test utils, payload builder test utils, RPC builder/client types, and engine/local components to run in-process nodes.
