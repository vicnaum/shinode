# consensus

## Purpose
`reth-consensus` crate: defines the consensus/validation trait surface and shared error types for reth, including optional post-execution validation (`FullConsensus`) and helper implementations for testing (`NoopConsensus`, `TestConsensus`).

## Contents (one hop)
### Subdirectories
- [x] `src/` - consensus traits, errors, and built-in no-op/test implementations.

### Files
- `Cargo.toml` - crate manifest (std/no-std support and `test-utils` feature).
  - **Key items**: `features.std`, `features.test-utils`

## Key APIs (no snippets)
- **Traits**: `HeaderValidator`, `Consensus`, `FullConsensus`
- **Errors**: `ConsensusError`
- **Implementations**: `NoopConsensus`

## Relationships
- **Used by**: block processing components that need an abstract validation interface and unified consensus error reporting.
