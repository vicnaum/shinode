# it

## Purpose
Integration tests for trace/debug RPC helper utilities.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `main.rs`
- **Role**: Test module wiring for trace tests.
- **Key items**: `trace`

### `trace.rs`
- **Role**: Local-node trace/debug integration tests using helpers.
- **Key items**: `trace_many_blocks()`, `trace_filters()`, `debug_trace_block_entire_chain()`
- **Knobs / invariants**: Tests are no-ops unless `RETH_RPC_TEST_NODE_URL` is set.

## End-to-end flow (high level)
- Build RPC client from environment variable.
- Run trace/debug streams against a live node.
- Print or inspect streamed results for errors.

## Key APIs (no snippets)
- `TraceApiExt`, `DebugApiExt`, `parse_env_url()`
