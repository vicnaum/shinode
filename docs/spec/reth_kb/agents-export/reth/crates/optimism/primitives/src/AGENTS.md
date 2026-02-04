# src

## Purpose
Optimism primitive types: OP block/receipt types, bedrock constants, and transaction wrappers.

## Contents (one hop)
### Subdirectories
- [x] `transaction/` - OP transaction types and test-only legacy helpers.

### Files
- `lib.rs` - Crate entrypoint defining `OpPrimitives` and type aliases.
  - **Key items**: `OpPrimitives`, `OpBlock`, `OpBlockBody`, `OpTransactionSigned`, `OpReceipt`
- `bedrock.rs` - Bedrock and replayed-transaction constants.
  - **Key items**: `BEDROCK_HEADER`, `BEDROCK_HEADER_HASH`, `BLOCK_NUMS_REPLAYED_TX`, `is_dup_tx()`
- `receipt.rs` - Receipt utilities and deposit receipt helpers.
  - **Key items**: `DepositReceipt`, `OpReceipt` serde compat (feature-gated)

## Key APIs (no snippets)
- `OpPrimitives`, `OpBlock`, `OpTransactionSigned`, `OpReceipt`
