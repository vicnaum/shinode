# providers

## Purpose
Implements concrete `BlockProvider` backends for the debug consensus client: fetching blocks either via an RPC endpoint (subscription/polling) or via the Etherscan HTTP API.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Provider module wiring and re-exports.
- **Key items**: `EtherscanBlockProvider`, `RpcBlockProvider`
- **Interactions**: re-exported by the crate root for easy consumption.
- **Knobs / invariants**: none.

#### `rpc.rs`
- **Role**: RPC-backed block provider that subscribes to new full blocks (websocket `eth_subscribe` when possible, otherwise polling) and supports historical `get_block` queries.
- **Key items**: `RpcBlockProvider<N, PrimitiveBlock>`, `RpcBlockProvider::new()`, `subscribe_blocks()`, `get_block()`, `full_block_stream()`
- **Interactions**: uses `alloy-provider`/`ProviderBuilder` with websocket config; converts `N::BlockResponse` into a primitives block via a caller-supplied converter.
- **Knobs / invariants**: configures large WS frame/message sizes for "big blocks"; re-establishes subscriptions when streams terminate.

#### `etherscan.rs`
- **Role**: Etherscan-backed block provider that polls the Etherscan "proxy" API for latest blocks and supports fetching blocks by number/tag.
- **Key items**: `EtherscanBlockProvider<RpcBlock, PrimitiveBlock>`, `load_block()`, `subscribe_blocks()`, `with_interval()`
- **Interactions**: issues HTTP GET requests with module/action parameters, parses `alloy_json_rpc::Response`, and converts the decoded RPC block into a primitives block via a caller-supplied converter.
- **Knobs / invariants**: polling interval (default ~3s); handles `chainid` parameter inclusion depending on the base URL format.

## Key APIs (no snippets)
- **RPC provider**: `RpcBlockProvider`
- **Etherscan provider**: `EtherscanBlockProvider`

## Relationships
- **Used by**: `DebugConsensusClient` as a source of new blocks and historical block lookups.
- **Depends on**: `alloy-provider` (RPC transport/subscriptions) and `reqwest` + `alloy-json-rpc` (Etherscan HTTP polling/parsing).
