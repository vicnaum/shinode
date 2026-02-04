# src

## Purpose
Shared `eth_` namespace API traits, type aliases, and server/client interfaces.

## Contents (one hop)
### Subdirectories
- [x] `helpers/` - Helper traits for loading data, calls, fees, state, and tracing.

### Files
- `lib.rs` - Module wiring and re-exports for `eth_` APIs.
  - **Key items**: `EthApiServer`, `EthFilterApiServer`, `EthPubSubApiServer`, `RpcNodeCore`
- `bundle.rs` - Bundle RPC trait definitions.
  - **Key items**: `EthCallBundleApi`, `EthBundleApi`
- `core.rs` - Core `eth_` API server trait definitions.
  - **Key items**: `EthApiServer`, `FullEthApiServer`
- `ext.rs` - L2 `eth_` extension traits.
  - **Key items**: `L2EthApiExt`
- `filter.rs` - Filter RPC trait definitions and query limits.
  - **Key items**: `EthFilterApi`, `EngineEthFilter`, `QueryLimits`
- `node.rs` - RPC node core abstraction traits.
  - **Key items**: `RpcNodeCore`, `RpcNodeCoreExt`
- `pubsub.rs` - PubSub RPC trait definitions.
  - **Key items**: `EthPubSubApi`
- `types.rs` - Core RPC type aliases and trait bounds.
  - **Key items**: `EthApiTypes`, `FullEthApiTypes`, `RpcBlock`, `RpcTransaction`

## Key APIs (no snippets)
- `EthApiServer`, `EthFilterApiServer`, `EthPubSubApi`
- `RpcNodeCore`, `EthApiTypes`
