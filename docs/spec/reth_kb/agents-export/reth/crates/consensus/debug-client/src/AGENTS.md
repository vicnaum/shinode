# src

## Purpose
Implements `reth-consensus-debug-client`: a small "debug consensus" worker that drives an execution client by sending `newPayload` and `forkchoiceUpdated` messages using blocks fetched from an external source (RPC or Etherscan), without running a real consensus node.

## Contents (one hop)
### Subdirectories
- [x] `providers/` - Block provider backends (RPC subscription/polling, Etherscan HTTP polling).

### Files
- `lib.rs` - crate module wiring and public re-exports.
  - **Key items**: `DebugConsensusClient`, `BlockProvider`, `EtherscanBlockProvider`, `RpcBlockProvider`
- `client.rs` - core debug-consensus logic: block provider trait and the worker that converts blocks into execution payloads and emits engine API messages.
  - **Key items**: `BlockProvider`, `DebugConsensusClient<P, T>`, `DebugConsensusClient::run()`, `get_or_fetch_previous_block()`

## Key APIs (no snippets)
- **Block source abstraction**: `BlockProvider`
- **Worker**: `DebugConsensusClient`

## Relationships
- **Used by**: tooling/tests that want to validate/experiment with the engine API/execution client behavior without a beacon/consensus client.
- **Depends on**: `reth-node-api` engine API handle/types and `alloy-rpc-types-engine` forkchoice types; provider backends depend on `alloy-provider`/RPC and/or Etherscan HTTP APIs.
