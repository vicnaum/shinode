# errors

## Purpose
Error types for the `reth-eth-wire` streams and handshakes: separates `p2p`-level framing/handshake failures from `eth` subprotocol errors and provides helpers to surface disconnect reasons and IO causes.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module glue that re-exports the `eth` and `p2p` error types under a single `errors` namespace.
- **Key items**: `pub use eth::*`, `pub use p2p::*`

### `p2p.rs`
- **Role**: Error taxonomy for `P2PStream` (hello handshake, snappy compression, ping/pong liveness, disconnect handling, capability negotiation).
- **Key items**: `P2PStreamError`, `P2PHandshakeError`, `PingerError`, `CapabilityNotShared`, `MismatchedProtocolVersion`, `Disconnected`, `UnknownReservedMessageId`
- **Interactions**: Produced by `UnauthedP2PStream::handshake()` / `P2PStream` read/write paths and propagated upward into `EthStreamError`.

### `eth.rs`
- **Role**: Error taxonomy for `EthStream` and eth subprotocol handshake (status validation, fork validation, message decoding, size limits, unsupported IDs).
- **Key items**: `EthStreamError`, `EthHandshakeError`, `MessageTooBig`, `InvalidMessage`, `UnsupportedMessage`, `TransactionHashesInvalidLenOfFields`
- **Interactions**: Returned by `UnauthedEthStream::handshake()` and `EthStreamInner::{decode_message, encode_message}`.

## Key APIs (no snippets)
- `P2PStreamError`, `P2PHandshakeError`
- `EthStreamError`, `EthHandshakeError`
