# src

## Purpose
Implements `reth-ethereum-payload-builder`: a basic Ethereum payload builder that selects transactions from the txpool, executes them with an Ethereum EVM config, and produces `EthBuiltPayload` suitable for serving via the Engine API (including fork-specific blob/request limits and well-formedness checks).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Main payload builder implementation: `EthereumPayloadBuilder` implements `reth_basic_payload_builder::PayloadBuilder` and provides `default_ethereum_payload()` to build a payload from best txs, tracking gas, RLP size limits, blob limits, and fees.
- **Key items**: `EthereumPayloadBuilder`, `default_ethereum_payload()`, `is_better_payload` usage, `BlobSidecars` handling, error mapping to `PayloadBuilderError`
- **Interactions**: pulls best transactions from `reth_transaction_pool`; executes via `reth_evm` block builder (`EthEvmConfig` by default); uses `reth_storage_api` state provider to build against the parent state.
- **Knobs / invariants**: enforces `MAX_RLP_BLOCK_SIZE` under Osaka; applies protocol and user-configured per-block blob limits (EIP-7872).

#### `config.rs`
- **Role**: Builder configuration knobs and helpers.
- **Key items**: `EthereumBuilderConfig`, `calculate_block_gas_limit()`
- **Interactions**: gas limit target is clamped to Ethereum's allowed delta (`GAS_LIMIT_BOUND_DIVISOR`) vs parent gas limit; carries extra-data and optional max-blobs-per-block.

#### `validator.rs`
- **Role**: Execution payload "well-formedness" validator for Engine API ingestion: converts `ExecutionData` into a sealed block and checks fork-specific required fields (Shanghai/Cancun/Prague).
- **Key items**: `EthereumExecutionPayloadValidator`, `ensure_well_formed_payload()`
- **Interactions**: delegates to `reth_payload_validator::{shanghai,cancun,prague}`; verifies payload hash matches derived block hash and checks sidecar fields match transactions.

## Key APIs (no snippets)
- `EthereumPayloadBuilder`
- `EthereumBuilderConfig`
- `EthereumExecutionPayloadValidator`

## Relationships
- **Used by**: Ethereum engine/payload service wiring to produce and validate payloads for `engine_forkchoiceUpdated` / `engine_getPayload*` flows.
