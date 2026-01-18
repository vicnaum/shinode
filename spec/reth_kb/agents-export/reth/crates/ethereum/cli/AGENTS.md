# cli

## Purpose
`reth-ethereum-cli` crate: Ethereum-specific CLI entrypoint and execution glue that wires `reth-cli-commands` to Ethereum node components (EVM config + beacon consensus) and provides Ethereum chain spec parsing.

## Contents (one hop)
### Subdirectories
- [x] `src/` - clap `Cli` type, command dispatch, and Ethereum chain spec parser.

### Files
- `Cargo.toml` - crate manifest (depends on core CLI crates + Ethereum node wiring).
  - **Key items**: feature flags for OTLP/tracy/jemalloc and CLI "dev" mode.

## Key APIs (no snippets)
- `Cli`, `CliApp`
- `EthereumChainSpecParser`
