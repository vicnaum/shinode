# transaction

## Purpose
Optimism transaction type definitions and test-only legacy compatibility helpers.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - Module wiring and type aliases for Optimism transactions.
  - **Key items**: `OpTransactionSigned`, re-exports `OpTransaction`, `OpTypedTransaction`, `OpTxType`
- `tx_type.rs` - Tests for compact encoding of OP transaction types.
  - **Key items**: `OpTxType` compact encoding/decoding tests
- `signed.rs` - Legacy signed transaction type kept for consistency tests.
  - **Key items**: `OpTransactionSigned` (legacy test-only)

## Key APIs (no snippets)
- `OpTransactionSigned`, `OpTxType`
