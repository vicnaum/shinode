# tree

## Purpose
Implements the in-memory "engine tree" that executes/validates incoming payloads and forkchoice updates, tracks canonical vs fork blocks, and coordinates background persistence while staying responsive to Engine API traffic.

## Contents (one hop)
### Subdirectories
- [x] `payload_processor/` - Prewarming + multiproof + sparse-trie pipeline used to compute state roots and trie updates for executed payloads.

### Files
- `block_buffer.rs` - `BlockBuffer`: bounded FIFO buffer for unconnected blocks (missing parent) with child tracking and eviction.
- `cached_state.rs` - `CachedStateProvider` + `ExecutionCache`: cross-block caches for accounts/storage/bytecode and helpers for proofs/state root.
- `error.rs` - internal error types for block/payload insertion and persistence advancement.
- `instrumented_state.rs` - `InstrumentedStateProvider`: wraps a provider and records per-call + lifetime latency metrics.
- `invalid_headers.rs` - `InvalidHeaderCache`: LRU cache mapping invalid blocks/ancestors with hit-based eviction and metrics.
- `metrics.rs` - `EngineApiMetrics` and related metric groups for execution, validation, persistence, and reorg observability.
- `mod.rs` - module root: defines `EngineApiTreeHandler`/`EngineApiTreeState`/`StateProviderBuilder` and re-exports key tree APIs.
- `payload_validator.rs` - `BasicEngineValidator` and `EngineValidator`: reusable payload/block validation + execution + state root computation.
- `persistence_state.rs` - `PersistenceState`: tracks in-flight persistence tasks and last persisted block.
- `precompile_cache.rs` - `PrecompileCacheMap` and `CachedPrecompile`: per-precompile input->output caching with metrics.
- `state.rs` - `TreeState`: stores executed blocks connected to canonical chain, fork structure, and canonical head tracking.
- `tests.rs` - unit tests for tree internals.
- `trie_updates.rs` - trie update helpers used during state root computation / comparison.

## Key APIs (no snippets)
- **Types**: `EngineApiTreeHandler`, `EngineApiTreeState`, `TreeState`, `BlockBuffer`, `InvalidHeaderCache`, `PersistenceState`, `StateProviderBuilder`
- **Traits**: `EngineValidator` (engine-tree validation interface), `PayloadValidator` (from `reth-engine-primitives`, implemented by chain-specific validators)
- **Functions/Methods**: `EngineApiTreeHandler::new()`, `EngineApiTreeHandler::spawn_new()`, `TreeState::insert_executed()`, `TreeState::prune_finalized_sidechains()`, `BlockBuffer::insert_block()`, `BlockBuffer::remove_block_with_children()`

## Relationships
- **Used by**: `reth/crates/engine/tree/src/engine.rs` (`EngineApiRequestHandler` forwards events into `EngineApiTreeHandler`), `reth/crates/engine/service/src/service.rs` (spawns the tree handler thread and wires it into `EngineService`).
- **Depends on**: `reth-provider` (state/db access), `reth-consensus` (header/body rules), `reth-evm`/`revm` (execution), `reth-trie*` crates (proofs/state-root), `reth-engine-primitives` (Engine API message/event/config types).
- **Data/control flow**:
  - Engine API messages arrive -> validated/executed -> tree state updated (canonical/fork) -> persistence actions scheduled -> metrics/events emitted back to orchestrator.

## Notes
- The design biases toward keeping Engine API handling off the DB write path; persistence is tracked via `PersistenceState` and executed asynchronously via `persistence::PersistenceHandle`.
