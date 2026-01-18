# alloy

## Purpose
Compact codec implementations for alloy consensus/primitives types used in storage encoding.

## Contents (one hop)
### Subdirectories
- [x] `transaction/` - Compact codecs for Ethereum and Optimism transaction types and envelopes.

### Files
- `access_list.rs` - Compact encoding for EIP-2930 access lists.
  - **Key items**: `AccessList`, `AccessListItem`, `Compact` impls
- `authorization_list.rs` - Compact encoding for EIP-7702 authorizations.
  - **Key items**: `Authorization`, `SignedAuthorization`, `Compact` impls
- `genesis_account.rs` - Compact bridges for genesis accounts and storage entries.
  - **Key items**: `GenesisAccount`, `GenesisAccountRef`, `StorageEntries`, `StorageEntry`
- `header.rs` - Compact bridge for block headers with extension struct.
  - **Key items**: `Header`, `HeaderExt`, `Compact` impl for `alloy_consensus::Header`
- `log.rs` - Compact codecs for log and log data types.
  - **Key items**: `Log`, `LogData`, `Compact` impls
- `optimism.rs` - Compact encoding for Optimism receipts using zstd helpers.
  - **Key items**: `CompactOpReceipt`, `OpReceipt`, `CompactZstd`
- `signature.rs` - Compact encoding for ECDSA signatures.
  - **Key items**: `Signature`, `Compact` impl
- `trie.rs` - Compact encoding for trie builder values and branch nodes.
  - **Key items**: `HashBuilderValue`, `BranchNodeCompact`, `TrieMask`
- `txkind.rs` - Compact encoding for transaction kind (create/call).
  - **Key items**: `TxKind`, `TX_KIND_TYPE_CREATE`, `TX_KIND_TYPE_CALL`
- `withdrawal.rs` - Compact encoding for withdrawals and withdrawal lists.
  - **Key items**: `Withdrawal`, `Withdrawals`, `Compact` impls
- `mod.rs` - Module wiring and conditional exports for alloy codecs.
  - **Key items**: `cond_mod!`, `transaction`, `optimism`

## Key APIs (no snippets)
- **Types / Classes / Interfaces**: `Header`, `GenesisAccount`, `Withdrawal`, `CompactOpReceipt`
- **Modules / Packages**: `transaction`, `optimism`
- **Functions**: `to_compact()`, `from_compact()`

## Relationships
- **Depends on**: `alloy-consensus`, `alloy-eips`, `alloy-genesis`, and `alloy-trie`.
- **Data/control flow**: Compact implementations bridge alloy types into storage-optimized
  encodings, reused by database codecs and transaction envelopes.
