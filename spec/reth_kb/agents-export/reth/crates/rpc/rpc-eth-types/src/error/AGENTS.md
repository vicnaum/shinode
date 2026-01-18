# error

## Purpose
Error types and conversion helpers for `eth` RPC.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Core `eth` RPC error types and mapping helpers.
- **Key items**: `EthApiError`, `EthResult`, `RpcInvalidTransactionError`, `RpcPoolError`, `SignError`

### `api.rs`
- **Role**: Error conversion traits for wrapping core errors.
- **Key items**: `FromEthApiError`, `IntoEthApiError`, `AsEthApiError`, `FromEvmError`, `FromRevert`

## End-to-end flow (high level)
- Convert lower-level errors into `EthApiError`.
- Map invalid transactions to specific RPC error codes.
- Expose conversion traits for EVM and provider errors.

## Key APIs (no snippets)
- `EthApiError`, `RpcInvalidTransactionError`, `FromEvmError`
