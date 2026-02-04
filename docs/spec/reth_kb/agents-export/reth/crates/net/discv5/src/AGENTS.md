# src

## Purpose
Implements `reth-discv5`: a reth-oriented wrapper around `sigp/discv5` that bootstraps discovery v5, builds/updates the local ENR (including EIP-868 fork IDs), filters discovered peers, and exposes discovered peers as `NodeRecord` + optional `ForkId` for upstream networking.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `config.rs`
- **Role**: Configuration surface for starting discv5: wraps `discv5::Config`, manages boot nodes (signed ENRs or unsigned "enode" multiaddrs), sets fork kv-pairs and additional ENR kv-pairs, and defines lookup/bootstrapping intervals.
- **Key items**: `Config`, `ConfigBuilder`, `BootNode`, `DEFAULT_DISCOVERY_V5_LISTEN_CONFIG`, `DEFAULT_SECONDS_LOOKUP_INTERVAL`, `amend_listen_config_wrt_rlpx()`
- **Interactions**: Feeds `Config` into `Discv5::start()` / `build_local_enr()` in `lib.rs`.
- **Knobs / invariants**: ENR is limited to one IP per IP-version; RLPx IP can overwrite discv5 listen IP of same version.

### `enr.rs`
- **Role**: Bridges identity types between discv5 and reth's v4-style peer IDs: converts ENRs/peer IDs between discv5 node IDs, libp2p peer IDs, and `reth-network-peers::PeerId`.
- **Key items**: `enr_to_discv4_id()`, `discv4_id_to_discv5_id()`, `discv4_id_to_multiaddr_id()`, `EnrCombinedKeyWrapper`
- **Interactions**: Used by `lib.rs` for banning (`discv4_id_to_discv5_id`) and for producing `NodeRecord`s from ENRs (`enr_to_discv4_id`).

### `error.rs`
- **Role**: Error taxonomy for operations around the underlying `discv5::Discv5` node (initialization, listen config mismatches, fork ID decoding, and node/peer compatibility).
- **Key items**: `Error` (variants: `AddNodeFailed`, `IncompatibleKeyType`, `ForkMissing`, `ListenConfigMisconfigured`, `Discv5Error`)

### `filter.rs`
- **Role**: Filtering predicates applied to discovered peers' ENRs (e.g. disallowing specific network stack keys such as `eth2`) to decide whether to pass peers to RLPx or drop them.
- **Key items**: `FilterOutcome`, `MustNotIncludeKeys`, `MustIncludeKey`, `MustNotIncludeKeys::filter()`, `add_disallowed_keys()`
- **Interactions**: Called by `Discv5::filter_discovered_peer()` in `lib.rs` during event processing.

### `lib.rs`
- **Role**: Main wrapper and orchestration: starts discv5, constructs local ENR (fork kv-pair + sockets), bootstraps with configured peers, spawns background kbucket maintenance, processes discv5 events into `DiscoveredPeer` outputs, and tracks discovery metrics.
- **Key items**: `Discv5`, `DiscoveredPeer`, `Discv5::start()`, `on_discv5_update()`, `on_discovered_peer()`, `build_local_enr()`, `bootstrap()`, `spawn_populate_kbuckets_bg()`, `try_into_reachable()`, `get_fork_id()`
- **Interactions**: Consumes `Config` from `config.rs`; uses `MustNotIncludeKeys` from `filter.rs`; uses ENR conversion helpers from `enr.rs`; updates `Discv5Metrics`.
- **Knobs / invariants**: Handles `UnverifiableEnr` events by deriving a reachable `NodeRecord` from the sender socket (compatibility with peers advertising unreachable ENR sockets).

### `metrics.rs`
- **Role**: Metrics collection for discovery: tracks sessions, kbucket counts/insertions, unverifiable ENRs, and frequencies of advertised network stack IDs.
- **Key items**: `Discv5Metrics`, `DiscoveredPeersMetrics`, `AdvertisedChainMetrics`, `increment_once_by_network_type()`

### `network_stack_id.rs`
- **Role**: Defines ENR kv-pair keys used to label which network stack a node belongs to (Ethereum EL/CL, Optimism EL/CL) and selects keys from a chain spec.
- **Key items**: `NetworkStackId::{ETH, ETH2, OPEL, OPSTACK}`, `NetworkStackId::id()`

## End-to-end flow (high level)
- Build a `Config` via `Config::builder(rlpx_tcp_socket)` and set discovery listen config, boot nodes, fork kv-pair, and lookup intervals.
- `Discv5::start()` derives the local ENR + a backwards-compatible `NodeRecord` with `build_local_enr()`.
- Start underlying `discv5::Discv5`, obtain its event stream, and add boot nodes (direct ENR insert or ENR request for unsigned "enode" multiaddrs).
- Spawn background lookup tasks to populate kbuckets over time (`spawn_populate_kbuckets_bg`).
- Consume `discv5::Event`s and pass them through `on_discv5_update()` / `on_discovered_peer()`.
- For discovered peers, attempt to produce a reachable `NodeRecord`, apply `MustNotIncludeKeys` filtering, and extract an optional `ForkId`.
- Emit `DiscoveredPeer { node_record, fork_id }` to upstream networking and record metrics throughout.

## Key APIs (no snippets)
- **Types**: `Discv5`, `DiscoveredPeer`, `Config`, `ConfigBuilder`, `BootNode`
- **Functions**: `build_local_enr()`, `bootstrap()`, `spawn_populate_kbuckets_bg()`
- **Filtering**: `MustNotIncludeKeys`, `FilterOutcome`
