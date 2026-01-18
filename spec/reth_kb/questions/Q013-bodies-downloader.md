# Q013 - Bodies Downloader Batching & Retries

Status: verified

## Question
How does the bodies downloader handle batching, retries, and missing bodies?

## Context Packs
- `spec/reth_repo_context/context-sync.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-sync.xml

Question (Q013):
How does the bodies downloader handle batching, retries, and missing bodies?
I want the request strategy, how bodies are queued/validated, how errors are handled, and how results are delivered to the Bodies stage.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe the request/response flow and how batches are formed.
3. Explain how errors, timeouts, or missing bodies are handled.
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
Based on the `context-sync.xml` pack, here is the architecture of the **Bodies Downloader**.

### 1. Summary
*   **Concurrent Batching**: The downloader fetches bodies in multiple concurrent batches. It reads headers from the database and groups them into requests sent to different peers.
*   **Self-Contained Request Futures**: A single `BodiesRequestFuture` is responsible for fetching a specific batch of bodies. It manages its own retries and handles partial responses internally without alerting the main downloader loop unless a fatal error occurs.
*   **Priority Escalation**: If a request fails or yields a partial response, the future re-requests the missing data with `Priority::High` to ensure fast completion.
*   **Internal Buffering**: Responses from peers arrive out of order. The downloader buffers them in a `BinaryHeap` to ensure they are yielded to the pipeline in strict block number order.
*   **Pre-Execution Validation**: Bodies are validated against their headers (e.g., transaction root, ommers hash) immediately upon download, before being passed to the stage.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/net/downloaders/src/bodies/bodies.rs` | `BodiesDownloader` | The main orchestrator struct. It implements `Stream` and manages the queue of requests and the buffer of responses. |
| `crates/net/downloaders/src/bodies/request.rs` | `BodiesRequestFuture` | A Future representing a *single* batch request. It handles the logic for retrying and filling gaps if a peer sends a partial response. |
| `crates/net/downloaders/src/bodies/queue.rs` | `BodiesRequestQueue` | A wrapper around `FuturesUnordered` that manages the collection of active `BodiesRequestFuture` tasks. |

---

### 3. Algorithm Flow

#### Step 1: Request Generation (`next_headers_request`)
The `BodiesDownloader` calculates the next range of blocks to fetch.
*   It queries the database for headers via `query_headers`.
*   It groups headers into a batch based on `request_limit` (default 200) and `stream_batch_size`.
*   **File**: `crates/net/downloaders/src/bodies/bodies.rs`

#### Step 2: Dispatch (`submit_request`)
The batch of headers is wrapped in a `BodiesRequestFuture` and pushed to the `in_progress_queue`.
*   The future initiates a P2P request via the `BodiesClient` (network interface).
*   **File**: `crates/net/downloaders/src/bodies/bodies.rs`

#### Step 3: Response & Partial Handling (`BodiesRequestFuture`)
Inside the `BodiesRequestFuture`, the network response is polled.
*   **Partial Responses**: If a peer returns fewer bodies than requested, `try_buffer_blocks` saves the valid ones. The future then calculates the remaining missing headers (`next_request`) and issues a *new* network request for the remainder immediately, staying in `Poll::Pending` state.
*   **Retries**: If a request fails (timeout, empty response), `on_error` is called. This penalizes the peer and resubmits the request with `Priority::High`.
*   **File**: `crates/net/downloaders/src/bodies/request.rs`

#### Step 4: Validation (`try_buffer_blocks`)
As bodies arrive, the future validates them against the headers using `consensus.validate_block_pre_execution`.
*   Checks calculated transaction root vs header root.
*   Checks ommers hash vs header ommers hash.
*   If invalid, the peer is penalized, and the specific header is put back into the pending queue for retry.
*   **File**: `crates/net/downloaders/src/bodies/request.rs`

#### Step 5: Ordering & Buffering (`poll_next`)
When a `BodiesRequestFuture` finally completes (has all bodies for its batch):
*   It returns `Vec<BlockResponse<B>>` to the main downloader.
*   The downloader pushes this into `buffered_responses` (a `BinaryHeap`).
*   `try_next_buffered` checks if the heap contains the *next expected block number*. If so, it moves the data to `queued_bodies`.
*   **File**: `crates/net/downloaders/src/bodies/bodies.rs`

#### Step 6: Yielding to Stage
The stream yields batches from `queued_bodies` to the `BodyStage` for database insertion.

---

### 4. Key Abstractions/Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `BodiesRequestFuture` | `crates/net/downloaders/src/bodies/request.rs` | The "worker" that refuses to die until it has fetched every body in its assigned batch of headers. |
| `OrderedBodiesResponse` | `crates/net/downloaders/src/bodies/bodies.rs` | A wrapper around a vector of bodies that implements `Ord` (reverse block number) for the `BinaryHeap`. |
| `BlockResponse<B>` | `crates/net/network-p2p/src/bodies/response.rs` (Ref) | Enum: `Full(SealedBlock)` or `Empty(SealedHeader)`. Allows handling empty blocks (no txs/ommers) gracefully. |
| `Priority` | `crates/net/network-p2p/src/priority.rs` (Ref) | `Normal` for initial requests, `High` for retries to ensure the pipeline doesn't stall on a single missing block. |

---

### 5. Where to Look Next
1.  **`crates/net/downloaders/src/bodies/request.rs`**: Specifically the `poll` method and `on_block_response`. This contains the complex logic for handling "soft limits" (receiving 2MB of data but not all requested blocks) and retrying partials.
2.  **`crates/net/downloaders/src/bodies/bodies.rs`**: The `poll_next` method demonstrates how the downloader balances submitting new requests vs. processing buffered responses to manage memory pressure (`can_submit_new_request`).

---

### 6. Suggested Follow-up Questions
*   "How does the `BodiesRequestFuture` determine if a response is 'empty' versus 'partial' regarding the Soft Response Limit?"
*   "What happens to the `in_progress_queue` if the `BodiesDownloader` receives a `clear()` command during a pipeline unwind?"
*   "How does `consensus.validate_block_pre_execution` differ from the full execution validation done in the Execution stage?"

## Verification
- Bodies downloader batches requests from DB headers and limits non-empty headers per request.```31:132:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/bodies.rs
/// Downloads bodies in batches.
/// All blocks in a batch are fetched at the same time.
...
fn next_headers_request(&self) -> DownloadResult<Option<Vec<SealedHeader<Provider::Header>>>> { ... }
...
fn query_headers(..., max_non_empty: u64) -> DownloadResult<Option<Vec<SealedHeader<B::Header>>>> { ... }
```
- Default request limit is 200 non-empty headers per request, configurable via `BodiesDownloaderBuilder`.```508:543:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/bodies.rs
pub struct BodiesDownloaderBuilder { ... pub request_limit: u64, ... }
impl Default for BodiesDownloaderBuilder {
    fn default() -> Self {
        Self { request_limit: 200, stream_batch_size: 1_000, ... }
    }
}
```
- Requests are queued as `BodiesRequestFuture` instances in a `FuturesUnordered` wrapper.```19:89:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/queue.rs
pub(crate) struct BodiesRequestQueue { inner: FuturesUnordered<BodiesRequestFuture<...>>, ... }
...
pub(crate) fn push_new_request(...) { ... BodiesRequestFuture::new(...).with_headers(request) }
```
- `BodiesRequestFuture` re-issues requests until pending headers are exhausted, and retries with `Priority::High` after errors/partial responses.```20:155:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/request.rs
If the response arrived with insufficient number of bodies, the future will issue another request until all bodies are collected.
...
if let Some(req) = self.next_request() { self.submit_request(req, Priority::High); }
...
fn on_error(...) { ... self.submit_request(self.next_request().expect(...), Priority::High); }
```
- Pre-execution body validation happens in `try_buffer_blocks` via `validate_block_pre_execution`.```159:199:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/request.rs
let block = SealedBlock::from_sealed_parts(next_header, next_body);
if let Err(error) = self.consensus.validate_block_pre_execution(&block) { ... }
```
- Out-of-order responses are buffered in a `BinaryHeap` and drained in order via `try_next_buffered`.```64:252:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/bodies.rs
buffered_responses: BinaryHeap<OrderedBodiesResponse<B>>,
...
fn try_next_buffered(&mut self) -> Option<Vec<BlockResponse<B>>> { ... }
```
- Downloader yields batches to the stage, and `BodyStage` stores them via `append_block_bodies`.```369:441:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/bodies.rs
fn poll_next(...) -> Poll<Option<Self::Item>> { ... }
```
```159:206:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/stages/bodies.rs
self.downloader.set_download_range(input.next_block_range())?;
...
provider.append_block_bodies(...)
```
- `BlockResponse` is `Full(SealedBlock)` or `Empty(SealedHeader)`.```4:11:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/bodies/response.rs
pub enum BlockResponse<B: Block> {
    Full(SealedBlock<B>),
    Empty(SealedHeader<B::Header>),
}
```

## Corrections / Caveats
- Peer selection is inside `BodiesClient`; the downloader does not guarantee "different peers."
- "Strict block number order" is enforced by buffering + `try_next_buffered`, but ordering is based on response ranges and expected next block, not a full global sort.
- The internal details of `validate_block_pre_execution` (tx root/ommers checks) are outside this pack.
- "Soft response limit (2MB)" appears only in a TODO comment, not in enforcement logic.

## Actionable Pointers
- Downloader orchestration: `reth/crates/net/downloaders/src/bodies/bodies.rs`.
- Per-request logic, retries, validation: `reth/crates/net/downloaders/src/bodies/request.rs`.
- Stage integration: `reth/crates/stages/stages/src/stages/bodies.rs`.
