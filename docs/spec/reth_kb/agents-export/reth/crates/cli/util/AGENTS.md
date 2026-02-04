# util

## Purpose
`reth-cli-util` crate: shared utilities used across reth's CLI crates (argument/value parsing, allocator configuration, cancellation helpers, and key loading).

## Contents (one hop)
### Subdirectories
- [x] `src/` - implementation of parsing + runtime/allocator helpers.

### Files
- `Cargo.toml` - crate manifest (allocator-related feature flags).

## Key APIs (no snippets)
- **Re-exports**: `parse_socket_address()`, `parse_duration_from_secs_or_ms()`, `parse_ether_value()`, `hash_or_num_value_parser()`, `get_secret_key()`
