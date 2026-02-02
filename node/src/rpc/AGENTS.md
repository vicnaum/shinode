# rpc

## Purpose
JSON-RPC server built on `jsonrpsee`. Exposes a small Ethereum-like API backed by the local
`storage::Storage` (primarily for reading headers, tx hashes, receipts, and derived logs).
Integrates with `SyncProgressStats` to track RPC request metrics for TUI display.
Designed for read-only access with configurable rate limiting; no state execution.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - RPC server bootstrap, method registrations, param parsing helpers, response formatting, and stats tracking (~1125 lines, ~476 lines tests).
  - **Key items**: `RpcConfig`, `RpcContext`, `start()`, `module()`, `eth_getBlockByNumber`, `eth_getLogs`, `parse_block_tag()`

## Key APIs (no snippets)
- **Types**: `RpcConfig`, `RpcContext`, `BlockTag`, `RpcBlock`, `RpcLog`, `DerivedLog`
- **Functions**: `start(bind, config, storage, stats)`, `module(ctx)`, `ensure_empty_params()`, `parse_block_tag()`, `resolve_block_tag_optional/required()`, `parse_address_filter()`, `parse_topics_filter()`, `log_matches()`, `format_log()`
- **RPC Methods**: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber(tag, false)`, `eth_getLogs(filter)`
- **Error helpers**: `invalid_params` (-32602), `internal_error` (-32000), `missing_block_error` (-32001)
- **Stats methods**: `SyncProgressStats::inc_rpc_total()`, `inc_rpc_get_logs()`, `inc_rpc_get_block()`, `inc_rpc_errors()` (invoked by the RPC layer when `stats: Option<Arc<SyncProgressStats>>` is present)

## Relationships
- **Used by**: `run/sync_runner.rs` spawns RPC server once synced; `node/src/main.rs` starts the server with `RpcConfig::from(&NodeConfig)`.
- **Depends on**: `storage::Storage` for reads (`last_indexed_block()`, `block_header()`, `block_receipts_range()`, etc.);
  `sync::SyncProgressStats` for tracking RPC request and error metrics;
  `cli::NodeConfig` for RPC config defaults;
  `jsonrpsee` (server framework), `alloy_primitives`, `reth_primitives_traits`.

## Files (detailed)

### `mod.rs`
- **Role**: Complete JSON-RPC server implementation (~1125 lines, ~476 lines tests). Defines the RPC surface, validates requests, and tracks RPC statistics through `SyncProgressStats`.
- **Key items**:
  - **Config/Context**: `RpcConfig`, `RpcContext` (wraps config, `Arc<Storage>`, and `Option<Arc<SyncProgressStats>>`)
  - **Server**: `start()`, `module()`
  - **Methods**: `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`
  - **Parsing**: `BlockTag` enum, `parse_block_tag`, `resolve_block_tag_optional/required`, `parse_address_filter`, `parse_topics_filter`, `ensure_empty_params`
  - **Log derivation**: `DerivedLog` struct, `log_matches`, `format_log`
  - **Response types**: `RpcBlock`, `RpcLog`
  - **Error helpers**: `invalid_params` (-32602), `internal_error` (-32000), `missing_block_error` (-32001)
  - **Context method**: `RpcContext::internal_error()` creates error and increments `rpc_errors` counter
- **Interactions**: `eth_getLogs` derives logs by combining `StoredReceipts` with stored tx hashes and sealing headers to compute block hashes.
  Every RPC method increments `rpc_total_requests`; `eth_getLogs` also increments `rpc_get_logs`; `eth_getBlockByNumber` also increments `rpc_get_block`; internal errors increment `rpc_errors`.
- **Knobs / invariants**:
  - `max_blocks_per_filter` (10K): caps `eth_getLogs` block range
  - `max_logs_per_response` (100K): caps log count per response
  - `max_connections` (100), `max_batch_requests` (100)
  - `max_request_body_bytes` (10MB), `max_response_body_bytes` (100MB)
  - Rejects unsupported inputs: `blockHash` filters, `pending` block tag, and `includeTransactions=true`.
  - Enforces `max_blocks_per_filter` (range width) and `max_logs_per_response` (output size) when non-zero.
  - Stats tracking is optional (`Option<Arc<SyncProgressStats>>`); increments are silently skipped when None.
  - Logs derived on-demand from receipts (no pre-indexing).

## Data/control flow
1. `start()` binds jsonrpsee server with configured limits
2. `module()` registers supported methods and wires them to `RpcContext { config, storage, stats }`
3. `eth_getBlockByNumber`: resolve tag -> read header + tx_hashes + size -> seal header for hash -> format `RpcBlock`
4. `eth_getLogs`: parse filter -> validate range -> check block presence -> batch read headers/tx_hashes/receipts -> derive logs -> filter by address/topics -> format `RpcLog` array
5. Stats: increment `rpc_total`, `rpc_get_block`/`rpc_get_logs`, `rpc_errors`

## End-to-end flow (high level)
1. `run/sync_runner.rs` calls `rpc::start()` after chain synced, returns `ServerHandle`.
2. Client sends JSON-RPC request to bound address (default 127.0.0.1:8545).
3. Method dispatched: `eth_chainId` (trivial), `eth_blockNumber` (last indexed), `eth_getBlockByNumber` (header+txs), `eth_getLogs` (receipts+filter).
4. Each method validates params, increments `rpc_total_requests`, loads required ranges from storage, and formats hex-quantity outputs.
5. Method-specific counters (`rpc_get_logs`, `rpc_get_block`) are incremented before processing.
6. On internal errors, `RpcContext::internal_error()` automatically increments `rpc_errors`.
7. `eth_getLogs` validates block range, checks presence, batch-reads from storage, derives logs on-demand from receipts, applies address/topic filters.
8. Responses serialized as JSON with hex-encoded fields; errors returned as JSON-RPC error objects with stable error codes.
9. Graceful shutdown via `handle.stop()`.

## Notes
