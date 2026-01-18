# src

## Purpose
Compatibility and conversion helpers between reth primitives and RPC types.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public re-exports for conversion traits.
- **Key items**: `TryFromBlockResponse`, `TryFromReceiptResponse`, `RpcConvert`, `RpcConverter`
- **Interactions**: Re-exports call fee helpers from `alloy_evm::rpc`.

### `block.rs`
- **Role**: Block response conversion trait definitions.
- **Key items**: `TryFromBlockResponse`, `from_block_response()`

### `receipt.rs`
- **Role**: Receipt response conversion trait definitions.
- **Key items**: `TryFromReceiptResponse`, `from_receipt_response()`

### `rpc.rs`
- **Role**: RPC type adapters and signable request abstraction.
- **Key items**: `RpcTypes`, `RpcTransaction`, `RpcReceipt`, `RpcHeader`, `RpcTxReq`, `SignableTxRequest`
- **Knobs / invariants**: `SignableTxRequest` validates typed tx construction.

### `transaction.rs`
- **Role**: Core conversion logic between consensus and RPC transaction types.
- **Key items**: `RpcConvert`, `ConvertReceiptInput`, `ReceiptConverter`, `HeaderConverter`, `FromConsensusHeader`
- **Interactions**: Bridges EVM env (`TxEnvFor`) and RPC request conversions.

## End-to-end flow (high level)
- Use `RpcTypes` to bind a network's RPC response types.
- Convert blocks/receipts with `TryFromBlockResponse` and `TryFromReceiptResponse`.
- Convert transaction requests and responses via `RpcConvert`/`RpcConverter`.
- Build EVM environments from RPC requests using `SignableTxRequest` helpers.

## Key APIs (no snippets)
- `RpcConvert`, `RpcConverter`, `RpcTypes`
- `TryFromBlockResponse`, `TryFromReceiptResponse`
