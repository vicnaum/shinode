# rpc

## Purpose
JSON-RPC server built on `jsonrpsee`. Exposes a small Ethereum-like API backed by the local
`storage::Storage` (primarily for reading headers, tx hashes, receipts, and derived logs).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - RPC server bootstrap, method registrations, param parsing helpers, and response formatting.
  - **Key items**: `RpcConfig`, `start()`, `module()`, `eth_getBlockByNumber`, `eth_getLogs`, `parse_block_tag()`

## Key APIs (no snippets)
- **Types**: `RpcConfig`, `RpcContext`, `BlockTag`, `RpcBlock`, `RpcLog`
- **Functions**: `start()`, `module()`, `ensure_empty_params()`, `parse_block_tag()`, `parse_topics_filter()`, `log_matches()`

## Relationships
- **Used by**: `node/src/main.rs` starts the server with `RpcConfig::from(&NodeConfig)`.
- **Depends on**: `node/src/storage` for reads (`last_indexed_block()`, `block_header()`, `block_receipts_range()`, etc.).

## Files (detailed)

### `mod.rs`
- **Role**: Defines the RPC surface and ensures requests are validated and bounded before reading from storage.
- **Key items**: `RpcConfig`, `ServerConfig`, `BlockTag`, `missing_block_error()`, `format_log()`, `RpcWithdrawal`
- **Interactions**: `eth_getLogs` derives logs by combining `StoredReceipts` with stored tx hashes and sealing headers to compute block hashes.
- **Knobs / invariants**:
  - Rejects unsupported inputs: `blockHash` filters, `pending` block tag, and `includeTransactions=true`.
  - Enforces `max_blocks_per_filter` (range width) and `max_logs_per_response` (output size) when non-zero.

## End-to-end flow (high level)
- `start()` builds a `jsonrpsee` server with request/response size limits and batch config.
- `module()` registers supported methods and wires them to `RpcContext { config, storage }`.
- Each method validates params, loads required ranges from storage, and formats hex-quantity outputs.
- Responses are returned as JSON values or JSON-RPC error objects with stable error codes.

