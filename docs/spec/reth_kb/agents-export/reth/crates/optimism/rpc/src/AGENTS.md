# src

## Purpose
OP-specific RPC server implementations and helpers: engine API, eth API wrapper, sequencer client, historical forwarding, and debug witness support.

## Contents (one hop)
### Subdirectories
- [x] `eth/` - OP `eth_` API wrapper with sequencer forwarding, flashblocks, and OP receipt/tx conversions.

### Files
- `lib.rs` - Module wiring and public re-exports for OP RPC types.
  - **Key items**: `OpEthApi`, `OpEthApiBuilder`, `OpReceiptBuilder`, `OpEngineApi`, `OP_ENGINE_CAPABILITIES`
- `engine.rs` - OP Engine API trait and server implementation over `EngineApi`.
  - **Key items**: `OpEngineApi`, `OP_ENGINE_CAPABILITIES`, `OP_STACK_SUPPORT`, `signal_superchain_v1`
- `error.rs` - OP-specific RPC error types and conversions.
  - **Key items**: `OpEthApiError`, `OpInvalidTransactionError`, `TxConditionalErr`, `SequencerClientError`
- `historical.rs` - Historical RPC forwarding layer for pre-bedrock data.
  - **Key items**: `HistoricalRpcClient`, `HistoricalRpc`, `HistoricalRpcService`
- `metrics.rs` - Sequencer client latency metrics.
  - **Key items**: `SequencerMetrics`
- `miner.rs` - Miner extension API for DA/gas limit controls.
  - **Key items**: `OpMinerExtApi`, `OpMinerMetrics`, `set_max_da_size()`, `set_gas_limit()`
- `sequencer.rs` - Sequencer client and request forwarding helpers.
  - **Key items**: `SequencerClient`, `forward_raw_transaction()`, `forward_raw_transaction_conditional()`
- `witness.rs` - Debug witness API backed by OP payload builder.
  - **Key items**: `OpDebugWitnessApi`, `execute_payload()`

## Key APIs (no snippets)
- `OpEthApi`, `OpEthApiBuilder`, `OpEngineApi`
- `SequencerClient`, `HistoricalRpcClient`
- `OpMinerExtApi`, `OpDebugWitnessApi`
