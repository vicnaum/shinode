# cli

## Purpose
`reth-cli` crate: shared CLI abstractions for reth-based nodes (argument parsing + command execution wiring), intended to be embedded by concrete binaries like `reth` / `op-reth`.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `RethCli` and chainspec parsing helpers.

### Files
- `Cargo.toml` - crate manifest.

## Key APIs (no snippets)
- **Traits**: `RethCli`, `ChainSpecParser`
