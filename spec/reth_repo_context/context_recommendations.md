This is a sophisticated approach to data indexing. By focusing on **Stateless Receipt Ingestion**, you are essentially building a "Light Client for History," which is the exact architectural gap currently existing between heavy Archive Nodes and centralized RPC providers.

To build the harness using **Reth as a library (Option 1)**, you need the networking stack, the wire protocol codecs, and the chain definitions, but you can discard the entire execution and storage engine.

Based on the `reth-tree.txt` and `tree-sizes.txt` provided, here is the filtered context list for your task.

---

### ✅ DEFINITELY INCLUDE (The Core Library)
*These crates provide the RLPx/devp2p stack, the message types, and the chain constants needed to talk to peers.*

**1. Networking & P2P Stack**
*   `crates/net/network`: **Highest Priority.** Contains the `NetworkManager` and `NetworkHandle`. This is your primary entry point for managing peer sessions and sending `PeerRequest`s.
*   `crates/net/eth-wire`: Contains the RLP codecs for the `eth` subprotocol. Necessary for encoding `GetReceipts`.
*   `crates/net/eth-wire-types`: The actual struct definitions for `Status`, `Receipts`, etc.
*   `crates/net/discv4` & `crates/net/discv5`: Necessary for finding peers on Ethereum and Base.
*   `crates/net/ecies` & `crates/net/p2p`: Low-level RLPx encryption and framing logic.

**2. Chain Specifications (The "Rules")**
*   `crates/chainspec`: Definitions for Ethereum Mainnet (genesis hashes, fork ids).
*   `crates/optimism/chainspec`: **Crucial for Phase 2.** Contains the Base and Optimism specific genesis and hardfork logic.

**3. Primitives & Trie (For Verification)**
*   `crates/primitives`: Core types (`Header`, `Receipt`, `Log`, `B256`).
*   `crates/primitives-traits`: Traits that abstractions in the network layer use.
*   `crates/trie/common`: Necessary logic for MPT (Merkle Patricia Trie) construction to verify `receiptsRoot`.

**4. Node Glue**
*   `crates/node/builder`: Even for a stateless harness, using the `NodeBuilder` is the easiest way to initialize a clean networking stack without manually wiring 50+ components.

---

### ❌ DEFINITELY EXCLUDE (The "Heavy" State Engine)
*These represent 80% of Reth's complexity and are entirely related to EVM execution and disk-based state storage, which you are explicitly avoiding.*

**1. Execution & EVM (Stateless = No EVM)**
*   `crates/revm`: The EVM implementation. 
*   `crates/ethereum/evm` & `crates/optimism/evm`: Glue code for executing blocks.
*   `crates/payload`: Logic for building new blocks/payloads.

**2. Storage & Database (The "State" you don't want)**
*   `crates/storage/libmdbx-rs`: The underlying C database.
*   `crates/storage/db`: Reth's abstraction over MDBX.
*   `crates/storage/provider`: The traits and implementations for reading/writing state to disk.
*   `crates/storage/codecs`: Compact serialization for database rows.
*   `crates/static-file`: While potentially useful later for caching, it's not needed for the P2P harness.

**3. Syncing & Stages**
*   `crates/stages`: This is the "Pipeline" (Headers -> Bodies -> Execution). Since you are doing a custom stateless probe, you don't want the standard Reth sync stages running.
*   `crates/etl`: Used for bulk loading data into the DB.

**4. Transaction Pool**
*   `crates/transaction-pool`: You are indexing history, not watching the mempool (gossip of unconfirmed txs).

**5. RPC Server**
*   `crates/rpc/*`: All crates in the RPC folder. You are building a *client* that consumes data, not a *server* that provides JSON-RPC to others.

---

### ⚠️ OPTIONAL / CONDITIONAL
*   `crates/era-downloader`: You mentioned using `era1` as a fallback. This crate contains the logic to parse those files. Keep it if you want to implement the backfill fallback early.
*   `crates/ress`: (Reth Stateless). This is worth reading as a reference (as it handles witnesses), but you likely won't import it directly as your goal is receipts-only indexing, not full stateless verification.
*   `examples/manual-p2p`: **Highly Recommended Reference.** This is a minimal example of how to use Reth's network stack without the rest of the node. It is very close to what your `harness_spec.md` describes.

---

### Recommended Strategy for your Harness
1.  **Dependencies:** Start by only importing `reth-network`, `reth-eth-wire`, `reth-chainspec`, and `reth-op-chainspec`.
2.  **Base Connection:** Use `reth_op_chainspec::BASE_MAINNET` to get the genesis hash.
3.  **The Probe:** Use `NetworkHandle::send_request` to target a specific `PeerId` with `PeerRequest::GetReceipts`.
4.  **Verification:** If you implement `receiptsRoot` verification, look at `reth_trie::common::root::receipts_root`.

**Summary of Filtered Context:**
Focus on `crates/net`, `crates/primitives`, and `crates/chainspec`. Discard `crates/storage`, `crates/revm`, and `crates/rpc`. This reduces your surface area from ~23M tokens to roughly ~2M tokens of relevant logic.

