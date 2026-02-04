# transaction

## Purpose
Transaction trait abstractions, signature recovery, and validation errors.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Transaction trait entrypoint and module wiring.
- **Key items**: `Transaction`, `FullTransaction`, `SignerRecoverable`, `TransactionMeta`
- **Interactions**: Exposes `signed`, `signature`, `recover`, and error helpers.

### `signed.rs`
- **Role**: Signed transaction trait with recovery helpers.
- **Key items**: `SignedTransaction`, `FullSignedTx`, `RecoveryError`
- **Knobs / invariants**: Checked vs unchecked recovery for pre-EIP-2 transactions.

### `signature.rs`
- **Role**: Signature type re-export and tests.
- **Key items**: `Signature`

### `recover.rs`
- **Role**: Batch recovery helpers with optional rayon support.
- **Key items**: `recover_signers()`, `recover_signers_unchecked()`
- **Knobs / invariants**: Uses rayon when `rayon` feature is enabled.

### `execute.rs`
- **Role**: Trait for filling EVM transaction environment.
- **Key items**: `FillTxEnv`

### `error.rs`
- **Role**: Transaction validation and conversion errors.
- **Key items**: `InvalidTransactionError`, `TransactionConversionError`, `TryFromRecoveredTransactionError`

### `access_list.rs`
- **Role**: Test-only compatibility helpers for access list codecs.
- **Key items**: `RethAccessList`, `RethAccessListItem` (tests)

## End-to-end flow (high level)
- Represent transactions via `Transaction` and `SignedTransaction` traits.
- Recover signers with checked/unchecked helpers.
- Report validation and conversion errors via shared error enums.

## Key APIs (no snippets)
- `Transaction`, `SignedTransaction`, `FillTxEnv`
- `InvalidTransactionError`
