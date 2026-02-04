# parallel

## Purpose
`reth-trie-parallel` crate: parallel state root and proof computation using worker pools.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Benchmarks comparing sequential and parallel root computation.
- [x] `src/` - Parallel root/proof implementations, worker pools, stats, and metrics.

### Files
- `Cargo.toml` - Manifest for parallel trie computation and optional metrics.
  - **Key items**: features `metrics`, `test-utils`; deps `reth-trie`, `reth-trie-common`, `reth-trie-sparse`, `rayon`, `tokio`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ParallelStateRoot`, `ParallelProof`, `ProofWorkerHandle`, `StorageRootTargets`
- **Modules / Packages**: `root`, `proof`, `proof_task`, `stats`
- **Functions**: `incremental_root()`, `decoded_multiproof()` - parallel root/proof entrypoints.

## Relationships
- **Depends on**: `reth-trie` and `reth-trie-sparse` for trie traversal, proofs, and sparse node access.
- **Data/control flow**: storage roots computed in parallel feed account trie traversal to finalize state root.
