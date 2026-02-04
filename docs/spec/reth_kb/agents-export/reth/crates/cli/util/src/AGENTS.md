# src

## Purpose
Implementation of `reth-cli-util`: small reusable helpers for CLI wiring (allocators, cancellation handling, secret-key loading, and common value parsers).

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `allocator.rs` - allocator selection/wrappers for CLI builds (jemalloc/snmalloc/tracy).
- `cancellation.rs` - cancellation primitives/helpers for long-running CLI operations.
- `lib.rs` - module wiring and public re-exports.
- `load_secret_key.rs` - load/parse a secp256k1 secret key from file/hex for CLI usage.
- `parsers.rs` - parsing helpers used as clap value parsers (durations, socket addresses, ether values, block hash-or-number, JSON file reading).
- `sigsegv_handler.rs` - unix-only signal handler hook (or no-op fallback) for better backtraces on crashes/stack overflows.

## Key APIs (no snippets)
- **Parsing**: `parse_socket_address()`, `parse_duration_from_secs_or_ms()`, `parse_ether_value()`, `hash_or_num_value_parser()`
- **Key handling**: `get_secret_key()`, `parse_secret_key_from_hex()`

## Relationships
- **Used by**: CLI crates (`reth-cli`, `reth-cli-commands`) for consistent argument parsing and runtime setup.
