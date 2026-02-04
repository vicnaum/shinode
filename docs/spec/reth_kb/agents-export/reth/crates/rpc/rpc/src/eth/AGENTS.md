# eth

## Purpose
Implementation of the `eth` namespace: core API type, builder/config, filters, pubsub, bundles, and simulation helpers.

## Contents (one hop)
### Subdirectories
- [x] `helpers/` - Trait adapters, signer helpers, pending block support, and sync utilities.

### Files
- `mod.rs` - Module wiring and re-exports for `eth` handlers.
  - **Key items**: `EthApi`, `EthApiBuilder`, `EthFilter`, `EthPubSub`, `EthBundle`
- `builder.rs` - Configurable builder for constructing `EthApi` instances.
  - **Key items**: `EthApiBuilder`, `GasCap`, `FeeHistoryCacheConfig`, `EthStateCacheConfig`, `PendingBlockKind`
  - **Interactions**: Wires task spawners, caches, and limits into `EthApiInner`.
- `core.rs` - Core `EthApi` implementation and shared types.
  - **Key items**: `EthApi`, `EthApiInner`, `EthApiFor`, `EthApiBuilderFor`, `EthRpcConverterFor`
  - **Interactions**: Centralizes provider/pool/network access and EVM execution helpers.
- `filter.rs` - `eth_getLogs` and filter lifecycle management.
  - **Key items**: `EthFilter`, `ActiveFilters`, `EthFilterError`, `QueryLimits`
  - **Interactions**: Reads headers/receipts, uses pool streams, and evicts stale filters.
- `pubsub.rs` - `eth_subscribe` implementation for headers/logs/txs/syncing.
  - **Key items**: `EthPubSub`, `SubscriptionSerializeError`
  - **Interactions**: Consumes canonical state broadcasts and pool events.
- `bundle.rs` - MEV bundle simulation for `eth_callBundle`.
  - **Key items**: `EthBundle`, `EthBundleError`, `call_bundle()`
  - **Interactions**: Executes bundles via EVM with state at specified block.
- `sim_bundle.rs` - MEV bundle simulation for `mev_simBundle`.
  - **Key items**: `EthSimBundle`, `FlattenedBundleItem`, `EthSimBundleError`
  - **Interactions**: Flattens nested bundles and simulates with EVM.

## Key APIs (no snippets)
- `EthApi`, `EthApiBuilder`
- `EthFilter`, `EthPubSub`
- `EthBundle`, `EthSimBundle`
