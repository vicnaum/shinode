# src

## Purpose
Parallel state root and proof computation with worker pools, metrics, and stats tracking.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint exposing parallel root/proof modules and helpers.
- **Key items**: `StorageRootTargets`, `stats`, `root`, `proof`, `proof_task`, `metrics` (feature)
- **Interactions**: Gated exports for metrics and proof-task metrics.

### `metrics.rs`
- **Role**: Metrics for parallel state root computation.
- **Key items**: `ParallelStateRootMetrics`, `ParallelTrieMetrics`, `record_state_trie()`
- **Interactions**: Wraps `TrieRootMetrics` and consumes `ParallelTrieStats`.

### `proof_task_metrics.rs`
- **Role**: Metrics for proof-task workers and cursor operations.
- **Key items**: `ProofTaskTrieMetrics`, `ProofTaskCursorMetrics`, `ProofTaskCursorMetricsCache`, `record()`, `record_spans()`
- **Interactions**: Uses `TrieCursorMetrics` and `HashedCursorMetrics` caches from `reth_trie`.

### `proof_task.rs`
- **Role**: Worker-pool execution for parallel multiproof and storage proof computation.
- **Key items**: `ProofWorkerHandle`, `ProofResultMessage`, `ProofResultContext`, `StorageProofInput`, `AccountMultiproofInput`, `StorageProofWorker`, `AccountProofWorker`
- **Interactions**: Uses `TrieWalker`, `HashBuilder`, blinded node providers, and `MultiAddedRemovedKeys` while coordinating storage proofs.
- **Knobs / invariants**: `v2_proofs_enabled`, worker counts, and `collect_branch_node_masks` shape results; workers own dedicated DB transactions.

### `proof.rs`
- **Role**: High-level parallel proof orchestrator built on worker pools.
- **Key items**: `ParallelProof`, `decoded_multiproof()`, `storage_proof()`, `extend_prefix_sets_with_targets()`, `with_v2_proofs_enabled()`
- **Interactions**: Dispatches tasks via `ProofWorkerHandle` and aggregates `DecodedMultiProof` results.

### `root.rs`
- **Role**: Parallel incremental state root calculator with optional trie updates.
- **Key items**: `ParallelStateRoot`, `incremental_root()`, `incremental_root_with_updates()`, `ParallelStateRootError`, `get_runtime_handle()`
- **Interactions**: Spawns blocking storage-root tasks and walks the account trie to build the root.
- **Knobs / invariants**: `retain_updates` toggles update collection; errors if storage root returns progress.

### `stats.rs`
- **Role**: Statistics tracking for parallel trie computations.
- **Key items**: `ParallelTrieStats`, `ParallelTrieTracker`, `inc_branch()`, `inc_leaf()`, `inc_missed_leaves()`, `finish()`
- **Interactions**: Wraps `TrieTracker` stats and optionally collects cursor metrics.

### `storage_root_targets.rs`
- **Role**: Aggregates storage root targets and prefix sets for parallel processing.
- **Key items**: `StorageRootTargets`, `new()`, `count()`, `IntoParallelIterator` impl
- **Interactions**: Combines account prefix sets with storage prefix sets and drives parallel iteration.

## End-to-end flow (high level)
- Build prefix sets and `StorageRootTargets` from state changes.
- Spawn worker pools for account and storage proofs via `ProofWorkerHandle`.
- Dispatch storage proof/root tasks in parallel, tracking availability and metrics.
- Walk the account trie and integrate storage roots into `HashBuilder`.
- Produce parallel roots or multiproofs, recording stats and optional metrics.
