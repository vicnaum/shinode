# txpool

## Purpose
`reth-optimism-txpool` crate: OP transaction pool types, validation, and maintenance for conditional and interop transactions.

## Contents (one hop)
### Subdirectories
- [x] `src/` - OP pooled transactions, validators, supervisor client, and maintenance loops.

### Files
- `Cargo.toml` - Manifest for OP txpool dependencies and metrics.
  - **Key items**: deps `reth-transaction-pool`, `reth-optimism-evm`, `op-alloy-consensus`
