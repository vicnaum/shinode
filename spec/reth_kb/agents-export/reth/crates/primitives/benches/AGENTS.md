# benches

## Purpose
Criterion benchmarks for transaction signature recovery and blob validation.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `recover_ecdsa_crit.rs`
- **Role**: Benchmarks ECDSA signer recovery for a sample transaction.
- **Key items**: `criterion_benchmark()`

### `validate_blob_tx.rs`
- **Role**: Benchmarks EIP-4844 blob validation with varying blob counts.
- **Key items**: `blob_validation()`, `validate_blob_tx()`
- **Knobs / invariants**: Runs across `MAX_BLOBS_PER_BLOCK_DENCUN` and KZG settings.

## End-to-end flow (high level)
- Construct sample transactions and sidecars for benchmarks.
- Run criterion benchmarks for signer recovery and blob validation.

## Key APIs (no snippets)
- Criterion benchmarks for primitives
