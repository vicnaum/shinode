# src

## Purpose
Implements `reth-cli`: a small abstraction layer for reth-based node CLIs, providing shared traits for parsing args, selecting chainspecs, and running commands on a standard runner.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `chainspec.rs` - `ChainSpecParser` trait + clap `TypedValueParser` adapter; helper `parse_genesis()` (inline JSON or load-from-disk).
- `lib.rs` - `RethCli` trait (name/version/client version + helpers for arg parsing and command execution); exports `chainspec` module.

## Key APIs (no snippets)
- **Traits**: `RethCli`, `ChainSpecParser`
- **Helpers**: `parse_genesis()`

## Relationships
- **Depends on**: `clap` (argument parsing), `reth-cli-runner` (tokio runner), `reth-db::ClientVersion`, `alloy-genesis` (genesis representation).
