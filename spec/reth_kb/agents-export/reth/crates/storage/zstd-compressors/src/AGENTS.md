# src

## Purpose
Zstd compressors and decompressors with shared dictionaries for receipts/transactions.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Dictionary-backed compressors/decompressors and helpers.
- **Key items**: `RECEIPT_DICTIONARY`, `TRANSACTION_DICTIONARY`,
  `create_tx_compressor()`, `create_receipt_compressor()`, `ReusableDecompressor`
