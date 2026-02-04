# exex

## Purpose
`reth-exex` crate: execution extensions runtime for reth-hosts long-running ExEx tasks that derive state from canonical execution notifications, report pruning-safe progress, supports catch-up via backfill, and persists notifications with a write-ahead log (WAL).

## Contents (one hop)
### Subdirectories
- [x] `src/` - ExEx manager, context APIs, notification streams/backfill, and WAL implementation.
- [x] `test-data/` - (skip: test fixtures) serialized `.wal` files used to validate WAL decoding/compatibility.

### Files
- `Cargo.toml` - crate manifest (integrates node components, provider/evm APIs, pruning types, tasks/tracing/metrics).
  - **Key items**: `description = "Execution extensions for Reth"`

## Key APIs (no snippets)
- `ExExContext`, `ExExEvent`
- `ExExNotifications`
- `ExExManager`
- `Wal`
