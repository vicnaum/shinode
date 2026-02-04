# src

## Purpose
Provides concrete `InvalidBlockHook` implementations for debugging invalid blocks, including witness generation and optional comparison against a "healthy" node.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - module wiring; exports `InvalidBlockWitnessHook`.
- `witness.rs` - `InvalidBlockWitnessHook`: re-executes invalid blocks, generates `ExecutionWitness`, writes JSON artifacts/diffs, and can fetch/compare a witness via `DebugApiClient`.

## Key APIs (no snippets)
- **Types**: `InvalidBlockWitnessHook<P, E>`
- **Implements**: `reth-engine-primitives::InvalidBlockHook`

## Relationships
- **Used by**: engine payload validation flows that want to capture diagnostics on invalid blocks (plugged in via `InvalidBlockHook`).
- **Depends on**: `reth-evm`/`reth-revm` (re-execution), `reth-trie` (hashed state + trie updates), `reth-rpc-api` debug client (optional healthy-node comparison).

## Notes
- This module is intentionally I/O heavy (writes artifacts to disk) and should be configured/used accordingly in production vs debugging contexts.
