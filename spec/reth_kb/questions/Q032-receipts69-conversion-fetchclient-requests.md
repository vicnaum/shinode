# Q032 - Receipts69/70 Bloom Reconstruction + FetchClient Request Shapes

Status: verified

## Question
I want two related clarifications:
1) Receipts69 conversion: where is the missing bloom handled when converting Receipts69/Receipts70 back to the standard receipt format used elsewhere (or when exposing receipts)? Show the conversion code, types, and what data is lost or reconstructed.
2) FetchClient request shape: why does HeadersRequest support direction/skip/limit while GetBlockBodies is just a list of hashes? Show where these request types are defined and how they are used in FetchClient/StateFetcher.

## Context Packs
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-network.xml

Question (Q032):
I want two related clarifications:
1) Receipts69 conversion: where is the missing bloom handled when converting Receipts69/Receipts70 back to the standard receipt format used elsewhere (or when exposing receipts)? Show the conversion code, types, and what data is lost or reconstructed.
2) FetchClient request shape: why does HeadersRequest support direction/skip/limit while GetBlockBodies is just a list of hashes? Show where these request types are defined and how they are used in FetchClient/StateFetcher.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. For receipts, identify any helper conversion functions and their behavior.
3. For fetch requests, point to the request structs/enums and call sites in FetchClient and downloader.
4. If any part is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Receipts69 conversion (step-by-step)
4. FetchClient request shapes (step-by-step)
5. Consequences for consumers (what callers need to know)
6. Key abstractions/types (with file paths)
7. "Where to look next"
8. Suggested follow-up questions
```

## Gemini Answer
Based on the provided XML pack, here is the clarification regarding Receipts69 conversion and FetchClient request shapes.

### 1. Summary
*   **Bloom Reconstruction**: In `eth/69` and `eth/70`, the Bloom Filter is removed from the wire protocol to save bandwidth. Reth converts `Receipts69` back to standard `Receipts` using the helper `into_with_bloom`, which iterates over the logs inside the receipt and mathematically recomputes the Bloom filter.
*   **Protocol Constraints**: The difference in request shapes between Headers and Bodies is dictated by the Ethereum Wire Protocol specification, not just Reth's design.
*   **Headers**: The protocol supports "chain traversal" (by number, direction, skip) because headers contain the metadata (parent hash, number) needed to navigate the chain structure.
*   **Bodies**: The protocol requires explicit hashes for bodies because bodies do not contain enough metadata to safely navigate chains or forks; the requester is expected to have validated the header (and thus possess the hash) before asking for the body.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/eth-wire-types/src/receipts.rs` | `Receipts69::into_with_bloom` | The specific function that converts wire-optimized receipts back to full receipts by recomputing the bloom. |
| `crates/net/eth-wire-types/src/blocks.rs` | `GetBlockHeaders` | Defines the rich query fields (`skip`, `direction`, `limit`). |
| `crates/net/eth-wire-types/src/blocks.rs` | `GetBlockBodies` | Defines the simple list-of-hashes query. |
| `crates/net/network/src/fetch/client.rs` | `FetchClient` | The API surface exposing these different request strategies to the pipeline. |

---

### 3. Receipts69 Conversion (Missing Bloom)

When a peer sends `Receipts69` (or `Receipts70`), the Bloom filter is missing from the payload. Reth provides a helper to convert this back to the standard `Receipts` type (which includes `ReceiptWithBloom`).

#### The Conversion Logic
Located in **`crates/net/eth-wire-types/src/receipts.rs`**:

```rust
impl<T: TxReceipt> Receipts69<T> {
    /// Encodes all receipts with the bloom filter.
    ///
    /// Eth/69 omits bloom filters on the wire... This helper reconstructs the bloom locally 
    /// from each receipt's logs so the older API can be used on top of eth/69 data.
    ///
    /// Note: This is an expensive operation that recalculates the bloom for
    /// every receipt.
    pub fn into_with_bloom(self) -> Receipts<T> {
        Receipts(
            self.0
                .into_iter()
                .map(|receipts| receipts.into_iter().map(|r| r.into_with_bloom()).collect())
                .collect(),
        )
    }
}
```

#### What is Data is Lost/Reconstructed?
*   **Lost on Wire**: The 256-byte `LogsBloom`.
*   **Reconstructed**: The `into_with_bloom()` call delegates to the `TxReceipt` trait (likely imported from `alloy_consensus` or similar, outside this pack). This trait method iterates the logs list inside the receipt and regenerates the Bloom bits.
*   **Performance Impact**: CPU cost increases to save Network bandwidth.

#### Receipts70
`Receipts70` also uses this logic. It wraps `Receipts69` internal structure plus metadata, so it calls the same helper:

```rust
impl<T: TxReceipt> Receipts70<T> {
    pub fn into_with_bloom(self) -> Receipts<T> {
        // Reuse the eth/69 helper...
        Receipts69(self.receipts).into_with_bloom()
    }
}
```

---

### 4. FetchClient Request Shapes

The `FetchClient` mirrors the underlying `eth` protocol messages defined in `eth-wire-types`.

#### A. HeadersRequest (Complex)
The `eth` protocol allows downloading headers by traversing the chain structure (See `crates/net/eth-wire-types/src/blocks.rs`).

*   **Structure**:
    ```rust
    pub struct GetBlockHeaders {
        pub start_block: BlockHashOrNumber,
        pub limit: u64,
        pub skip: u32,
        pub direction: HeadersDirection,
    }
    ```
*   **FetchClient Usage**:
    In `crates/net/network/src/fetch/client.rs`, `HeadersClient::get_headers` accepts a `HeadersRequest` struct that exposes all these fields.
*   **StateFetcher Usage**:
    In `crates/net/network/src/fetch/mod.rs`, `prepare_block_request` maps this directly to the wire message:
    ```rust
    BlockRequest::GetBlockHeaders(GetBlockHeaders {
        start_block: start,
        limit,
        skip: 0, // Note: StateFetcher currently hardcodes skip to 0 for internal requests
        direction,
    })
    ```

#### B. GetBlockBodies (Simple)
The `eth` protocol dictates that bodies are retrieved by Hash only.

*   **Structure** (`crates/net/eth-wire-types/src/blocks.rs`):
    ```rust
    pub struct GetBlockBodies(
        /// The block hashes to request bodies for.
        pub Vec<B256>,
    );
    ```
*   **Why no Number/Limit?** Block bodies do not contain the "Parent Hash" link required to traverse a chain. If you asked for "Body #100", the peer wouldn't know *which* fork of #100 you want. Therefore, you must provide the specific Hash (which implies you already downloaded the Header).
*   **FetchClient Usage**:
    In `crates/net/network/src/fetch/client.rs`, `BodiesClient::get_block_bodies` accepts `Vec<B256>`.
*   **StateFetcher Usage**:
    In `crates/net/network/src/fetch/mod.rs`:
    ```rust
    BlockRequest::GetBlockBodies(GetBlockBodies(request))
    ```

---

### 5. Consequences for Consumers

*   **Receipt Consumers**: If you manually request `Receipts` from an `eth/69` peer using `NetworkHandle`, you will receive `Receipts69`. If your application logic expects `ReceiptWithBloom`, you **must** call `.into_with_bloom()` manually on the result.
*   **Downloader Implementers**: You cannot "scan" for bodies. You must implement a "Skeleton Sync" or "Header First" strategy where you download Headers (traversal) first, then use the hashes from those headers to download Bodies (lookup).

---

### 6. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `Receipts69` | `crates/net/eth-wire-types/src/receipts.rs` | Wrapper for `Vec<Vec<Receipt>>` (no bloom). |
| `HeadersRequest` | `crates/net/p2p/src/headers/client.rs` | High-level struct used by the client to configure traversal. |
| `DownloadRequest` | `crates/net/network/src/fetch/mod.rs` | Internal enum carrying the specific request payload to the `StateFetcher`. |
| `HeadersDirection` | `crates/net/eth-wire-types/src/header.rs` | Enum `Rising` (Ascending) or `Falling` (Descending). |

---

### 7. Where to Look Next
*   **`crates/net/eth-wire-types/src/receipts.rs`**: To see the exact implementation of `into_with_bloom`.
*   **`crates/net/network/src/fetch/mod.rs`**: Look at `prepare_block_request` to see how the high-level `HeadersRequest` is converted into the low-level `GetBlockHeaders` wire message.

---

### 8. Suggested Follow-up Questions
*   "Since `StateFetcher` hardcodes `skip: 0` in `prepare_block_request`, how would I implement a downloader that needs to skip headers (e.g. for binary search)?"
*   "Where is `TxReceipt` defined (outside this pack), and does `into_with_bloom` cache the result, or is it recomputed every time?"

## Corrections / Caveats
- `HeadersRequest` does not expose `skip`; only `GetBlockHeaders` includes `skip`, and `StateFetcher` hardcodes `skip: 0`. The FetchClient API only provides `start`, `limit`, and `direction`.
- The rationale about why the protocol uses hash-only body requests is not encoded in code; it is a protocol-spec rationale (outside the pack). The code only shows the wire shape.
- Bloom reconstruction is delegated to `TxReceipt::into_with_bloom`, which comes from `alloy_consensus` (outside the pack).

## Verification
- `Receipts` uses `ReceiptWithBloom`, while `Receipts69` omits bloom and provides `into_with_bloom`.```56:111:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire-types/src/receipts.rs
pub struct Receipts<T = Receipt>(pub Vec<Vec<ReceiptWithBloom<T>>>);
...
pub struct Receipts69<T = Receipt>(pub Vec<Vec<T>>);
impl<T: TxReceipt> Receipts69<T> {
    pub fn into_with_bloom(self) -> Receipts<T> {
        Receipts(
            self.0
                .into_iter()
                .map(|receipts| receipts.into_iter().map(|r| r.into_with_bloom()).collect())
                .collect(),
        )
    }
}
```
- `Receipts70` carries `last_block_incomplete` and `receipts`, and reuses the eth/69 helper to rebuild bloom.```120:173:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire-types/src/receipts.rs
pub struct Receipts70<T = Receipt> {
    pub last_block_incomplete: bool,
    pub receipts: Vec<Vec<T>>,
}
impl<T: TxReceipt> Receipts70<T> {
    pub fn into_with_bloom(self) -> Receipts<T> {
        Receipts69(self.receipts).into_with_bloom()
    }
}
```
- Wire request shapes: `GetBlockHeaders` includes `skip` and `direction`, while `GetBlockBodies` is a hash list.```11:66:/Users/vicnaum/github/stateless-history-node/reth/crates/net/eth-wire-types/src/blocks.rs
pub struct GetBlockHeaders {
    pub start_block: BlockHashOrNumber,
    pub limit: u64,
    pub skip: u32,
    pub direction: HeadersDirection,
}
...
pub struct GetBlockBodies(pub Vec<B256>);
```
- `HeadersRequest` (FetchClient API) includes `start`, `limit`, `direction` only.```13:23:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/headers/client.rs
pub struct HeadersRequest {
    pub start: BlockHashOrNumber,
    pub limit: u64,
    pub direction: HeadersDirection,
}
```
- `FetchClient` accepts `HeadersRequest` and `Vec<B256>` for bodies.```58:99:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/fetch/client.rs
fn get_headers_with_priority(&self, request: HeadersRequest, priority: Priority) -> Self::Output { ... }
...
fn get_block_bodies_with_priority_and_range_hint(&self, request: Vec<B256>, ... ) -> Self::Output { ... }
```
- `StateFetcher::prepare_block_request` hardcodes `skip: 0` when converting to `GetBlockHeaders`.```242:252:/Users/vicnaum/github/stateless-history-node/reth/crates/net/network/src/fetch/mod.rs
let HeadersRequest { start, limit, direction } = request;
BlockRequest::GetBlockHeaders(GetBlockHeaders {
    start_block: start,
    limit,
    skip: 0,
    direction,
})
```
- Bodies requests remain hash-only in the client API as well.```24:48:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/bodies/client.rs
fn get_block_bodies(&self, hashes: Vec<B256>) -> Self::Output { ... }
fn get_block_bodies_with_priority_and_range_hint(&self, hashes: Vec<B256>, ...) -> Self::Output;
```

## Actionable Pointers
- Bloom reconstruction: `reth/crates/net/eth-wire-types/src/receipts.rs`.
- Request shape mapping: `reth/crates/net/network/src/fetch/mod.rs`.
- FetchClient API surface: `reth/crates/net/network/src/fetch/client.rs`.
