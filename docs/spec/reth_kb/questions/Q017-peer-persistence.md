# Q017 - Peer Persistence (Disk Format + Lifecycle)

Status: verified

## Question
How does peer persistence work (disk format + lifecycle)?

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q017):
How does peer persistence work (disk format + lifecycle)?
I want how peer records are stored, loaded on startup, updated during runtime, and any pruning/expiration behavior.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe the read/write flow and when persistence is triggered.
3. Explain how persisted peers interact with discovery and the in-memory peer set.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Algorithm flow (step-by-step)
4. Key abstractions/types (with file paths)
5. "Where to look next"
6. Suggested follow-up questions
```

## Gemini Answer
Based on the provided `context-network.xml` pack, here is the architecture of peer persistence in Reth.

### 1. Summary
*   **Simple JSON Storage**: Peer records are persisted as a JSON list of `NodeRecord`s. There is no complex database for peer history in this layer; it relies on a flat file (usually named `known_peers.json` in the data dir, though the filename is passed in by the caller).
*   **Manual Triggering**: The `network` crate does **not** contain an internal timer loop to auto-save peers. Instead, it exposes a `write_peers_to_file` method on `NetworkManager`, expecting the node application (the binary) to trigger persistence (e.g., on shutdown or periodically).
*   **Load-at-Startup**: Peers are loaded during configuration via `PeersConfig::with_basic_nodes_from_file`. These peers are loaded into memory as `PeerKind::Basic`.
*   **Discovery Integration**: While Discovery v4/v5 have their own internal routing tables (K-Buckets), Reth persists the higher-level "Known Peers" list from `PeersManager`, which aggregates nodes found via discovery, trusted peers, and static peers.
*   **No Explicit File Pruning**: The persistence logic simply dumps the current in-memory state of `PeersManager`. Any pruning or cleanup must happen in memory before the dump occurs.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/manager.rs` | `NetworkManager::write_peers_to_file` | The primary function to dump all currently known peers to disk as JSON. |
| `crates/net/network-types/src/peers/config.rs` | `PeersConfig::with_basic_nodes_from_file` | The configuration hook used at startup to read the JSON file and populate the initial peer set. |
| `crates/net/network/src/peers.rs` | `PeersManager::iter_peers` | The iterator used by the export function to collect all `NodeRecord`s for persistence. |
| `crates/net/network-peers/src/lib.rs` | `NodeRecord` | The serializable struct representing a peer (ID, IP, TCP/UDP ports) stored in the file. (Deduced from usage). |

---

### 3. Peer Persistence Lifecycle

#### Step 1: Loading at Startup
When the node starts, it builds the `NetworkConfig`.
1.  The caller (node binary) calls `PeersConfig::with_basic_nodes_from_file(path)`.
2.  **File Read**: It opens the file using `std::fs::File::open`.
3.  **Deserialization**: It uses `serde_json::from_reader` to parse a `HashSet<NodeRecord>`.
4.  **Memory Population**: These records are added to the `basic_nodes` field of `PeersConfig`.
5.  **Manager Init**: When `PeersManager::new` is called, it iterates `basic_nodes` and inserts them into the `peers` HashMap with `PeerKind::Basic`.

*   **Cite**: `crates/net/network-types/src/peers/config.rs` (`with_basic_nodes_from_file`).
*   **Cite**: `crates/net/network/src/peers.rs` (`PeersManager::new`).

#### Step 2: Runtime Updates (In-Memory)
During runtime, the peer list changes, but these changes are **not** immediately written to disk.
1.  **Discovery**: When `Discovery` finds a new node (`DiscoveryEvent::NewNode`), `NetworkManager` calls `peers_mut().add_peer(...)`.
2.  **Connections**: Incoming connections create new ephemeral peer entries in `PeersManager`.
3.  **Pruning**: `PeersManager` removes peers if they are explicitly banned or if an active session disconnects and the peer was marked `remove_after_disconnect` (common for incoming connections without a known discovery record).

*   **Cite**: `crates/net/network/src/manager.rs` (`on_discovery_event`).

#### Step 3: Persisting to Disk
Persistence is an explicit action, not a background background background task within this crate.
1.  **Trigger**: The node application calls `network_manager.write_peers_to_file(&path)`.
2.  **Collection**: `NetworkManager` delegates to `self.all_peers()`, which calls `self.swarm.state().peers().iter_peers()`.
3.  **Serialization**: The collected `Vec<NodeRecord>` is serialized to JSON.
4.  **Write**: `reth_fs_util::write_json_file` writes the data to `persistent_peers_file`.

*   **Cite**: `crates/net/network/src/manager.rs` (`write_peers_to_file`).

---

### 4. Key Abstractions & Types

*   **`NodeRecord`** (re-exported in `crates/net/network/src/lib.rs`):
    The data structure serialized to disk. Contains:
    *   `Address` (IP)
    *   `tcp_port`
    *   `udp_port`
    *   `id` (PeerId/Public Key)
*   **`PeersConfig`** (`crates/net/network-types/src/peers/config.rs`):
    Holds the configuration for loading peers, including the `basic_nodes` set populated from disk.
*   **`PeersManager`** (`crates/net/network/src/peers.rs`):
    The authoritative source of truth for "Known Peers". Only peers in this struct's `peers` HashMap are persisted. Note that `Discv4` and `Discv5` have their own internal routing tables (`KBucketsTable`), but Reth chooses to persist the application-level `PeersManager` list instead of raw K-Buckets.

---

### 5. Where to Look Next
*   **`crates/net/network/src/manager.rs`**: Look at `write_peers_to_file` to see exactly how the export is orchestrated.
*   **`crates/net/network-types/src/peers/config.rs`**: Look at `with_basic_nodes_from_file` to see the loading logic and error handling (e.g., ignoring `NotFound` errors).
*   **`crates/net/network/src/peers.rs`**: Look at `iter_peers` to see precisely which data from the internal `Peer` struct is converted into the exportable `NodeRecord`.

---

### 6. Suggested Follow-up Questions
*   "Since `PeersManager` does not seem to have a maximum size limit in this code, does the `known_peers.json` file grow indefinitely if the node runs for months?"
*   "How does `Discv4` persist its specific Kademlia table logic? Or is the K-bucket structure lost on restart, re-bootstrapping purely from the flat list in `PeersManager`?"
*   "Where is `write_peers_to_file` actually called? (Likely in `reth-node` shutdown hooks, outside this pack)."

## Verification
- `NetworkManager::write_peers_to_file` collects `all_peers()` and writes JSON to the provided path.```418:441:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/manager.rs
pub fn all_peers(&self) -> impl Iterator<Item = NodeRecord> + '_ {
    self.swarm.state().peers().iter_peers()
}
pub fn write_peers_to_file(&self, persistent_peers_file: &Path) -> Result<(), FsPathError> {
    let known_peers = self.all_peers().collect::<Vec<_>>();
    persistent_peers_file.parent().map(fs::create_dir_all).transpose()?;
    reth_fs_util::write_json_file(persistent_peers_file, &known_peers)?;
    Ok(())
}
```
- `PeersConfig::with_basic_nodes_from_file` loads a JSON file into `HashSet<NodeRecord>` and ignores `NotFound`.```295:308:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-types/src/peers/config.rs
pub fn with_basic_nodes_from_file(self, optional_file: Option<impl AsRef<Path>>) -> Result<Self, io::Error> {
    let reader = match std::fs::File::open(file_path.as_ref()) { ... };
    let nodes: HashSet<NodeRecord> = serde_json::from_reader(reader)?;
    Ok(self.with_basic_nodes(nodes))
}
```
- Default peer list path in the node is `known-peers.json` under the chain data dir.```335:340:/Users/vicnaum/github/stateless-history-node/reth/crates/node/core/src/dirs.rs
/// `<DIR>/<CHAIN_ID>/known-peers.json`
pub fn known_peers(&self) -> PathBuf { self.data_dir().join("known-peers.json") }
```
- `NodeRecord` is serialized via display/parse helpers (enode URL format), not a JSON object.```24:35:/Users/vicnaum/github/stateless-history-node/reth/crates/net/peers/src/node_record.rs
#[derive(... SerializeDisplay, DeserializeFromStr, ...)]
pub struct NodeRecord { ... }
```
```122:143:/Users/vicnaum/github/stateless-history-node/reth/crates/net/peers/src/node_record.rs
impl fmt::Display for NodeRecord { ... f.write_str("enode://")?; ... }
```
```163:198:/Users/vicnaum/github/stateless-history-node/reth/crates/net/peers/src/node_record.rs
impl FromStr for NodeRecord { ... }
```
- `PeersManager::new` seeds peers from `basic_nodes` and `trusted_nodes`.```121:142:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
for trusted_peer in &trusted_nodes { ... Peer::trusted(...) }
for NodeRecord { address, tcp_port, udp_port, id } in basic_nodes { peers.entry(id).or_insert_with(|| Peer::new(...)) }
```
- Discovery updates feed into the peer set via state actions.```325:333:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/state.rs
DiscoveryEvent::NewNode(...) => { self.queued_messages.push_back(StateAction::DiscoveredNode { ... }) }
```
```248:255:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/swarm.rs
StateAction::DiscoveredNode { peer_id, addr, fork_id } => {
    self.state_mut().peers_mut().add_peer(peer_id, addr, fork_id);
}
```
- Incoming-only peers can be marked for removal on disconnect, and are removed when sessions close.```372:378:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
let mut peer = Peer::with_state(PeerAddr::from_tcp(addr), PeerConnectionState::In);
peer.remove_after_disconnect = true;
```
```540:549:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/peers.rs
if entry.get().remove_after_disconnect && !entry.get().is_trusted() {
    entry.remove();
    self.queued_actions.push_back(PeerAction::PeerRemoved(peer_id));
}
```
- Persistence is triggered on graceful shutdown in the node builder, not by a timer inside `NetworkManager`.```907:924:/Users/vicnaum/github/stateless-history-node/reth/crates/node/builder/src/builder/mod.rs
if let Some(peers_file) = known_peers_file {
    ... network.write_peers_to_file(peers_file.as_path()) ...
}
```

## Corrections / Caveats
- The persisted file is `known-peers.json` (dash), not `known_peers.json`, and the path is configurable.
- The JSON format is an array of stringified `enode://...` URLs, not a JSON object per peer.
- The "Discovery K-bucket persistence" claim is outside this pack; only `PeersManager` persistence is shown here.
- `NodeRecord` is defined in `reth/crates/net/peers/src/node_record.rs`, not `network-peers/src/lib.rs`.

## Actionable Pointers
- Persistence entrypoint: `reth/crates/net/network/src/manager.rs` (`write_peers_to_file`).
- Startup loading: `reth/crates/net/network-types/src/peers/config.rs` (`with_basic_nodes_from_file`).
- Default file path and CLI hooks: `reth/crates/node/core/src/dirs.rs`, `reth/crates/node/core/src/args/network.rs`.
- Peer set updates: `reth/crates/net/network/src/state.rs`, `reth/crates/net/network/src/peers.rs`.
