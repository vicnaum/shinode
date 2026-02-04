# payload

## Purpose
`reth-ethereum-payload-builder` crate: Ethereum payload building and validation on top of the txpool API and Ethereum EVM config, producing `EthBuiltPayload` and enforcing fork-specific layout rules for Engine API payloads.

## Contents (one hop)
### Subdirectories
- [x] `src/` - payload builder, configuration, and execution payload validator.

### Files
- `Cargo.toml` - crate manifest (ties together txpool, payload-builder traits, Ethereum EVM config, and payload validators).

## Key APIs (no snippets)
- `EthereumPayloadBuilder`
- `EthereumBuilderConfig`
- `EthereumExecutionPayloadValidator`
