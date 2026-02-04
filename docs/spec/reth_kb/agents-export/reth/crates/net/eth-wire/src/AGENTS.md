# src

## Purpose
Implements `reth-eth-wire`: stream/handshake machinery for RLPx `p2p` + `eth` protocols on top of an encrypted transport (ECIES), including hello/capability negotiation, message-id multiplexing, ping/pong liveness, eth status handshake, and optional combined eth+snap stream handling.

## Contents (one hop)
### Subdirectories
- [x] `errors/` - error types for `P2PStream` and `EthStream` (handshake, decoding, disconnects).

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint: wires modules and re-exports the primary stream types, handshake helpers, and `reth-eth-wire-types` payloads.
- **Key items**: `P2PStream`, `UnauthedP2PStream`, `EthStream`, `UnauthedEthStream`, `HelloMessage*`, `RlpxProtocolMultiplexer`, `Capability`, `ProtocolVersion`

### `protocol.rs`
- **Role**: Defines `Protocol` = a capability plus its reserved message-ID count (needed to perform RLPx message-id multiplexing deterministically).
- **Key items**: `Protocol`, `Protocol::eth()`, `ProtoVersion`, `messages()`
- **Interactions**: Used by hello negotiation (`hello.rs`) and shared capability computation (`capability.rs`).

### `hello.rs`
- **Role**: Hello message types and builders for the `p2p` handshake; tracks both the raw `HelloMessage` and an enriched `HelloMessageWithProtocols` that includes per-capability message counts.
- **Key items**: `HelloMessage`, `HelloMessageWithProtocols`, `HelloMessageBuilder`, `DEFAULT_TCP_PORT`
- **Knobs / invariants**: Capability set defaults to the node's supported `EthVersion::ALL_VERSIONS` and `ProtocolVersion::V5` unless overridden.

### `capability.rs`
- **Role**: Shared capability negotiation and message-id offset assignment for multiplexing (selects highest version per capability name and assigns offsets beyond the reserved `p2p` range).
- **Key items**: `SharedCapability`, `SharedCapabilities`, `SharedCapabilityError`, `UnsupportedCapabilityError`, `find_by_offset()`, `eth_version()`
- **Knobs / invariants**: Offsets must be `> MAX_RESERVED_MESSAGE_ID`; shared capabilities are ordered alphabetically by name.

### `p2pstream.rs`
- **Role**: Implements the RLPx `p2p` stream: performs hello handshake, applies Snappy compression after handshake, demuxes reserved `p2p` message IDs vs subprotocol payloads, and runs ping/pong liveness via `Pinger`.
- **Key items**: `UnauthedP2PStream`, `P2PStream`, `P2PMessage`, `P2PMessageID`, `DisconnectP2P`, `HANDSHAKE_TIMEOUT`, `MAX_RESERVED_MESSAGE_ID`
- **Interactions**: Produces/consumes multiplexed bytes for subprotocol streams and feeds the multiplexer (`multiplex.rs`).
- **Knobs / invariants**: Enforces EIP-706 payload limits and buffer-capacity backpressure (`MAX_P2P_CAPACITY`).

### `pinger.rs`
- **Role**: Internal ping state machine used by `P2PStream` to schedule pings, detect timeouts, and handle late pongs.
- **Key items**: `Pinger`, `PingState`, `PingerEvent`, `on_pong()`, `poll_ping()`

### `handshake.rs`
- **Role**: Eth subprotocol handshake abstraction: defines a pluggable `EthRlpxHandshake` trait and provides the default Ethereum eth-status handshake implementation with fork/genesis/version validation.
- **Key items**: `EthRlpxHandshake`, `UnauthEth`, `EthHandshake`, `EthereumEthHandshake::eth_handshake()`
- **Knobs / invariants**: Validates genesis/chain/version match, fork filter acceptance, message size limits, and eth/69 history range sanity.

### `ethstream.rs`
- **Role**: Typed eth protocol stream built on top of a byte stream: handles `Status` handshake gating and version-aware encode/decode of `EthMessage` payloads.
- **Key items**: `UnauthedEthStream`, `EthStream`, `EthStreamInner`, `MAX_MESSAGE_SIZE`, `start_send_broadcast()`, `start_send_raw()`
- **Interactions**: Uses `ProtocolMessage`/`EthMessage` from `reth-eth-wire-types`; used as primary protocol stream in the multiplexer.

### `eth_snap_stream.rs`
- **Role**: Combined eth + snap message stream over a single connection: decodes/encodes either `EthMessage` or `SnapProtocolMessage` based on message IDs while keeping eth version constraints.
- **Key items**: `EthSnapStream`, `EthSnapMessage`, `EthSnapStreamError`, `decode_message()`, `encode_eth_message()`, `encode_snap_message()`

### `multiplex.rs`
- **Role**: RLPx subprotocol multiplexer: installs multiple subprotocol handlers over a single `P2PStream`, routes messages by capability offsets, and provides a "satellite stream" pattern (primary protocol + dependent satellites like snap).
- **Key items**: `RlpxProtocolMultiplexer`, `ProtocolProxy`, `ProtocolConnection`, `RlpxSatelliteStream`, `install_protocol()`, `into_eth_satellite_stream()`
- **Interactions**: Consumes `SharedCapabilities` from `P2PStream` and drives the primary stream handshake before returning a running satellite stream.

### `disconnect.rs`
- **Role**: Abstraction that lets higher-level streams request a disconnect on the underlying transport (with a `DisconnectReason` when supported).
- **Key items**: `CanDisconnect`

### `test_utils.rs`
- **Role**: Test-only utilities for constructing hello/status fixtures and a passthrough `P2PStream`, plus a small synthetic "test" subprotocol for multiplexer tests.
- **Key items**: `eth_hello()`, `eth_handshake()`, `connect_passthrough()`, `proto::TestProtoMessage`, `proto::TestProtoMessageId`

## Key APIs (no snippets)
- **Streams**: `UnauthedP2PStream` -> `P2PStream`; `UnauthedEthStream` -> `EthStream`
- **Multiplexing**: `RlpxProtocolMultiplexer`, `SharedCapabilities`
- **Handshake**: `EthHandshake` / `EthRlpxHandshake`, `HelloMessageWithProtocols`
