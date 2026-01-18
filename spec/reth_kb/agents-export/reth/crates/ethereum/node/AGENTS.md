# node

## Purpose
`reth-node-ethereum` crate: Ethereum node wiring for reth-provides the canonical `EthereumNode` type configuration, component builders (executor/pool/network/consensus), payload builder integration, Engine API validators, and RPC add-on wiring for a full Ethereum node.

## Contents (one hop)
### Subdirectories
- [x] `src/` - node types/builders, Engine API validator, payload builder wiring, and EVM re-exports.
- [x] `tests/` - end-to-end and integration tests for Ethereum node wiring and behaviours.

### Files
- `Cargo.toml` - crate manifest (pulls together Ethereum consensus/EVM/payload/engine primitives and node-builder/RPC subsystems; enables required `revm` features for Ethereum).

## Key APIs (no snippets)
- `EthereumNode`
- `EthereumAddOns`
- `EthereumEngineValidator`
