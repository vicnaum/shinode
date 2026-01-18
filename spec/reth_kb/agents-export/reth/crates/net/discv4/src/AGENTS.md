# src

## Purpose
Implements Discovery v4 (devp2p discv4) for reth: maintains a Kademlia-like routing table over UDP, performs PING/PONG bonding and `FINDNODE` lookups, and can infer an external/public IP in NAT environments (with optional periodic re-resolution).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Main protocol/service implementation: `Discv4` frontend + `Discv4Service` driving UDP I/O, routing table maintenance, lookups, bonding, and update/event streams.
- **Key items**: `Discv4`, `Discv4Service`, `DiscoveryUpdate` (routing-table change notifications), constants like `DEFAULT_DISCOVERY_ADDRESS`
- **Interactions**: Uses `reth-net-nat` for public IP resolution; uses `reth-network-peers::NodeRecord` as peer address primitive.

#### `config.rs`
- **Role**: Configuration surface for discv4 performance and behavior (timeouts, buffers, bootstrap nodes, EIP-868 behavior, NAT resolver + interval).
- **Key items**: `Discv4Config`, `Discv4ConfigBuilder`, `resolve_external_ip_interval()` (builds `ResolveNatInterval`)

#### `proto.rs`
- **Role**: Wire protocol types and encode/decode logic for UDP packets (message IDs, RLP payload encoding, secp256k1 recoverable signatures, packet hash verification).
- **Key items**: `Message`, `MessageId`, `Packet`, `Ping`/`Pong`, `FindNode`, `Neighbours`, `EnrRequest`/`EnrResponse`, `NodeEndpoint`

#### `node.rs`
- **Role**: Adapters for using `PeerId`/node IDs in kbucket tables (compute Kademlia key via keccak hash).
- **Key items**: `kad_key()`, `NodeKey`

#### `table.rs`
- **Role**: Small helper tables for tracking per-peer state needed by the service (e.g. last PONG per node/IP).
- **Key items**: `PongTable`

#### `error.rs`
- **Role**: Error types for decoding packets and interacting with the service via channels.
- **Key items**: `DecodePacketError`, `Discv4Error`

#### `test_utils.rs`
- **Role**: Feature-gated testing utilities, including a mock discovery peer and helpers for constructing test `Discv4` instances.
- **Key items**: `MockDiscovery`, `MockCommand`, `MockEvent`, `create_discv4()`

## Key APIs (no snippets)
- `Discv4`, `Discv4Service`
- `Discv4Config`, `Discv4ConfigBuilder`
- `Message`/`Packet` (protocol encoding/decoding)
