# benches

## Purpose
Benchmarks for `reth-engine-tree` performance-critical components (e.g., channel throughput and state root task behavior).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `channel_perf.rs` - benchmark focused on channel/message throughput used in engine-tree workflows.
- `state_root_task.rs` - benchmark focused on the state root computation task pipeline.

## Key APIs (no snippets)
- **Bench entrypoints**: `channel_perf.rs`, `state_root_task.rs`

## Relationships
- **Declared by**: `reth/crates/engine/tree/Cargo.toml` (`[[bench]] channel_perf`, `[[bench]] state_root_task`).
