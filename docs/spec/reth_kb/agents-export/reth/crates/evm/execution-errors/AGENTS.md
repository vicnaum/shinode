# execution-errors

## Purpose
`reth-execution-errors` crate: execution-related error types used across reth's execution pipeline, including EVM block execution/validation errors and trie/state-root/proof/witness error families.

## Contents (one hop)
### Subdirectories
- [x] `src/` - error definitions and re-exports.

### Files
- `Cargo.toml` - crate manifest (depends on `alloy-evm`, `reth-storage-errors`, and trie utilities).

## Key APIs (no snippets)
- `BlockExecutionError`, `BlockValidationError`
- `StateRootError`, `TrieWitnessError`
