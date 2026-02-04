# src

## Purpose
Core primitive type aliases and re-exports used throughout reth (blocks, transactions, receipts).

## Contents (one hop)
### Subdirectories
- [x] `transaction/` - Transaction types, signature helpers, and pooled aliases.

### Files
- `lib.rs` - Crate entrypoint and re-exports for primitives and fork definitions.
  - **Key items**: `Block`, `Receipt`, `Transaction`, `TxType`, `static_file`, `EthPrimitives`
- `block.rs` - Ethereum block and body type aliases.
  - **Key items**: `Block`, `BlockBody`, `SealedBlock`
- `receipt.rs` - Receipt type alias and helpers.
  - **Key items**: `Receipt`, `gas_spent_by_transactions`

## Key APIs (no snippets)
- `Block`, `BlockBody`, `SealedBlock`
- `Receipt`, `Transaction`, `TxType`
