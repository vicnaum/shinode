# src

## Purpose
Implements `reth-ethereum`: a feature-gated "meta crate" that re-exports commonly used Ethereum-flavoured reth types and subsystems behind a single dependency, for convenience in downstream crates/apps.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - feature-gated re-exports for primitives, chainspec, CLI, consensus, EVM, network, provider/storage, node/engine, trie, and RPC modules.
  - **Key items**: modules `primitives`, `chainspec`, `cli` (feature), `consensus` (feature), `evm` (feature), `network` (feature), `provider`/`storage` (feature), `node`/`engine` (feature), `trie` (feature), `rpc` (feature)
