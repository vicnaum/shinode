# transaction

## Purpose
Compact codec implementations for Ethereum and Optimism transaction types and envelopes.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Implements `Compact` for `EthereumTypedTransaction` and wires transaction modules.
- **Key items**: `EthereumTypedTransaction`, `CompactEnvelope`, `Envelope`, `FromTxCompact`,
  `ToTxCompact`
- **Interactions**: Delegates to per-tx-type modules and `ethereum.rs` helpers.

### `ethereum.rs`
- **Role**: Compact serialization for transaction envelopes with signature handling and optional
  zstd compression.
- **Key items**: `ToTxCompact`, `FromTxCompact`, `Envelope`, `CompactEnvelope`
- **Interactions**: Uses `TxType` for type discrimination and `Signature` for rehydration.
- **Knobs / invariants**: Flag byte packs signature/type/compression bits; compresses inputs >= 32B.

### `eip1559.rs`
- **Role**: Compact bridge struct for `TxEip1559`.
- **Key items**: `TxEip1559` fields (fees, access list, input), `Compact` impl

### `eip2930.rs`
- **Role**: Compact bridge struct for `TxEip2930`.
- **Key items**: `TxEip2930`, `AccessList`, `Compact` impl

### `eip4844.rs`
- **Role**: Compact bridge struct for `TxEip4844` blob transactions.
- **Key items**: `TxEip4844`, `blob_versioned_hashes`, `max_fee_per_blob_gas`, placeholder bit
- **Knobs / invariants**: Placeholder field preserves bitflag layout for compatibility.

### `eip7702.rs`
- **Role**: Compact bridge struct for `TxEip7702` set-code transactions.
- **Key items**: `TxEip7702`, `authorization_list`, `Compact` impl

### `legacy.rs`
- **Role**: Compact bridge struct for legacy transactions.
- **Key items**: `TxLegacy`, `gas_price`, `chain_id`, `Compact` impl

### `txtype.rs`
- **Role**: Compact mapping for `TxType` identifiers with extended type support.
- **Key items**: `TxType::to_compact()`, `TxType::from_compact()`,
  `COMPACT_EXTENDED_IDENTIFIER_FLAG`

### `optimism.rs`
- **Role**: Compact support for Optimism deposit transactions and typed envelopes.
- **Key items**: `TxDeposit`, `OpTxType`, `OpTypedTransaction`, `CompactEnvelope` impls
- **Interactions**: Extends tx-type encoding to include deposit identifiers.

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `CompactEnvelope`, `Envelope`, `TxEip1559`, `TxEip2930`,
  `TxEip4844`, `TxEip7702`, `TxLegacy`, `TxDeposit`
- **Modules / Packages**: `ethereum`, `txtype`
- **Functions**: `to_compact()`, `from_compact()`, `to_tx_compact()`, `from_tx_compact()`

## Relationships
- **Depends on**: `alloy-consensus` transaction types and `alloy-eips` access lists.
- **Depends on**: `reth-zstd-compressors` for optional envelope compression.
- **Data/control flow**: tx type identifiers drive dispatch into per-type compact codecs, with
  envelope helpers adding signature and compression metadata.

## End-to-end flow (high level)
- Encode tx type identifier and signature metadata.
- Serialize the concrete transaction payload with Compact.
- Optionally compress payload for large inputs.
- Decode by reading flags, inflating payload, and constructing typed transactions.
