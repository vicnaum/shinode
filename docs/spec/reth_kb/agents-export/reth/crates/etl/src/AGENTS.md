# src

## Purpose
Implements `reth-etl`: a lightweight ETL (extract/transform/load) collector that buffers unsorted key/value pairs, periodically sorts + spills them to temporary files, and later provides a merged sorted iterator (useful for efficient sorted DB inserts and memory management).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - defines the `Collector` type, flush-to-disk logic, and `EtlIter` for merged sorted iteration across ETL files.
  - **Key items**: `Collector`, `Collector::insert()`, `Collector::iter()`, `Collector::clear()`, `EtlIter`

## Relationships
- **Used by**: import/indexing flows that need sorted key order when writing to DB tables (e.g., building hash->number indexes).
