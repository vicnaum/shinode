# Q033 - TransactionsManager Gossip Filtering by Version and Policy

Status: verified

## Question
How does TransactionsManager use peer capabilities / negotiated eth version to filter or shape
transaction gossip?
I want:
1) what peer metadata is stored (version, peer_kind, capabilities) and where it comes from,
2) how outgoing announcements are built per peer (Eth66 vs Eth68 formats, types/sizes),
3) how announcement filtering decides accept/ignore/reject by tx type,
4) how full transaction broadcasts avoid blob txs and fall back to hashes,
5) how incoming announcements are validated for version/metadata consistency.

## Context Packs
- `spec/reth_repo_context/context-transactions.xml`

## Gemini Prompt

```text
You are analyzing a repository packed by Repomix (attached).
Use the attached Repomix file: spec/reth_repo_context/context-transactions.xml

Question (Q033):
How does TransactionsManager use peer capabilities / negotiated eth version to filter or shape
transaction gossip? I want:
1) what peer metadata is stored (version, peer_kind, capabilities) and where it comes from,
2) how outgoing announcements are built per peer (Eth66 vs Eth68 formats, types/sizes),
3) how announcement filtering decides accept/ignore/reject by tx type,
4) how full transaction broadcasts avoid blob txs and fall back to hashes,
5) how incoming announcements are validated for version/metadata consistency.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Show the call flow from session info -> peer metadata -> propagation/filters.
3. If any claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Peer metadata & capability inputs (step-by-step)
4. Outgoing announcements by version (step-by-step)
5. Incoming announcement filtering (step-by-step)
6. Full-tx vs hash propagation rules
7. Key abstractions/types (with file paths)
8. "Where to look next"
9. Suggested follow-up questions
```

## Gemini Answer
Based on the provided `context-transactions.xml` pack, here is the detailed analysis of how `TransactionsManager` handles peer capabilities and ETH versions to filter and shape transaction gossip.

### 1. Summary
*   **Version Tracking**: The manager stores the negotiated `EthVersion` (e.g., Eth66, Eth68) in a `PeerMetadata` struct for every connected peer, initialized upon session establishment.
*   **Announcement Shaping**: Outgoing announcements use the `PooledTransactionsHashesBuilder` enum. If `Eth68` is negotiated, it includes transaction types and sizes; if `Eth66`, it sends only hashes.
*   **Blob Protection**: EIP-4844 blob transactions are explicitly excluded from full broadcasts via `is_broadcastable_in_full()`. They are demoted to hash-only announcements, forcing peers to manually request them if needed.
*   **Ingress Filtering**: Incoming announcements are filtered through an `AnnouncementFilteringPolicy`. By default, `StrictEthAnnouncementFilter` rejects unknown transaction types (e.g., future EIPs or spam) and recommends penalizing the peer.
*   **Validation**: The manager strictly enforces that incoming data matches the negotiated version (e.g., ensuring metadata exists if the version is Eth68) and penalizes peers for "bad announcements" (empty, duplicates, or invalid types).

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/network/src/transactions/mod.rs` | `TransactionsManager` | The central actor managing peer state and gossip logic. |
| `crates/net/network/src/transactions/mod.rs` | `PeerMetadata` | Stores `EthVersion` and `seen_transactions` for a specific peer. |
| `crates/net/network/src/transactions/mod.rs` | `PropagateTransactionsBuilder` | Logic that switches between Pooled (Hash) and Full (Object) propagation based on peer version. |
| `crates/net/network/src/transactions/config.rs` | `AnnouncementFilteringPolicy` | Trait defining how to accept/reject incoming tx types. |
| `crates/net/eth-wire-types/src/broadcast.rs` | `NewPooledTransactionHashes` | The wire type enum (`Eth66` vs `Eth68`) used for announcements. |

---

### 3. Peer Metadata & Capability Inputs

The `TransactionsManager` does not perform the handshake itself but receives the result via `NetworkEvent`.

1.  **Input Event**: The `NetworkManager` emits `NetworkEvent::ActivePeerSession` (or `PeerEvent::SessionEstablished`) containing `SessionInfo`.
    *   *File*: `crates/net/network-api/src/events.rs`
    *   *Type*: `SessionInfo` contains `version: EthVersion`, `capabilities`, and `peer_kind`.

2.  **Processing**: `TransactionsManager::on_network_event` calls `handle_peer_session`.
    *   *File*: `crates/net/network/src/transactions/mod.rs`
    *   *Function*: `handle_peer_session`

3.  **Storage**: A `PeerMetadata` struct is created and stored in `self.peers` (HashMap).
    *   *File*: `crates/net/network/src/transactions/mod.rs`
    *   *Struct*:
        ```rust
        pub struct PeerMetadata<N: NetworkPrimitives = EthNetworkPrimitives> {
            // ...
            version: EthVersion,
            peer_kind: PeerKind,
            // ...
        }
        ```
    *   *Note*: While `SessionInfo` has `capabilities`, `PeerMetadata` primarily extracts and stores the `EthVersion` derived from those capabilities during the handshake (which happened upstream).

---

### 4. Outgoing Announcements by Version

When propagating transactions, the manager builds version-specific messages for each peer.

1.  **Trigger**: `propagate_transactions` iterates over peers.
    *   *File*: `crates/net/network/src/transactions/mod.rs`

2.  **Builder Selection**: It initializes a builder based on `peer.version`.
    ```rust
    let mut builder = if peer_idx > max_num_full {
        PropagateTransactionsBuilder::pooled(peer.version)
    } else {
        PropagateTransactionsBuilder::full(peer.version)
    };
    ```

3.  **Hash Construction (`PooledTransactionsHashesBuilder`)**:
    *   *File*: `crates/net/network/src/transactions/mod.rs`
    *   *Function*: `PooledTransactionsHashesBuilder::push`
    *   **Eth66**: Pushes just the hash.
        ```rust
        Self::Eth66(msg) => msg.0.push(*tx.tx_hash()),
        ```
    *   **Eth68**: Pushes hash, size, and type.
        ```rust
        Self::Eth68(msg) => {
            msg.hashes.push(*tx.tx_hash());
            msg.sizes.push(tx.size);
            msg.types.push(tx.transaction.ty());
        }
        ```

---

### 5. Incoming Announcement Filtering

When a peer sends hashes (`NewPooledTransactionHashes`), the manager filters them before requesting the full bodies.

1.  **Ingress Policy**: First, `accepts_incoming_from` checks `TransactionIngressPolicy` (e.g., allow `All` or only `Trusted`).
    *   *File*: `crates/net/network/src/transactions/mod.rs`

2.  **Type/Size Filtering**: Inside `on_new_pooled_transaction_hashes`, the code iterates the received hashes using `partially_valid_msg.retain(...)`.

3.  **Policy Execution**: It calls the configured `AnnouncementFilteringPolicy`.
    *   *File*: `crates/net/network/src/transactions/mod.rs`
    *   *Call*: `self.policies.announcement_filter().decide_on_announcement(ty_byte, tx_hash, size_val)`

4.  **Strict Filter Logic**: The default `StrictEthAnnouncementFilter` (alias for `TypedStrictFilter`) checks if the type is known.
    *   *File*: `crates/net/network/src/transactions/config.rs`
    *   *Logic*:
        ```rust
        if N::PooledTransaction::is_type(ty) {
            AnnouncementAcceptance::Accept
        } else {
            // ... logs trace ...
            AnnouncementAcceptance::Reject { penalize_peer: true }
        }
        ```
    *   *Result*: If rejected with penalty, `report_peer(peer_id, ReputationChangeKind::BadAnnouncement)` is called.

---

### 6. Full Tx vs. Hash Propagation (Blob Handling)

Reth ensures EIP-4844 Blob transactions are never broadcast as full objects (`Transactions` message), only as hashes (`NewPooledTransactionHashes`).

1.  **Logic Location**: `FullTransactionsBuilder::push`
    *   *File*: `crates/net/network/src/transactions/mod.rs`

2.  **The Check**:
    ```rust
    if !transaction.transaction.is_broadcastable_in_full() {
        self.pooled.push(transaction); // Fallback to hash propagation
        return
    }
    ```

3.  **Trait Definition**: `is_broadcastable_in_full` is defined on `SignedTransaction`.
    *   *File*: `crates/primitives-traits/src/transaction/signed.rs`
    *   *Implementation*:
        ```rust
        fn is_broadcastable_in_full(&self) -> bool {
            // EIP-4844 transactions are not broadcastable in full, only hashes are allowed.
            !self.is_eip4844()
        }
        ```

4.  **Result**: If it's a blob tx, it skips `self.transactions.push(...)` and goes to `self.pooled`, which builds the `NewPooledTransactionHashes` message instead.

---

### 7. Incoming Announcement Validation

The `TransactionsManager` validates metadata consistency in `on_new_pooled_transaction_hashes`.

1.  **Version Check**:
    *   *File*: `crates/net/network/src/transactions/mod.rs`
    *   It checks `is_eth68_message`.

2.  **Metadata Consistency**:
    *   It iterates the message. If it is `Eth68` but metadata (type/size) is `None`, or if it is *not* `Eth68` but metadata is present (an implementation artifact), it sets `should_report_peer = true`.
    ```rust
    // crates/net/network/src/transactions/mod.rs
    let (ty_byte, size_val) = match *metadata_ref_mut {
        Some((ty, size)) => {
            if !is_eth68_message { should_report_peer = true; } // Metadata present but version < 68
            (ty, size)
        }
        None => {
            if is_eth68_message {
                should_report_peer = true; // Version 68 but metadata missing
                return false;
            }
            (0u8, 0)
        }
    };
    ```

3.  **Deduplication**: It calls `msg.dedup()`. If `partially_valid_msg.len() != original_len`, it penalizes the peer for spamming duplicates.

---

### 8. Key Abstractions & Types

*   **`PeerMetadata`** (`crates/net/network/src/transactions/mod.rs`): Stores `EthVersion` used to decide message formats.
*   **`NewPooledTransactionHashes66` / `68`** (`crates/net/eth-wire-types/src/broadcast.rs`): The concrete wire types. `68` includes `types: Vec<u8>` and `sizes: Vec<usize>`.
*   **`AnnouncementFilteringPolicy`** (`crates/net/network/src/transactions/config.rs`): Logic to `Accept`, `Ignore`, or `Reject` announcements based on type/size.
*   **`FullTransactionsBuilder`** (`crates/net/network/src/transactions/mod.rs`): Helper struct that enforces the "No full blobs" rule and size limits for `Transactions` messages.

---

### 9. Where to Look Next
*   **`crates/net/network/src/transactions/fetcher.rs`**: Look at `pack_request` to see how the `TransactionFetcher` uses the peer's `EthVersion` to pack `GetPooledTransactions` requests (Eth66 vs Eth68 packing strategies differ).
*   **`crates/net/eth-wire-types/src/broadcast.rs`**: To see the specific RLP encoding differences between `NewPooledTransactionHashes66` and `68`.

---

### 10. Suggested Follow-up Questions
*   "How does `TransactionFetcher` decide which peers to request from if multiple peers announce the same hash via different Eth versions?"
*   "What happens if `AnnouncementFilteringPolicy` returns `Ignore`? Is the hash removed from `seen_transactions` or just skipped for fetching?"
*   "How does the `TxTypesCounter` in `on_new_pooled_transaction_hashes` feed into metrics for network monitoring?"

## Corrections / Caveats
- `SessionInfo` includes `capabilities`, but `PeerMetadata` does not store or use them. TransactionsManager uses `version` (eth/66 vs eth/68 message shape) and `peer_kind` (ingress policy), not capability lists.
- Incoming announcement validation checks message shape (eth/66 vs eth/68 metadata presence) and spam/invalid types, but it does not explicitly compare the peer's negotiated `EthVersion` against the incoming announcement version in this layer.
- The set of valid transaction types is determined by `N::PooledTransaction::is_type`, whose concrete implementation is outside this pack.

## Verification
- `SessionInfo` carries `capabilities`, `version`, and `peer_kind` for established sessions.```54:71:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network-api/src/events.rs
pub struct SessionInfo {
    pub peer_id: PeerId,
    pub remote_addr: SocketAddr,
    pub client_version: Arc<str>,
    pub capabilities: Arc<Capabilities>,
    pub status: Arc<UnifiedStatus>,
    pub version: EthVersion,
    pub peer_kind: PeerKind,
}
```
- `handle_peer_session` stores the negotiated `version` and `peer_kind` in `PeerMetadata` (no capability field).```1183:1197:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
fn handle_peer_session(&mut self, info: SessionInfo, messages: PeerRequestSender<PeerRequest<N>>) {
    let SessionInfo { peer_id, client_version, version, .. } = info;
    let peer = PeerMetadata::<N>::new(
        messages,
        version,
        client_version,
        self.config.max_transactions_seen_by_peer_history,
        info.peer_kind,
    );
    ...
}
```
- `PeerMetadata` stores `version` and `peer_kind` used by policy/propagation decisions.```1964:1977:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
pub struct PeerMetadata<N: NetworkPrimitives = EthNetworkPrimitives> {
    seen_transactions: LruCache<TxHash>,
    request_tx: PeerRequestSender<PeerRequest<N>>,
    version: EthVersion,
    client_version: Arc<str>,
    peer_kind: PeerKind,
}
```
- Default policies use `StrictEthAnnouncementFilter`.```351:365:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
pub fn new(...) -> Self {
    Self::with_policy(
        ...,
        NetworkPolicies::new(
            TransactionPropagationKind::default(),
            StrictEthAnnouncementFilter::default(),
        ),
    )
}
```
- The strict announcement filter accepts only known tx types and penalizes unknown types.```254:272:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/config.rs
impl<N: NetworkPrimitives> AnnouncementFilteringPolicy<N> for TypedStrictFilter {
    fn decide_on_announcement(&self, ty: u8, hash: &B256, size: usize) -> AnnouncementAcceptance {
        if N::PooledTransaction::is_type(ty) {
            AnnouncementAcceptance::Accept
        } else {
            AnnouncementAcceptance::Reject { penalize_peer: true }
        }
    }
}
```
- Outgoing propagation selects full vs hash mode per peer and uses `peer.version` for message shape.```1008:1021:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
let max_num_full = self.config.propagation_mode.full_peer_count(self.peers.len());
for (peer_idx, (peer_id, peer)) in self.peers.iter_mut().enumerate() {
    if !self.policies.propagation_policy().can_propagate(peer) { continue }
    let mut builder = if peer_idx > max_num_full {
        PropagateTransactionsBuilder::pooled(peer.version)
    } else {
        PropagateTransactionsBuilder::full(peer.version)
    };
    ...
}
```
- Versioned hash announcements include types/sizes for eth/68+ and hashes only for eth/66/67.```1911:1918:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
fn push<T: SignedTransaction>(&mut self, tx: &PropagateTransaction<T>) {
    match self {
        Self::Eth66(msg) => msg.0.push(*tx.tx_hash()),
        Self::Eth68(msg) => {
            msg.hashes.push(*tx.tx_hash());
            msg.sizes.push(tx.size);
            msg.types.push(tx.transaction.ty());
        }
    }
}
```
```1922:1928:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
fn new(version: EthVersion) -> Self {
    match version {
        EthVersion::Eth66 | EthVersion::Eth67 => Self::Eth66(Default::default()),
        EthVersion::Eth68 | EthVersion::Eth69 | EthVersion::Eth70 => Self::Eth68(Default::default()),
    }
}
```
- Full transaction broadcast excludes blob transactions using `is_broadcastable_in_full`, which maps to `!is_eip4844()`.```1843:1853:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
if !transaction.transaction.is_broadcastable_in_full() {
    self.pooled.push(transaction);
    return
}
```
```51:58:/Users/vicnaum/github/stateless-history-node/reth/crates/primitives-traits/src/transaction/signed.rs
fn is_broadcastable_in_full(&self) -> bool {
    // EIP-4844 transactions are not broadcastable in full, only hashes are allowed.
    !self.is_eip4844()
}
```
- Incoming announcement checks message emptiness/duplicates and enforces eth/68 metadata presence.```620:707:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
if msg.is_empty() {
    self.report_peer(peer_id, ReputationChangeKind::BadAnnouncement);
    return;
}
let original_len = msg.len();
let mut partially_valid_msg = msg.dedup();
if partially_valid_msg.len() != original_len {
    self.report_peer(peer_id, ReputationChangeKind::BadAnnouncement);
}
...
let is_eth68_message = partially_valid_msg
    .msg_version()
    .expect("partially valid announcement should have a version")
    .is_eth68();
...
let (ty_byte, size_val) = match *metadata_ref_mut {
    Some((ty, size)) => { if !is_eth68_message { should_report_peer = true; } (ty, size) }
    None => { if is_eth68_message { should_report_peer = true; return false; } (0u8, 0) }
};
...
let decision = self.policies.announcement_filter().decide_on_announcement(ty_byte, tx_hash, size_val);
...
if should_report_peer {
    self.report_peer(peer_id, ReputationChangeKind::BadAnnouncement);
}
```
- Ingress policy gates handling of incoming transactions/announcements using `peer_kind`.```1269:1277:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/transactions/mod.rs
fn accepts_incoming_from(&self, peer_id: &PeerId) -> bool {
    if self.config.ingress_policy.allows_all() {
        return true;
    }
    let Some(peer) = self.peers.get(peer_id) else { return false; };
    self.config.ingress_policy.allows(peer.peer_kind())
}
```

## Actionable Pointers
- Versioned announcement shaping: `reth/crates/net/network/src/transactions/mod.rs`.
- Announcement policy: `reth/crates/net/network/src/transactions/config.rs`.
- Wire announcement formats: `reth/crates/net/eth-wire-types/src/broadcast.rs`.
