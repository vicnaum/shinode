# src

## Purpose
Implements `reth-network-peers`: peer identity and node record utilities for networking and RPC configuration, including parsing/displaying enode URLs, bootnode lists, and "trusted peer" records that can resolve DNS names to IPs.

## Contents (one hop)
### Subdirectories
- [x] `bootnodes/` - bootnode lists for Ethereum networks and OP-stack networks.

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: defines `PeerId` alias, node-record types (`NodeRecord`, `TrustedPeer`, `AnyNode`), and helpers to convert between peer IDs and secp256k1 public keys (feature-gated).
- **Key items**: `PeerId`, `AnyNode`, `pk2id()` / `id2pk()` (feature `secp256k1`), `WithPeerId<T>`
- **Interactions**: `AnyNode` is used when accepting "any peer identifier" inputs (e.g. trusted peers via RPC).

#### `node_record.rs`
- **Role**: `NodeRecord` type (enode URL form) used by discovery v4 and related tooling: stores IP + tcp/udp ports + peer ID, supports parsing/display, and IPv4-mapped IPv6 normalization.
- **Key items**: `NodeRecord`, `NodeRecord::new()`, `new_with_ports()`, `tcp_addr()`, `udp_addr()`, `NodeRecordParseError`

#### `trusted_peer.rs`
- **Role**: `TrustedPeer` type for persistent/trusted peer configuration: supports domain names and can resolve them to IP addresses to produce a `NodeRecord`.
- **Key items**: `TrustedPeer`, `resolve_blocking()` (std/test), `resolve()` (feature `net`)

## Key APIs (no snippets)
- `PeerId`, `NodeRecord`, `TrustedPeer`, `AnyNode`
