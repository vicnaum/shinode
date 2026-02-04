# net

## Purpose
Networking subsystem crates for reth: peer discovery (discv4/discv5, DNS), cryptography (ECIES), wire protocols (eth-wire and types), peer management, P2P sync/downloaders, the network manager/swarm implementation, and supporting types/APIs.

## Contents (one hop)
### Subdirectories
- [x] `banlist/` - Peer/IP banning utilities: `BanList` with optional expiry and `IpFilter` for CIDR allow-lists.
- [x] `discv4/` - Discovery v4: UDP-based peer discovery with kbucket routing table, bonding/lookups, and NAT external IP detection (`Discv4`/`Discv4Service`).
- [x] `discv5/` - Discovery v5 wrapper: starts `sigp/discv5`, builds local ENR (fork kv-pairs), filters discovered peers, and emits reachable `NodeRecord`s (`Discv5`, `Config`).
- [x] `dns/` - EIP-1459 DNS discovery: verify ENR trees, rate-limit TXT lookups, and stream discovered `NodeRecord`s (`DnsDiscoveryService`, `DnsNodeRecordUpdate`).
- [x] `downloaders/` - Header/body downloaders plus file import clients and metrics (`ReverseHeadersDownloader`, `BodiesDownloader`, `FileClient`, `ChunkedFileReader`).
- [x] `ecies/` - RLPx ECIES framed transport: AUTH/ACK handshake, encrypted header/body framing, and Tokio codec/stream wrappers (`ECIESStream`, `ECIESCodec`).
- [x] `eth-wire/` - RLPx `p2p` + `eth` stream plumbing: hello/capability negotiation, message multiplexing, ping/pong, and typed `EthStream`/`P2PStream` over ECIES (`RlpxProtocolMultiplexer`).
- [x] `eth-wire-types/` - Devp2p `eth`/SNAP payload types and versioned decoding/encoding (eth/66-70): messages, IDs, status, requests/responses, broadcasts (`EthMessage`, `EthVersion`).
- [x] `nat/` - NAT helpers: resolve external/public IP via public IP services, domain/net-if, and interval-based polling (`NatResolver`, `ResolveNatInterval`).
- [x] `network/` - Full network stack: discovery, sessions, peers, request routing, tx gossip, and network handle APIs (`NetworkManager`, `NetworkHandle`).
- [x] `network-api/` - Network interfaces/types: peer management + event streams + downloader providers and a noop impl (`FullNetwork`, `Peers`, `NetworkEvent`).
- [x] `network-types/` - Shared networking config/types: peers (addr/kind/state/reputation/backoff) and session limits/timeouts (`PeersConfig`, `ReputationChangeKind`, `SessionsConfig`).
- [x] `p2p/` - Shared p2p traits/types for download clients, errors, and sync state (`DownloadClient`, `HeadersClient`, `BodiesClient`).
- [x] `peers/` - Peer record utilities: `PeerId`/`NodeRecord`/`TrustedPeer`/`AnyNode` parsing and bootnode lists for Ethereum and OP-stack networks.

### Files
- (none)
