# src

## Purpose
Implements `reth-evm`: core EVM abstraction layer for reth. Defines the `ConfigureEvm` trait (EVM env/context derivation + executor factory + block assembler), execution traits/utilities for executing blocks and building new blocks, and optional helpers for Engine API payload execution and metrics.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `ConfigureEvm` (the primary EVM configuration trait) and re-exports key building blocks from `alloy_evm` plus local modules (`execute`, `engine`, etc.).
- **Key items**: `ConfigureEvm`, `NextBlockEnvAttributes` (via re-exports), `EvmEnvFor`/type helpers (via `aliases`), `ConfigureEngineEvm` (feature `std`)
- **Interactions**: consumed by node wiring and payload builders to execute blocks and/or build new blocks with chain-specific configs (e.g., Ethereum vs OP).

#### `aliases.rs`
- **Role**: Type aliases and helper trait bounds for working with `ConfigureEvm`-based stacks.
- **Key items**: `EvmFactoryFor`, `SpecFor`, `BlockEnvFor`, `EvmFor`, `EvmEnvFor`, `ExecutionCtxFor`, `InspectorFor`

#### `execute.rs`
- **Role**: Execution/building traits and helpers: defines the `Executor` trait (execute one/batch blocks against `State<DB>`), block assembly input types, and exports execution/output/error types.
- **Key items**: `Executor`, `BlockAssemblerInput`, `BlockAssembler`, `BlockBuilder`, `BasicBlockExecutor`, `BasicBlockBuilder`, `BlockExecutionOutput`, `ExecutionOutcome`
- **Interactions**: bridges `reth_execution_errors` + `reth_execution_types` with `alloy_evm` execution primitives; integrates trie update types (`TrieUpdates`, `HashedPostState`) and provider state access.

#### `engine.rs` (feature `std`)
- **Role**: Engine API payload execution helpers: trait extension for `ConfigureEvm` to derive EVM env/context/tx-iterator from an Engine API payload (and parallelize decode/recovery work).
- **Key items**: `ConfigureEngineEvm`, `ExecutableTxTuple`, `ExecutableTxIterator`

#### `either.rs`
- **Role**: `Either<A,B>` adapter that implements `Executor` by delegating to one of two executor implementations.
- **Key items**: `Executor` impl for `futures_util::future::Either`

#### `noop.rs`
- **Role**: No-op EVM config wrapper for satisfying `ConfigureEvm` bounds in tests/type-level plumbing (panics if invoked).
- **Key items**: `NoopEvmConfig`

#### `metrics.rs` (feature `metrics`)
- **Role**: Execution metrics helpers (gas processed, timings, state load/update histograms).
- **Key items**: `ExecutorMetrics`, `ExecutorMetrics::metered_one()`

#### `test_utils.rs` (tests / feature `test-utils`)
- **Role**: Small helpers for testing `BasicBlockExecutor` state access.
- **Key items**: `BasicBlockExecutor::with_state()`, `with_state_mut()`

## Key APIs (no snippets)
- `ConfigureEvm`
- `Executor` / `BasicBlockExecutor`
- `BlockAssemblerInput`
- `ConfigureEngineEvm` (std)

## Relationships
- **Builds on**: `alloy_evm`/`revm` for execution primitives, and reth's execution error/result crates for error typing and execution outcomes.
