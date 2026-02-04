# benches

## Purpose
Benchmarks for `reth-chain-state`, focused on performance of canonical-chain helpers over in-memory overlays.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `canonical_hashes_range.rs` - Criterion benchmark for `MemoryOverlayStateProviderRef::canonical_hashes_range()` across different range sizes and scenarios.

## Key APIs (no snippets)
- **Bench entrypoint**: `canonical_hashes_range.rs`

## Relationships
- **Declared by**: `reth/crates/chain-state/Cargo.toml` (`[[bench]] canonical_hashes_range`, requires `test-utils`).
