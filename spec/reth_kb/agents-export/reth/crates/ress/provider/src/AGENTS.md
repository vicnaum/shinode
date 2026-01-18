# src

## Purpose
Reth-backed provider implementation for the RESS protocol.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Implements `RessProtocolProvider` using reth storage, EVM execution, and witness caching.
  - **Key items**: `RethRessProtocolProvider`, `generate_witness()`, `block_by_hash()`
  - **Knobs / invariants**: `max_witness_window`, `witness_max_parallel`, LRU witness cache.
- `pending_state.rs` - Tracks executed and invalid blocks for witness construction.
  - **Key items**: `PendingState`, `maintain_pending_state()`
  - **Interactions**: Consumes `ConsensusEngineEvent` stream to keep pending state updated.
- `recorder.rs` - EVM database wrapper that records accessed state during execution.
  - **Key items**: `StateWitnessRecorderDatabase`
  - **Interactions**: Collects hashed account/storage accesses for witness generation.

## Key APIs (no snippets)
- `RethRessProtocolProvider`
- `PendingState`, `maintain_pending_state`
