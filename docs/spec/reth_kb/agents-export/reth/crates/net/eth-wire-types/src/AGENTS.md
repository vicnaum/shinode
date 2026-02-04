# src

## Purpose
Defines the data model for the devp2p `eth` wire protocol (eth/66-eth/70) and related sub-protocol payloads: message IDs, request/response wrappers, status handshakes, block/tx/state/receipt message types, broadcast announcements, and SNAP message structures, all with version-aware decoding/encoding.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `blocks.rs`
- **Role**: Request/response payloads for block headers and bodies (`GetBlockHeaders`, `GetBlockBodies`) and their responses.
- **Key items**: `GetBlockHeaders`, `BlockHeaders`, `GetBlockBodies`, `BlockBodies`, `BlockHashOrNumber`, `HeadersDirection`

### `broadcast.rs`
- **Role**: Broadcast payloads for new blocks and transactions: block hash announcements, full block announcements, transaction broadcasts, pooled-transaction hash announcements, and related validation metadata.
- **Key items**: `NewBlockHashes`, `BlockHashNumber`, `NewBlock`, `Transactions`, `SharedTransactions`, `NewPooledTransactionHashes`, `NewPooledTransactionHashes66`, `NewPooledTransactionHashes68`, `ValidAnnouncementData`, `BlockRangeUpdate`
- **Knobs / invariants**: Broadcast formats vary by `EthVersion` (e.g. pooled tx hash announcement shapes across eth/66 vs eth/68+).

### `capability.rs`
- **Role**: Capability negotiation helpers and raw capability framing (`p2p` capability lists and raw payload forwarding).
- **Key items**: `Capability`, `Capabilities`, `RawCapabilityMessage`, `Capability::eth_*()`, `Capabilities::supports_eth_*()`

### `disconnect_reason.rs`
- **Role**: `p2p` disconnect reason codes with RLP encoding/decoding and unknown-reason handling.
- **Key items**: `DisconnectReason`, `UnknownDisconnectReason`

### `header.rs`
- **Role**: Header-request direction abstraction, bridging the legacy `reverse` flag to an explicit direction.
- **Key items**: `HeadersDirection`, `HeadersDirection::new()`, `is_rising()`, `is_falling()`

### `lib.rs`
- **Role**: Crate entrypoint: re-exports all message payload modules and key shared types under a stable public surface (including no-std support).
- **Key items**: `Status`/`UnifiedStatus`, `EthVersion`, `ProtocolMessage`, `EthMessage`, `EthMessageID`, `Capability`, `SnapProtocolMessage`

### `message.rs`
- **Role**: Central protocol message model: message IDs, version-aware decoding/encoding into strongly typed `EthMessage` variants, request/response pairing, and broadcast message wrappers.
- **Key items**: `ProtocolMessage`, `EthMessage`, `EthMessageID`, `RequestPair<T>`, `MessageError`, `MAX_MESSAGE_SIZE`, `ProtocolBroadcastMessage`, `EthBroadcastMessage`
- **Knobs / invariants**: Decoding is gated by `EthVersion` (e.g. `Status` legacy vs eth/69, receipt shapes for eth/69/70, removal of `GetNodeData` in eth/67).

### `primitives.rs`
- **Role**: Abstractions over "network primitives" used in messages (block/header/body, broadcasted vs pooled tx formats, receipt types, new-block payloads), with a standard Ethereum instantiation.
- **Key items**: `NetworkPrimitives`, `NetPrimitivesFor`, `BasicNetworkPrimitives`, `EthNetworkPrimitives`

### `receipts.rs`
- **Role**: Receipt request/response payloads across protocol versions (legacy bloom-bearing receipts vs eth/69+ bloomless receipts; eth/70 partial receipts support).
- **Key items**: `GetReceipts`, `GetReceipts70`, `Receipts`, `Receipts69`, `Receipts70`, `into_with_bloom()`, `last_block_incomplete`

### `snap.rs`
- **Role**: SNAP protocol message payloads (snap/1) for state snapshot exchange: account ranges, storage ranges, bytecodes, and trie nodes.
- **Key items**: `SnapMessageId`, `SnapProtocolMessage`, `GetAccountRangeMessage`, `AccountRangeMessage`, `GetStorageRangesMessage`, `StorageRangesMessage`, `GetByteCodesMessage`, `ByteCodesMessage`, `GetTrieNodesMessage`, `TrieNodesMessage`

### `state.rs`
- **Role**: Legacy eth message payloads for node/state data retrieval (removed in eth/67).
- **Key items**: `GetNodeData`, `NodeData`

### `status.rs`
- **Role**: Status handshake payloads across protocol versions, including a unified superset type and builders to produce the correct on-wire status message by version.
- **Key items**: `UnifiedStatus`, `StatusBuilder`, `Status`, `StatusEth69`, `StatusMessage`, `UnifiedStatus::into_message()`, `spec_builder()`
- **Knobs / invariants**: eth/66-68 uses total difficulty; eth/69 uses earliest/latest history range instead.

### `transactions.rs`
- **Role**: Pooled transaction request/response payloads and helpers.
- **Key items**: `GetPooledTransactions`, `PooledTransactions`, `PooledTransactions::hashes()`

### `version.rs`
- **Role**: Protocol version enums and parsing/encoding helpers for eth and base `p2p` protocol versions.
- **Key items**: `EthVersion`, `ProtocolVersion`, `ParseVersionError`, `EthVersion::ALL_VERSIONS`, `EthVersion::LATEST`

## End-to-end flow (high level)
- During handshake, peers exchange `StatusMessage` (legacy `Status` vs `StatusEth69`) based on negotiated `EthVersion`.
- Wire messages are received as `(EthMessageID, rlp-bytes)` and decoded into `EthMessage` via `ProtocolMessage::decode_message(version, ...)`.
- Requests are wrapped in `RequestPair<T>` to correlate responses with request IDs (e.g. headers/bodies/receipts/pooled transactions).
- Broadcast announcements use dedicated payload types (e.g. `NewBlockHashes`, `NewPooledTransactionHashes*`) and are gated by version where formats differ.
- Optional SNAP messages (`SnapProtocolMessage`) are encoded/decoded independently for state snapshot sync on top of RLPx.

## Key APIs (no snippets)
- **Core**: `EthMessage`, `EthMessageID`, `ProtocolMessage`, `RequestPair<T>`, `EthVersion`
- **Handshake**: `UnifiedStatus`, `StatusMessage`
- **Broadcast**: `NewBlockHashes`, `NewPooledTransactionHashes*`
