# evm

## Purpose
EVM/execution subsystem crates: core EVM execution traits and adapters (`reth-evm`), execution error types (`reth-execution-errors`), and execution outcome/result types (`reth-execution-types`).

## Contents (one hop)
### Subdirectories
- [x] `evm/` - Core EVM abstraction: `ConfigureEvm` + execution/building traits for executing blocks and building new blocks, with optional Engine API helpers and metrics.
- [x] `execution-errors/` - Execution error types: re-exports block execution/validation errors and defines state root/proof/sparse trie/trie witness error families.
- [x] `execution-types/` - Execution result types: `BlockExecutionResult`/`BlockExecutionOutput`, multi-block `ExecutionOutcome`, and `Chain` containers pairing blocks with derived execution/trie state.

### Files
- (none)
