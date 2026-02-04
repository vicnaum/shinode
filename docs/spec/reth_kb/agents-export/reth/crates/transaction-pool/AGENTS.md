# transaction-pool

## Purpose
`reth-transaction-pool` crate: mempool implementation, validation, ordering, and maintenance.

## Contents (one hop)
### Subdirectories
- [x] `benches/` - Transaction pool performance benchmarks.
- [x] `docs/` - (skip: mermaid diagram assets).
- [x] `src/` - Pool implementation, validation, and maintenance.
- [x] `test_data/` - (skip: test fixture JSON data).
- [x] `tests/` - Integration tests for pool behavior.

### Files
- `Cargo.toml` - Manifest with pool features, benches, and test utils.
  - **Key items**: features `serde`, `test-utils`, `arbitrary`
