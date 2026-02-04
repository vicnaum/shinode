# components

## Purpose
Component builder traits and defaults for assembling node subsystems (pool, network, payload service, consensus, EVM).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring and shared `NodeComponents` trait and `Components` container.
  - **Key items**: `NodeComponents`, `Components`
- `builder.rs` - Generic `ComponentsBuilder` with default/noop component builders.
  - **Key items**: `ComponentsBuilder`, `NodeComponentsBuilder`, `NoopTransactionPoolBuilder`, `NoopNetworkBuilder`, `NoopConsensusBuilder`, `NoopPayloadBuilder`
- `consensus.rs` - Consensus builder trait.
  - **Key items**: `ConsensusBuilder`
- `execute.rs` - EVM/executor builder trait.
  - **Key items**: `ExecutorBuilder`
- `network.rs` - Network builder trait.
  - **Key items**: `NetworkBuilder`
- `payload.rs` - Payload service builder traits and basic/noop implementations.
  - **Key items**: `PayloadServiceBuilder`, `PayloadBuilderBuilder`, `BasicPayloadServiceBuilder`, `NoopPayloadServiceBuilder`
- `pool.rs` - Transaction pool builder trait and helper builders.
  - **Key items**: `PoolBuilder`, `PoolBuilderConfigOverrides`, `TxPoolBuilder`

## Key APIs (no snippets)
- `NodeComponents`, `ComponentsBuilder`
- `PoolBuilder`, `NetworkBuilder`, `PayloadServiceBuilder`
