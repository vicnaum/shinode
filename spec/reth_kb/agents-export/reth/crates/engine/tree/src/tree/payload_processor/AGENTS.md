# payload_processor

## Purpose
Implements the engine-tree "payload processing" pipeline: prewarm execution/state caches and compute state roots + trie updates for a block using parallel tasks (prewarm -> multiproof -> sparse trie).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `bal.rs` - EIP-7928 Block Access List helpers (slot iteration, BAL -> `HashedPostState`).
- `configured_sparse_trie.rs` - runtime switch between serial and parallel sparse trie implementations.
- `executor.rs` - `WorkloadExecutor` wrapper for spawning blocking tasks (reuses/creates a Tokio runtime).
- `mod.rs` - main entrypoint; wires tasks together and defines handles/types/constants.
- `multiproof.rs` - multiproof task: turns state updates/BAL into ordered `SparseTrieUpdate`s + proofs (with metrics).
- `prewarm.rs` - cache prewarming task (tx-parallel execution or BAL-driven prefetch) and state-update emission.
- `sparse_trie.rs` - sparse trie task: applies updates and computes final state root + `TrieUpdates`.

## Key APIs (no snippets)
- **Types**: `PayloadProcessor<Evm>`, `PayloadHandle<Tx, Err, R>`, `ExecutionEnv<Evm>`, `WorkloadExecutor`, `SparseTrieUpdate`, `StateRootComputeOutcome`
- **Functions**: `PayloadProcessor::spawn()`, `PayloadProcessor::spawn_cache_exclusive()`, `PayloadHandle::state_root()`, `PayloadHandle::state_hook()`, `update_sparse_trie()`, `bal_to_hashed_post_state()`, `total_slots()`
- **Constants**: `PARALLEL_SPARSE_TRIE_PARALLELISM_THRESHOLDS`, `SPARSE_TRIE_MAX_NODES_SHRINK_CAPACITY`, `SPARSE_TRIE_MAX_VALUES_SHRINK_CAPACITY`

## Relationships
- **Used by**: `reth/crates/engine/tree/src/tree/payload_validator.rs` (`BasicEngineValidator` owns a `PayloadProcessor` for state root / trie-update computation during block validation).
- **Internal flow**: `PrewarmCacheTask` warms caches and emits state updates -> `MultiProofTask` produces `SparseTrieUpdate` proofs in order -> `SparseTrieTask` applies proofs/updates and produces `(state_root, trie_updates)`.
- **Depends on**: `reth_trie_parallel` (proof workers), `reth_trie_sparse{,_parallel}` (sparse trie), `reth_evm`/`revm` (transaction execution for prewarming).

## Notes
- `prewarm.rs` includes special handling for "system" tx types (> 4) to broadcast the first tx to all workers (useful for L2s that embed metadata in the first transaction).
