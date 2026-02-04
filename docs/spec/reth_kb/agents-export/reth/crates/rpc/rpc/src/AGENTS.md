# src

## Purpose
Concrete RPC handlers for reth namespaces (`eth`, `admin`, `debug`, `trace`, `txpool`, etc.).

## Contents (one hop)
### Subdirectories
- [x] `eth/` - `eth_` namespace core, filters, pubsub, and bundle simulation.

### Files
- `lib.rs` - Crate entrypoint and RPC handler re-exports.
  - **Key items**: `EthApi`, `EthApiBuilder`, `EthFilter`, `EthPubSub`, `AdminApi`, `DebugApi`, `TraceApi`
- `admin.rs` - `admin_` namespace handlers (peers, node info, txpool maintenance).
  - **Key items**: `AdminApi`, `add_peer()`, `peers()`, `node_info()`, `clear_txpool()`
- `aliases.rs` - Type aliases for dynamic RPC converter usage.
  - **Key items**: `DynRpcConverter`
- `debug.rs` - `debug_` namespace handlers and tracing helpers.
  - **Key items**: `DebugApi`, `debug_trace_block()`, `debug_trace_raw_block()`, `BadBlockStore`
- `engine.rs` - `engine_`-compatible `eth_` subset wrapper.
  - **Key items**: `EngineEthApi`, `syncing()`, `block_by_number()`, `logs()`
- `miner.rs` - `miner_` namespace stub implementation.
  - **Key items**: `MinerApi`, `set_extra()`, `set_gas_price()`, `set_gas_limit()`
- `net.rs` - `net_` namespace implementation.
  - **Key items**: `NetApi`, `version()`, `peer_count()`, `is_listening()`
- `otterscan.rs` - Otterscan and Erigon compatibility endpoints.
  - **Key items**: `OtterscanApi`, `get_block_details()`, `get_internal_operations()`, `trace_transaction()`
- `reth.rs` - `reth_` namespace for prototype endpoints and subscriptions.
  - **Key items**: `RethApi`, `balance_changes_in_block()`, `reth_subscribe_chain_notifications()`
- `rpc.rs` - `rpc_` namespace implementation (`rpc_modules`).
  - **Key items**: `RPCApi`, `rpc_modules()`
- `testing.rs` - `testing_` namespace block builder (non-production).
  - **Key items**: `TestingApi`, `build_block_v1()`, `with_skip_invalid_transactions()`
- `trace.rs` - Parity-style `trace_` namespace implementation.
  - **Key items**: `TraceApi`, `trace_call()`, `trace_raw_transaction()`, `trace_call_many()`
- `txpool.rs` - `txpool_` namespace implementation.
  - **Key items**: `TxPoolApi`, `txpool_status()`, `txpool_inspect()`, `txpool_content()`
- `validation.rs` - Builder validation namespace for payload/bid verification.
  - **Key items**: `ValidationApi`, `ValidationApiConfig`, `ValidationApiError`, `validate_message_against_block()`
- `web3.rs` - `web3_` namespace handlers.
  - **Key items**: `Web3Api`, `client_version()`, `sha3()`

## Key APIs (no snippets)
- `EthApi`, `EthApiBuilder`, `EthFilter`, `EthPubSub`
- `AdminApi`, `DebugApi`, `TraceApi`, `TxPoolApi`
