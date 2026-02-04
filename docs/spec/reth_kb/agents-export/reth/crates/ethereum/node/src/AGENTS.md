# src

## Purpose
Implements `reth-node-ethereum`: Ethereum-flavoured node wiring for reth (node type configuration, component builders for pool/network/executor/consensus, payload builder wiring, RPC add-ons, and Engine API payload validation).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: exposes Ethereum node building blocks and re-exports common types (`EthEngineTypes`, `EthEvmConfig`, consensus module, node config, payload builder wiring, engine validator).
- **Key items**: `EthEngineTypes`, `EthEvmConfig`, module `node`, module `payload`, `EthereumEngineValidator`
- **Interactions**: imported by `reth-ethereum-cli`/`reth-node-builder` to instantiate a standard Ethereum node.

#### `node.rs`
- **Role**: Core Ethereum node integration with `reth-node-builder`: defines `EthereumNode` (implements `NodeTypes`), provides `components()` builder preset, and implements builders for executor, txpool, networking, consensus, and engine payload validation; wires RPC add-ons (`EthereumAddOns`) including module selection and middleware hooks.
- **Key items**: `EthereumNode`, `EthereumNode::components()`, `EthereumNode::provider_factory_builder()`, `EthereumAddOns`, `EthereumEthApiBuilder`, `EthereumExecutorBuilder`, `EthereumPoolBuilder`, `EthereumNetworkBuilder`, `EthereumConsensusBuilder`, `EthereumEngineValidatorBuilder`
- **Interactions**: uses `EthBeaconConsensus` + `EthEvmConfig`; builds `EthTransactionPool` with blob store/cache sizing and KZG init; launches networking via `NetworkHandle`; configures RPC modules via `reth_rpc_builder` and per-module wiring.

#### `payload.rs`
- **Role**: Payload component wiring for the Ethereum node: adapts `reth-ethereum-payload-builder` into `reth-node-builder`'s `PayloadBuilderBuilder` interface and maps node config -> `EthereumBuilderConfig`.
- **Key items**: `EthereumPayloadBuilder` (builder), `PayloadBuilderBuilder` impl
- **Interactions**: feeds gas limit / max blobs / extra data from `PayloadBuilderConfig` into the payload builder crate.

#### `engine.rs`
- **Role**: Engine API validation for Ethereum: wraps `EthereumExecutionPayloadValidator` and implements `PayloadValidator` + `EngineApiValidator` to enforce version-specific Engine API object rules (including Cancun/Prague execution requests).
- **Key items**: `EthereumEngineValidator`, `PayloadValidator` impl, `EngineApiValidator` impl
- **Interactions**: used by Engine API server wiring to validate newPayload/forkchoice messages and payload attributes based on `EngineApiMessageVersion`.

#### `evm.rs`
- **Role**: Re-exports Ethereum EVM types used by the node.
- **Key items**: `EthEvmConfig`, `EthEvm`

## Key APIs (no snippets)
- `EthereumNode`
- `EthereumAddOns`
- `EthereumEngineValidator`

## Relationships
- **Builds on**: Ethereum consensus (`reth-ethereum-consensus`), Ethereum EVM config (`reth-evm-ethereum`), payload builder (`reth-ethereum-payload-builder`), and the generic node builder (`reth-node-builder`).
