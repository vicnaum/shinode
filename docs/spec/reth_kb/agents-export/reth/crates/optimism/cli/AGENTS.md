# cli

## Purpose
`reth-optimism-cli` crate: op-reth command-line interface with custom commands and import codecs.

## Contents (one hop)
### Subdirectories
- [x] `src/` - CLI parsing, commands, and file codecs.

### Files
- `Cargo.toml` - Manifest for op-reth CLI dependencies and features.
  - **Key items**: features `dev`, `otlp`, `jemalloc`; deps `reth-cli-commands`, `reth-optimism-node`
