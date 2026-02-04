# src

## Purpose
Implements `reth-errors`: a small "umbrella" error crate that unifies common reth error types into a single `RethError` and re-exports frequently used error/result aliases.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: re-exports `RethError`/`RethResult` and common error types from consensus, execution, and storage crates.
- **Key items**: `RethError`, `RethResult`, `ConsensusError`, `BlockExecutionError`, `ProviderError`, `DatabaseError`
- **Interactions**: used by higher-level crates to depend on a single error type instead of wiring multiple subsystem errors.
- **Knobs / invariants**: `#![no_std]` + `alloc` for boxed errors.

#### `error.rs`
- **Role**: Defines `RethError` enum variants for core subsystems and an "Other" escape hatch, plus constructors and size assertions.
- **Key items**: `RethError`, `RethError::other()`, `RethError::msg()`, `RethResult<T>`
- **Interactions**: wraps `BlockExecutionError`, `ConsensusError`, `DatabaseError`, `ProviderError`.
- **Knobs / invariants**: size assertions on x86_64 ensure common error types don't grow unexpectedly.

## Key APIs (no snippets)
- **Error type**: `RethError`
- **Result alias**: `RethResult<T>`

## Relationships
- **Depends on**: `reth-consensus`, `reth-execution-errors`, `reth-storage-errors` (re-exported/wrapped).
