# src

## Purpose
Implements `reth-execution-types`: shared data types for EVM/block execution outputs in reth, including per-block execution results with bundle state, aggregated multi-block execution outcomes, and `Chain` containers that pair blocks with their derived execution/trie state.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports the execution result/output/outcome types and the `Chain` container.
- **Key items**: modules `execute`, `execution_outcome`, `chain`, optional `serde_bincode_compat`

#### `execute.rs`
- **Role**: Defines `BlockExecutionOutput<T>` (execution result + bundle state) and re-exports `alloy_evm::block::BlockExecutionResult`.
- **Key items**: `BlockExecutionResult`, `BlockExecutionOutput`

#### `execution_outcome.rs`
- **Role**: Aggregated execution outcome representation over one or more blocks: bundle state (with reverts), per-block receipts, and per-block EIP-7685 requests, plus helpers for querying and reverting/splitting outcomes.
- **Key items**: `ExecutionOutcome`, `ExecutionOutcome::single()`, `ExecutionOutcome::from_blocks()`, `ExecutionOutcome::revert_to()`, `ExecutionOutcome::hash_state_slow()`
- **Interactions**: uses `revm::database::BundleState` and trie helpers (`HashedPostState`) to represent post-state and compute hashed representations.

#### `chain.rs`
- **Role**: `Chain` container: holds an ordered map of recovered blocks plus their execution outcome and per-block trie updates/hashed post-state, used in fork-tree / blockchain-tree contexts.
- **Key items**: `Chain`, `Chain::new()`, `Chain::from_block()`, `execution_outcome_at_block()`, iterators over blocks/receipts, helpers to find tx+receipt metadata, and fork-block metadata.

## Key APIs (no snippets)
- `BlockExecutionResult`, `BlockExecutionOutput`
- `ExecutionOutcome`
- `Chain`
