# src

## Purpose
Implements `reth-evm-ethereum`: Ethereum EVM configuration for reth, including revm spec selection, EVM environment/context derivation for block execution and Engine API payload execution, block assembly (header/body roots and fork-specific fields), and receipt building over reth primitive types.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `EthEvmConfig` (executor factory + block assembler), implements `ConfigureEvm` (and `ConfigureEngineEvm` for Engine API `ExecutionData`), and re-exports helper pieces.
- **Key items**: `EthEvmConfig`, `EthEvmConfig::mainnet()`, `EthEvmConfig::ethereum()`, `ConfigureEvm` impl, `ConfigureEngineEvm<ExecutionData>` impl
- **Interactions**: connects `reth-evm` traits to `alloy_evm`'s Ethereum execution context (`EthBlockExecutionCtx`) and `revm::SpecId` hardfork selection.
- **Knobs / invariants**: this crate does not enforce revm feature flags (noted in docs); fork selection uses timestamp/block-number + chainspec hardfork schedule.

#### `config.rs`
- **Role**: Re-exports revm spec selection helpers from `alloy_evm`.
- **Key items**: `revm_spec`, `revm_spec_by_timestamp_and_block_number`

#### `build.rs`
- **Role**: Ethereum block assembler: constructs an `alloy_consensus::Block` (header + body) from execution outputs, computing tx/receipt roots, bloom, withdrawals root, and fork-gated Cancun/Prague fields.
- **Key items**: `EthBlockAssembler`, `BlockAssembler::assemble_block()`
- **Interactions**: uses chainspec to decide withdrawals/request hash/Cancun blob fields; consumes `BlockExecutionResult` (receipts/requests/gas/blob gas) from execution.

#### `receipt.rs`
- **Role**: Receipt builder mapping EVM execution results to reth primitive receipts.
- **Key items**: `RethReceiptBuilder`
- **Interactions**: implements `alloy_evm::eth::receipt_builder::ReceiptBuilder` for `TransactionSigned` -> `Receipt`.

#### `test_utils.rs` (feature `test-utils`)
- **Role**: Test helpers for mocking execution: `MockEvmConfig`/`MockExecutor` return predetermined `ExecutionOutcome` results and can be used anywhere a `ConfigureEvm`/`BlockExecutorFactory` is required.
- **Key items**: `MockEvmConfig`, `MockExecutorProvider`, `MockEvmConfig::extend()`

## Key APIs (no snippets)
- `EthEvmConfig`
- `EthBlockAssembler`
- `RethReceiptBuilder`

## Relationships
- **Used by**: Ethereum node wiring (`reth-node-ethereum`) and any executor/payload builder needing Ethereum-specific EVM env derivation and block assembly.
