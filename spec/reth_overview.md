This technical specification provides a detailed mapping of the **Reth (v1.10.x)** repository for building the "Receipt-Availability Harness." It focuses on using Reth as a modular library to handle the complexities of RLPx, ECIES, and the `eth` wire protocol.

---

### 1. Cargo Workspace & Dependency Mapping

To build this harness, your `Cargo.toml` should import these specific crates from the Reth workspace. Using Reth as a library requires pinning to a specific version/commit.

```toml
[dependencies]
# Core Networking Logic
reth-network = { path = "../../reth/crates/net/network" }
reth-network-api = { path = "../../reth/crates/net/network-api" }
reth-network-peers = { path = "../../reth/crates/net/peers" }

# Wire Protocol & Handshakes
reth-eth-wire = { path = "../../reth/crates/net/eth-wire" }
reth-eth-wire-types = { path = "../../reth/crates/net/eth-wire-types" }

# Chain Configurations
reth-chainspec = { path = "../../reth/crates/chainspec" }
reth-optimism-chainspec = { path = "../../reth/crates/optimism/chainspec" }

# Primitives & Storage Traits (No DB required)
reth-ethereum-primitives = { path = "../../reth/crates/ethereum/primitives" }
reth-storage-api = { path = "../../reth/crates/storage/storage-api" }
reth-primitives-traits = { path = "../../reth/crates/primitives-traits" }

# Async / Utilities
tokio = { version = "1.37", features = ["full"] }
futures = "0.3"
tracing = "0.1"
```

---

### 2. Implementation Guide: The Reth API Surface

#### A. Initializing the Network (Mainnet vs. Base)
You must configure the `NetworkManager` differently depending on the target chain. Use `NoopProvider` to satisfy Reth's requirement for a data source without actually having a database.

*   **Imports:**
    *   `reth_network::{NetworkConfig, NetworkManager, NetworkConfigBuilder}`
    *   `reth_storage_api::noop::NoopProvider`
    *   `reth_chainspec::{MAINNET, SEPOLIA}`
    *   `reth_optimism_chainspec::{BASE_MAINNET, OP_MAINNET}`
*   **Key Functions:**
    *   `NetworkConfigBuilder::new(secret_key)`: Generates the base config.
    *   `.mainnet_boot_nodes()`: Automatically populates L1 bootnodes.
    *   `.build(client)`: Where `client` is `NoopProvider::eth(Arc::clone(&MAINNET))`.

#### B. Listening for Peers
Once the network starts, you need to react when a peer completes the `eth` handshake (Status exchange).

*   **Location:** `crates/net/network-api/src/events.rs`
*   **Struct:** `NetworkEvent`
*   **Usage:**
    ```rust
    let mut event_listener = network_handle.event_listener();
    while let Some(event) = event_listener.next().await {
        match event {
            NetworkEvent::ActivePeerSession { info, messages } => {
                // 'info' contains PeerId, client_version, and 'status'
                // 'messages' is a PeerRequestSender used to send probes
                tokio::spawn(run_probe_sequence(info, messages));
            }
            _ => {}
        }
    }
    ```

#### C. Sending Probes (`GetBlockHeaders` & `GetReceipts`)
You will use the `PeerRequestSender` provided by the event listener to dispatch messages directly to a specific peer.

*   **Location:** `crates/net/eth-wire-types/src/blocks.rs` and `src/receipts.rs`
*   **Request Types:**
    *   `GetBlockHeaders`: Fields `start_block`, `limit: 1`, `direction: Falling`.
    *   `GetReceipts`: Field `Vec<B256>` (the hash obtained from the header).
*   **Key Function:**
    *   `messages.try_send(PeerRequest::GetBlockHeaders { request, response })`
    *   `response` is a `oneshot::channel()` that yields `RequestResult<BlockHeaders>`.

#### D. Identifying "Head" Block Number
Because the `Status` message only provides the `blockhash`, you must resolve the number of that hash before you can calculate "Head - N" offsets.

*   **Logic:**
    1. Connect to Peer.
    2. Extract `info.status.blockhash`.
    3. Send `GetBlockHeaders` with `start_block: BlockHashOrNumber::Hash(blockhash)`.
    4. Resulting `BlockHeader` contains the `u64` block number.

---

### 3. Detailed Component Spec for the Developer

| Requirement | Reth Crate | Module/File Path | Key Type/Trait |
| :--- | :--- | :--- | :--- |
| **P2P Identity** | `reth-network` | `src/config.rs` | `rng_secret_key()` |
| **Handshake Logic** | `reth-eth_wire` | `src/handshake.rs` | `EthRlpxHandshake` |
| **Status Message** | `reth-eth-wire-types` | `src/status.rs` | `UnifiedStatus` (superset of all versions) |
| **Message IDs** | `reth-eth-wire-types` | `src/message.rs` | `EthMessageID` (0x0f for Receipts) |
| **Peer Info** | `reth-network-api` | `src/lib.rs` | `PeerInfo` |
| **Discovery** | `reth-network` | `src/discovery.rs` | `Discovery` (handles discv4/v5/DNS) |
| **Base Config** | `reth-optimism-chainspec` | `src/base.rs` | `BASE_MAINNET` (Static Arc) |
| **L1 Config** | `reth-chainspec` | `src/spec.rs` | `MAINNET` (Static Arc) |
| **Receipt Structure** | `reth-ethereum-primitives` | `src/receipt.rs` | `Receipt` |

---

### 4. Harness Execution Workflow (The "Script")

1.  **Orchestration:**
    *   Start `NetworkManager`.
    *   Obtain a `NetworkHandle`.
    *   Subscribe to `NetworkEvent` stream.
2.  **On Peer Discovery:**
    *   Peer finishes RLPx (`Hello`) and ETH (`Status`).
    *   Check `info.status.genesis` against `MAINNET.genesis_hash()` (or Base).
    *   Discard mismatched peers immediately.
3.  **Probing:**
    *   **Header Probe:** Send `GetBlockHeaders` for number `N`. Wait for `Header`.
    *   **Receipt Probe:** If Header exists, send `GetReceipts` for `Header.hash`.
    *   **Metric:** Wrap these calls in a timer to measure RTT.
4.  **Reporting:**
    *   Format `PeerInfo` + `RequestResult` into a flat JSON structure.
    *   Log to stdout/file.

### 5. Why this familiarizes you with Reth
By building this, the developer will learn:
1.  **The Swarm/Session Boundary:** How Reth separates low-level TCP/ECIES from high-level "Active Sessions."
2.  **Wire Type Versioning:** How Reth handles `eth/66` vs `eth/68` differences (important for `NewPooledTransactionHashes`).
3.  **The Primitive Abstraction:** How Reth uses traits to allow `reth-network` to work for both Ethereum and Optimism without duplicate code.
4.  **ChainSpecs:** How hardforks and genesis data are hardcoded into the binary for extreme performance and reliability.

### 6. Critical Base Mainnet Constant
If the developer needs the Base genesis hash for `Option 2` manual checking, it is defined in:
`crates/optimism/chainspec/src/base.rs` as:
`b256!("0xf712aa9241cc24369b143cf6dce85f0902a9731e70d66818a3a5845b296c73dd")`
and its Network ID is `8453`.

---

This is a deep-dive technical mapping of the **Reth (v1.10.x)** repository specifically for building a stateless history harness. I have structured this to guide a developer through the exact modules, traits, and logic flows required to bypass the full node requirements while using Reth’s P2P stack.

---

### I. Identity & Cryptography
**Goal:** Generate the node's identity and handle ECIES/RLPx handshakes.

*   **Crate:** `reth-network`
*   **File:** `crates/net/network/src/config.rs`
    *   **Function:** `rng_secret_key()`
    *   **Usage:** Use this to generate a `secp256k1::SecretKey`. This is the "Identity" of your probe.
*   **Crate:** `reth-network-peers`
*   **File:** `crates/net/peers/src/lib.rs`
    *   **Function:** `pk2id(&public_key)`
    *   **Usage:** Converts the `secp256k1::PublicKey` into a `PeerId` (B512), which is how you identify yourself and peers in logs.

---

### II. The "Stateless" Provider (The Mock Layer)
**Goal:** Reth's `NetworkManager` requires a "Client" that can read the blockchain. Since you have no database, you must provide a "Noop" implementation that behaves as if the chain is empty but knows the `ChainSpec`.

*   **Crate:** `reth-storage-api`
*   **File:** `crates/net/network-api/src/noop.rs`
    *   **Struct:** `NoopProvider<ChainSpec>`
    *   **Usage:**
        ```rust
        // For Ethereum
        let client = NoopProvider::eth(reth_chainspec::MAINNET.clone());
        // For Base
        let client = NoopProvider::eth(reth_optimism_chainspec::BASE_MAINNET.clone());
        ```
*   **Why this matters:** This satisfies the trait bounds for `BlockNumReader` and `HeaderProvider` required by the networking layer without actually initializing MDBX or static files.

---

### III. Chain Configurations & Network IDs
**Goal:** Differentiate between Ethereum and Base at the P2P level.

*   **Ethereum Mainnet:**
    *   **Crate:** `reth-chainspec`
    *   **File:** `crates/chainspec/src/spec.rs`
    *   **Static:** `MAINNET`
    *   **Network ID:** `1`
*   **Base Mainnet (OP Stack):**
    *   **Crate:** `reth-optimism-chainspec`
    *   **File:** `crates/optimism/chainspec/src/base.rs`
    *   **Static:** `BASE_MAINNET`
    *   **Genesis Hash:** `0xf712aa9241cc24369b143cf6dce85f0902a9731e70d66818a3a5845b296c73dd`
    *   **Network ID:** `8453`
*   **Crate:** `reth-discv5`
*   **File:** `crates/net/discv5/src/network_stack_id.rs`
    *   **Constant:** `NetworkStackId::ETH` (for L1) and `NetworkStackId::OPEL` (for Base).
    *   **Note:** Base uses the "Optimism Execution Layer" (`opel`) identifier in its ENR.

---

### IV. Configuring the Network Stack
**Goal:** Setup the `NetworkManager` to only do discovery and session management.

*   **Crate:** `reth-network`
*   **File:** `crates/net/network/src/config.rs`
    *   **Struct:** `NetworkConfigBuilder`
    *   **Implementation Steps:**
        1.  Initialize with `NetworkConfigBuilder::new(secret_key)`.
        2.  Set `set_addrs(SocketAddr)` to listen on `0.0.0.0:30303`.
        3.  Call `.mainnet_boot_nodes()` for Ethereum or `.boot_nodes(base_nodes())` for Base.
        4.  **Crucial:** Call `.disable_tx_gossip(true)` and `.block_import(Box::new(ProofOfStakeBlockImport))` (found in `crates/net/network/src/import.rs`). This prevents the node from trying to process/verify blocks.

---

### V. The Orchestrator: `NetworkManager` & `Swarm`
**Goal:** Understand how Reth handles the peer lifecycle.

*   **File:** `crates/net/network/src/manager.rs`
    *   **Struct:** `NetworkManager<N: NetworkPrimitives>`
    *   **Generic `N`:** For Ethereum, use `EthNetworkPrimitives`. For Base, you should still use `EthNetworkPrimitives` as the wire-level serialization is identical for receipts.
*   **File:** `crates/net/network/src/swarm.rs`
    *   **Logic:** The `Swarm` is the internal state machine. It manages the `SessionManager` (RLPx) and `Discovery` (discv4/v5).
    *   **Interaction:** You don't interact with the Swarm directly; you poll the `NetworkManager`.

---

### VI. Executing the Probes (The Developer's Task)
**Goal:** Send `GetBlockHeaders` and `GetReceipts` to specific peers.

*   **Crate:** `reth-network-api`
*   **File:** `crates/net/network-api/src/events.rs`
    *   **Event:** `NetworkEvent::ActivePeerSession { info, messages }`
    *   **Flow:**
        1.  Wait for this event. `info` (Type `SessionInfo`) tells you the peer's client version (e.g., `reth/v1.0.0`) and their reported head.
        2.  `messages` (Type `PeerRequestSender`) is your "gun."
*   **Crate:** `reth-eth-wire-types`
*   **File:** `crates/net/eth-wire-types/src/blocks.rs`
    *   **Struct:** `GetBlockHeaders`
    *   **Usage:**
        ```rust
        let req = GetBlockHeaders {
            start_block: BlockHashOrNumber::Number(12345),
            limit: 1,
            skip: 0,
            direction: HeadersDirection::Rising,
        };
        let (tx, rx) = oneshot::channel();
        messages.try_send(PeerRequest::GetBlockHeaders { request: req, response: tx });
        ```
*   **File:** `crates/net/eth-wire-types/src/receipts.rs`
    *   **Struct:** `GetReceipts`
    *   **Usage:** Take the `hash` from the header returned above and send `GetReceipts(vec![hash])`.

---

### VII. Correctness: Receipt Verification (Optional)
**Goal:** Verify that the `receiptsRoot` in the header matches the receipts fetched.

*   **Crate:** `reth-primitives-traits`
*   **File:** `crates/primitives-traits/src/proofs.rs`
    *   **Function:** `calculate_receipt_root(receipts)`
    *   **Logic:** This function takes a slice of receipts (which include the `logsBloom`) and recomputes the MPT root.
*   **Crate:** `reth-ethereum-primitives`
*   **File:** `crates/ethereum/primitives/src/receipt.rs`
    *   **Struct:** `Receipt`
    *   **Detail:** This crate contains the RLP definitions for Ethereum receipts. Note the `tx_type` field; Reth handles Legacy (0), EIP-2930 (1), EIP-1559 (2), and EIP-4844 (3) automatically.

---

### VIII. Summary for the Developer

1.  **Main Entry:** Create a `tokio` task that runs `NetworkManager::new(config).await`.
2.  **Handle:** Use `manager.handle()` to get a `NetworkHandle`.
3.  **Events:** Call `handle.event_listener()` and loop over `NetworkEvent`.
4.  **Filters:** Inside the loop, when `ActivePeerSession` triggers, check `info.status.genesis`. If it doesn't match your target (ETH vs Base), ignore the peer.
5.  **Probing:** Use the `messages` channel to send a `GetBlockHeaders` for your target number. On success, use the hash to send a `GetReceipts`.
6.  **Stats:** Wrap the `oneshot::Receiver` await in an `Instant::now()` timer. Log the `peer_id`, `client_version`, `block_number`, and `Duration`.

### Repository "Navigational Map" for this Project:
*   `crates/net/eth-wire-types`: **The Dictionary.** (What do the messages look like?)
*   `crates/net/network`: **The Engine.** (How do I stay connected?)
*   `crates/net/network-api`: **The Controls.** (How do I tell the engine to send a message?)
*   `crates/optimism/chainspec`: **The L2 Blueprint.** (What is Base's genesis hash?)
*   `crates/ethereum/primitives`: **The Data.** (How do I decode a Receipt?)