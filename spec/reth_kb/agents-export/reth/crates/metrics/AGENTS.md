# metrics

## Purpose
`reth-metrics` crate: metrics utilities used across reth-re-exports the `Metrics` derive macro and provides optional common helpers (e.g., metered tokio mpsc channels).

## Contents (one hop)
### Subdirectories
- [x] `src/` - `Metrics` re-export and common utilities.

### Files
- `Cargo.toml` - crate manifest (feature `common` pulls in tokio/futures utilities for metered channels).

## Key APIs (no snippets)
- `Metrics` (derive macro re-export)
