# src

## Purpose
Implements `reth-ethereum-primitives`: Ethereum-specific primitive types for reth (block/tx/receipt aliases, `NodePrimitives` mapping, and receipt encoding/decoding utilities compatible with typed transactions and multiple serialization targets).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines Ethereum type aliases (transactions/blocks) based on `alloy-consensus` and provides `EthPrimitives` as the `NodePrimitives` implementation for Ethereum.
- **Key items**: `Transaction`, `TransactionSigned`, `PooledTransactionVariant`, `Block`, `BlockBody`, `EthPrimitives`
- **Interactions**: used pervasively throughout Ethereum node components (EVM, engine, payload, RPC) as the canonical primitive set.

#### `receipt.rs`
- **Role**: Ethereum receipt type and encoding/decoding support: typed receipt envelopes (EIP-2718), bloom handling, RLP encoding helpers, and receipt-root calculation helpers.
- **Key items**: `Receipt`/`EthereumReceipt`, `TxTy` (trait alias), `calculate_receipt_root_no_memo()`, EIP-2718 encode/decode impls
- **Interactions**: integrates with `alloy_consensus` receipt traits and reth trie-root helpers (`ordered_trie_root_with_encoder`); optionally supports RPC log types (`rpc` feature).

#### `transaction.rs` (test-only)
- **Role**: Legacy `Transaction`/`TransactionSigned` implementation kept for consistency tests against the newer `alloy-consensus` transaction envelope types.
- **Key items**: `Transaction` enum (legacy/EIP-2930/EIP-1559/EIP-4844/EIP-7702), trait impls for encoding/signing and size.
- **Interactions**: compiled under `#[cfg(test)]` (and some feature-gated codec tests).

## Key APIs (no snippets)
- `EthPrimitives`
- `TransactionSigned`, `Block`, `Receipt`

## Relationships
- **Builds on**: `alloy-consensus`/`alloy-eips` for typed transactions/receipts; implements reth's `NodePrimitives` for Ethereum.
