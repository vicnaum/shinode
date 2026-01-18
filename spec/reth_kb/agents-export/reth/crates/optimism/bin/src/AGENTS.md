# src

## Purpose
op-reth binary entrypoint and re-export surface for OP crates.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Re-exports op-reth CLI and subsystem crates.
  - **Key items**: modules `cli`, `chainspec`, `consensus`, `evm`, `node`, `rpc`
- `main.rs` - Binary entrypoint that parses CLI and launches node.
  - **Key items**: `Cli::run()`, `OpNode::new()`

## Key APIs (no snippets)
- `op-reth` binary entrypoint
