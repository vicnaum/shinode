# src

## Purpose
Implements `reth-consensus`: core consensus/validation traits and shared error types for validating headers and blocks (pre- and post-execution), plus a no-op and test consensus implementation.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Defines the primary consensus traits (`HeaderValidator`, `Consensus`, `FullConsensus`) and the shared error model (`ConsensusError`, related helpers) used across reth's validation pipeline.
- **Key items**: `HeaderValidator`, `Consensus`, `FullConsensus`, `ConsensusError`, `HeaderConsensusError`
- **Interactions**: implemented by concrete consensus engines; consumed by sync/pipeline code that needs uniform validation hooks.
- **Knobs / invariants**: provides default `HeaderValidator::validate_header_range()` behavior (standalone validation + parent-chain validation in ascending order).

#### `noop.rs`
- **Role**: A "do nothing" consensus engine for testing and special cases where validation is intentionally bypassed.
- **Key items**: `NoopConsensus`, `NoopConsensus::arc()`
- **Interactions**: implements `HeaderValidator`, `Consensus`, and `FullConsensus` by always returning `Ok(())`.
- **Knobs / invariants**: explicitly not suitable for production use (no security/consensus guarantees).

#### `test_utils.rs`
- **Role**: Test helper consensus implementation with toggles to force validation failures in different phases.
- **Key items**: `TestConsensus`, `set_fail_validation()`, `set_fail_body_against_header()`
- **Interactions**: used in tests to exercise networking/sync behavior when blocks/headers are rejected.
- **Knobs / invariants**: separate flags for "header/overall" validation vs "body-against-header" failures.

## Key APIs (no snippets)
- **Traits**: `HeaderValidator`, `Consensus`, `FullConsensus`
- **Errors**: `ConsensusError`, `HeaderConsensusError`
- **Implementations**: `NoopConsensus`, `TestConsensus`

## Relationships
- **Used by**: pipeline stages, sync/networking components, and execution validation flows that need to validate blocks/headers consistently.
- **Depends on**: primitives and execution result types (`reth-primitives-traits`, `reth-execution-types`) for block/header representations and post-execution outcome validation.

## End-to-end flow (high level)
- A component chooses/constructs a concrete consensus engine implementing `HeaderValidator` (+ optionally `Consensus`/`FullConsensus`).
- Sync/pipeline validates standalone headers, then headers against parents, and bodies against headers.
- After execution, `FullConsensus::validate_block_post_execution()` can validate receipts/state roots/etc. using the execution result.
- Errors are normalized into `ConsensusError` (and `HeaderConsensusError` for range validation context).
