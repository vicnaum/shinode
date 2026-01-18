# src

## Purpose
Implements `reth-metrics`: small utilities for metrics usage across reth, including the `Metrics` derive macro re-export and (feature-gated) common helpers like metered tokio channels.

## Contents (one hop)
### Subdirectories
- [x] `common/` - metered tokio mpsc channel wrappers (feature `common`).

### Files
- `lib.rs` - re-exports `metrics_derive::Metrics` and the core `metrics` crate; conditionally exposes `common` helpers.
  - **Key items**: `Metrics` derive macro, `metrics` re-export
