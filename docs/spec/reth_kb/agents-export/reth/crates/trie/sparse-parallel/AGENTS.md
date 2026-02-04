# sparse-parallel

## Purpose
`reth-trie-sparse-parallel` crate: parallel sparse MPT implementation that splits the trie into
upper and lower subtries for concurrent reveal and hash updates.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Parallel sparse trie implementation, lower subtrie management, and metrics.

### Files
- `Cargo.toml` - Crate manifest for parallel sparse trie implementation and features.
  - **Key items**: features `std`, `metrics`; deps `reth-trie-sparse`, `reth-trie-common`,
    optional `rayon`, `metrics`/`reth-metrics`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `ParallelSparseTrie`, `ParallelismThresholds`,
  `LowerSparseSubtrie`, `SparseSubtrie`
- **Modules / Packages**: `trie`, `lower`, `metrics`
- **Functions**: `with_parallelism_thresholds()`, `reveal_nodes()`, `root()`

## Relationships
- **Depends on**: `reth-trie-sparse` for sparse node types and common interfaces.
- **Depends on**: `reth-trie-common` for masks, prefix sets, and RLP utilities.
- **Data/control flow**: lower subtrie hash updates feed upper subtrie hashing to produce the
  final root; optional metrics record update counts and latencies.
