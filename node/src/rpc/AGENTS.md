# rpc

## Purpose
JSON-RPC server built on `jsonrpsee`. Exposes a small Ethereum-like API backed by the local
`storage::Storage` (primarily for reading headers, tx hashes, receipts, and derived logs).
Integrates with `SyncProgressStats` to track RPC request metrics for TUI display.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - RPC server bootstrap, method registrations, param parsing helpers, response formatting, and stats tracking.
  - **Key items**: `RpcConfig`, `RpcContext`, `start()`, `module()`, `eth_getBlockByNumber`, `eth_getLogs`, `parse_block_tag()`

## Key APIs (no snippets)
- **Types**: `RpcConfig`, `RpcContext`, `BlockTag`, `RpcBlock`, `RpcLog`, `DerivedLog`
- **Functions**: `start()`, `module()`, `ensure_empty_params()`, `parse_block_tag()`, `parse_topics_filter()`, `log_matches()`, `format_log()`
- **Stats methods**: `RpcContext::inc_rpc_total()`, `inc_rpc_get_logs()`, `inc_rpc_get_block()`, `inc_rpc_errors()`

## Relationships
- **Used by**: `node/src/main.rs` starts the server with `RpcConfig::from(&NodeConfig)`.
- **Depends on**: `node/src/storage` for reads (`last_indexed_block()`, `block_header()`, `block_receipts_range()`, etc.);
  `node/src/sync::SyncProgressStats` for tracking RPC request and error metrics.

## Files (detailed)

### `mod.rs`
- **Role**: Defines the RPC surface, validates requests, and tracks RPC statistics through `SyncProgressStats`.
- **Key items**: `RpcConfig`, `RpcContext` (wraps config, `Arc<Storage>`, and `Option<Arc<SyncProgressStats>>`),
  `RpcContext::internal_error()` (creates error and increments error counter), `BlockTag`, `DerivedLog`, `RpcBlock`, `RpcLog`
- **Interactions**: `eth_getLogs` derives logs by combining `StoredReceipts` with stored tx hashes and sealing headers to compute block hashes.
  Every RPC method increments `rpc_total_requests`; `eth_getLogs` also increments `rpc_get_logs`; `eth_getBlockByNumber` also increments `rpc_get_block`; internal errors increment `rpc_errors`.
- **Knobs / invariants**:
  - Rejects unsupported inputs: `blockHash` filters, `pending` block tag, and `includeTransactions=true`.
  - Enforces `max_blocks_per_filter` (range width) and `max_logs_per_response` (output size) when non-zero.
  - Stats tracking is optional (`Option<Arc<SyncProgressStats>>`); increments are silently skipped when None.

## End-to-end flow (high level)
- `start()` builds a `jsonrpsee` server with request/response size limits and batch config.
- `module()` registers supported methods and wires them to `RpcContext { config, storage, stats }`.
- Each method validates params, increments `rpc_total_requests`, loads required ranges from storage, and formats hex-quantity outputs.
- Method-specific counters (`rpc_get_logs`, `rpc_get_block`) are incremented before processing.
- On internal errors, `RpcContext::internal_error()` automatically increments `rpc_errors`.
- Responses are returned as JSON values or JSON-RPC error objects with stable error codes.
