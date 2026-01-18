# etl

## Purpose
`reth-etl` crate: ETL data collector that sorts/spills key/value pairs to temp files and provides merged sorted iteration for downstream loading (commonly used to optimize DB insert patterns).

## Contents (one hop)
### Subdirectories
- [x] `src/` - `Collector` and merged iterator implementation.

### Files
- `Cargo.toml` - crate manifest (uses `tempfile`, `rayon`, and `reth-db-api` table encoding/compression traits).

## Key APIs (no snippets)
- `Collector`
- `EtlIter`
