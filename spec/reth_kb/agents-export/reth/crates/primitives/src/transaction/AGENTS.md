# transaction

## Purpose
Transaction type definitions, signature recovery helpers, and pooled transaction aliases.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and public re-exports for transaction types and errors.
- **Key items**: `Transaction`, `TransactionSigned`, `InvalidTransactionError`, `recover_signer`, `TxType`
- **Interactions**: Re-exports helpers from `signature`, `util`, and `tx_type`.

### `pooled.rs`
- **Role**: Deprecated pooled transaction alias for recovered pooled txs.
- **Key items**: `PooledTransactionsElementEcRecovered`

### `signature.rs`
- **Role**: Signature recovery helpers for transactions.
- **Key items**: `recover_signer`, `recover_signer_unchecked`

### `tx_type.rs`
- **Role**: Transaction type enum re-export and codec notes.
- **Key items**: `TxType`
- **Knobs / invariants**: Tx type encoding is limited to 2 bits for compact codecs.

### `util.rs`
- **Role**: Signature utility re-exports.
- **Key items**: `crypto` re-exports

## End-to-end flow (high level)
- Use `TransactionSigned`/`TxType` to represent EIP-2718 transactions.
- Recover signers via `recover_signer` helpers.
- Map pooled transaction aliases where needed for compatibility.

## Key APIs (no snippets)
- `Transaction`, `TransactionSigned`, `TxType`
- `recover_signer`, `recover_signer_unchecked`
