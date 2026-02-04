# primitives

## Purpose
`reth-ethereum-primitives` crate: Ethereum primitive types for reth (blocks, typed transactions, receipts) and the `EthPrimitives` `NodePrimitives` mapping used across Ethereum node components.

## Contents (one hop)
### Subdirectories
- [x] `src/` - type aliases + receipt/tx helpers.

### Files
- `Cargo.toml` - crate manifest (feature flags for `serde`, `arbitrary`, and optional codec/rpc integrations).

## Key APIs (no snippets)
- `EthPrimitives`
- `TransactionSigned`, `Block`, `Receipt`
