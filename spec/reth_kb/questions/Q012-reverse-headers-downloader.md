# Q012 - ReverseHeadersDownloader Algorithm

Status: verified

## Question
How does the ReverseHeadersDownloader work?

## Context Packs
- `spec/reth_repo_context/context-sync.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-sync.xml

Question (Q012):
How does the ReverseHeadersDownloader work?
I want the algorithm: starting point selection, range requests, validation strategy, retry/backoff, and how results feed into the Headers stage.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe the request/response flow and how ranges are chosen.
3. Explain how errors or invalid headers are handled (retry, ban, or unwind).
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
Based on the code in `crates/net/downloaders/src/headers/reverse_headers.rs` and `crates/stages/stages/src/stages/headers.rs`, here is the architectural breakdown of the **ReverseHeadersDownloader**.

### 1. Summary
*   **Reverse Direction**: Downloads headers from the Chain Tip *down* to the Local Head. This strategy secures the chain by anchoring to a trusted tip and verifying lineage backwards, preventing long-range attacks.
*   **Concurrent Fetching**: Issues multiple `GetBlockHeaders` requests in parallel to different peers to fill the gap between the tip and local head.
*   **Response Buffering**: Responses arriving out of order are stored in a binary heap (`buffered_responses`) and processed only when they connect to the currently validated high-block.
*   **Strict Lineage Validation**: Every downloaded header is checked to ensure its `parent_hash` matches the hash of the next header in the chain (descending).
*   **Detached Head Detection**: If the downloaded chain reaches the `local_head` block number but the hashes don't match, it returns a `DetachedHead` error, triggering a pipeline unwind.
*   **Stream Output**: It acts as a Rust `Stream`, yielding batches of validated `SealedHeader`s to the `HeaderStage`.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/net/downloaders/src/headers/reverse_headers.rs` | `ReverseHeadersDownloader` | The core struct managing state, requests, and buffering. |
| `crates/net/downloaders/src/headers/reverse_headers.rs` | `Stream::poll_next` | The execution driver. Polled by the `HeaderStage`. orchestrates request submission and response processing. |
| `crates/net/downloaders/src/headers/reverse_headers.rs` | `update_local_head` | Sets the lower bound (where we are currently). |
| `crates/net/downloaders/src/headers/reverse_headers.rs` | `update_sync_target` | Sets the upper bound (where we want to go). |

---

### 3. Algorithm Flow

#### Step 1: Boundary Initialization
The `HeaderStage` sets the boundaries before polling.
*   **Lower Bound**: `update_local_head` sets the block we currently have on disk.
*   **Upper Bound**: `update_sync_target` sets the target (Hash or Number).
    *   *Refinement*: If the target is a **Hash**, the downloader first issues a single request for that hash to resolve its **Block Number**. Only once the number is known does the parallel download begin (`on_sync_target_outcome`).

#### Step 2: Request Calculation (`next_request`)
The downloader calculates the next range to fetch.
*   It tracks `next_request_block_number` (initially the Tip).
*   It calculates a **Falling** request: `start = next_request_block_number`, `limit = request_limit`.
*   It subtracts `limit` from `next_request_block_number` for the next iteration.
*   **File**: `crates/net/downloaders/src/headers/reverse_headers.rs` -> `calc_next_request`.

#### Step 3: Concurrent Dispatch (`poll_next`)
Inside the `poll_next` loop:
*   It keeps the `in_progress_queue` full up to `concurrent_request_limit`.
*   Requests are sent via the `HeadersClient` (usually `FetchClient` in `reth-network`).
*   **Priority**: Standard requests use `Priority::Normal`; Retries use `Priority::High`.

#### Step 4: Response Handling (`on_headers_outcome`)
When a peer responds:
1.  **Validation**: Checks if the response length matches the request and if the starting block number matches.
2.  **Buffering**: If the response block number is higher than the `next_chain_tip_block_number` (the next one we need to validate), it pushes the response into `buffered_responses` (a Binary Heap).
3.  **Processing**: If the response matches `next_chain_tip_block_number`, it calls `process_next_headers`.

#### Step 5: Lineage Validation (`process_next_headers`)
This is the critical security step.
1.  **Internal Consistency**: Iterates through the batch ensuring `header[i].parent_hash == header[i+1].hash`.
2.  **Connection**: Ensures the highest block in this batch connects to the lowest block of the previous batch (`queued_validated_headers`).
3.  **Local Attachment**: If the batch reaches the `local_head` number:
    *   It checks if `batch_last.parent_hash == local_head.hash`.
    *   If **Mismatch**: Returns `HeadersDownloaderError::DetachedHead`. This signals that the local chain is on a different fork than the network tip.

#### Step 6: Output to Stage
Validated headers are pushed to `queued_validated_headers`.
*   When `queued_validated_headers` reaches `stream_batch_size`, `poll_next` returns `Poll::Ready(Some(batch))`.
*   The `HeaderStage` receives these, writes them to the ETL collector, and eventually commits them to the database.

---

### 4. Error Handling & Retry Logic

*   **Peer Penalization**: If a peer returns a "Likely Bad" response (empty when not expected, or invalid lineage), `penalize_peer` is called, which reports to the `PeersHandle` to lower reputation.
*   **Retry**: `on_headers_error` re-submits the exact same request to the `in_progress_queue` with `Priority::High` to ensure it gets picked up by a different peer quickly.
*   **Channel Closure**: If the network channel closes, the stream terminates (`Poll::Ready(None)`).
*   **Detached Head**: This is a fatal error for the *stage run*, but recoverable for the *node*. It causes the pipeline to unwind the local database to a common ancestor before retrying.

---

### 5. Key Abstractions/Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `HeadersRequest` | `crates/net/network-p2p/src/headers/client.rs` (via import) | Defines a request: `start` (Hash/Num), `limit`, and `direction` (Falling). |
| `SyncTargetBlock` | `crates/net/downloaders/src/headers/reverse_headers.rs` | Enum (`Hash`, `Number`, `HashAndNumber`) tracking what we are trying to sync to. |
| `OrderedHeadersResponse` | `crates/net/downloaders/src/headers/reverse_headers.rs` | Wrapper for responses stored in the `BinaryHeap` buffer, ordered by block number. |
| `DetachedHead` | `crates/net/network-p2p/src/headers/error.rs` (via import) | The specific error returned when the download stream cannot connect to the local database head. |

---

### 6. Where to Look Next

1.  **`crates/net/downloaders/src/headers/reverse_headers.rs`**: Specifically the `process_next_headers` function. This contains the logic for validating the parent hash chain and detecting detached heads.
2.  **`crates/stages/stages/src/stages/headers.rs`**: Look at `poll_execute_ready`. This is where the stage consumes the downloader stream and handles the `DetachedHead` error by preparing an unwind.
3.  **`crates/net/network/src/fetch/client.rs`**: To see how the `HeadersClient` trait is implemented and how it actually sends the message over the P2P network. (Located in `crates/net/network/src/fetch/client.rs` in the full repo, or referenced in `fetch/mod.rs` in this pack).

---

### 7. Suggested Follow-up Questions
*   "How does the `HeaderStage` handle the `DetachedHead` error returned by the downloader to trigger an unwind?"
*   "How does the `ReverseHeadersDownloader` determine the `request_limit` dynamically based on peer count?"
*   "What happens to the `buffered_responses` heap if the `sync_target` changes mid-download?"

## Verification
- Reverse headers download direction and batch output via `Stream::poll_next`.```63:66:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
/// This downloader downloads in reverse ... falling block numbers.
```
```778:900:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
impl Stream for ReverseHeadersDownloader<H> {
    fn poll_next(...) -> Poll<Option<Self::Item>> { ... }
    ...
    if this.queued_validated_headers.len() >= this.stream_batch_size { ... return Poll::Ready(Some(Ok(next_batch))) }
}
```
- Sync target by hash is resolved via a one-header request, then updates target number.```705:737:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
SyncTarget::Tip(tip) => {
    ...
    self.sync_target_request = Some(self.request_fut(self.get_sync_target_request(tip.into()), Priority::High));
}
```
```374:437:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
let header = headers.swap_remove(0);
let target = SealedHeader::seal_slow(header);
...
let parent_block_number = target.number().saturating_sub(1);
self.on_block_number_update(target.number(), parent_block_number);
self.queued_validated_headers.push(target);
```
- Request range uses `calc_next_request` with falling direction and `limit = min(request_limit, next_request_block_number - local_head)`.```174:183:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
let request = calc_next_request(local_head, self.next_request_block_number, self.request_limit);
self.next_request_block_number -= request.limit;
```
```1235:1251:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
let diff = next_request_block_number - local_head;
let limit = diff.min(request_limit);
HeadersRequest::falling(start.into(), limit)
```
- Concurrency is controlled by `concurrent_request_limit`, derived from peer count.```153:166:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
let num_peers = self.client.num_connected_peers();
let dynamic_target = num_peers * REQUESTS_PER_PEER_MULTIPLIER;
...
max_dynamic.min(self.max_concurrent_requests)
```
- Buffered responses stored in a `BinaryHeap` and validated in order against `next_chain_tip_block_number`.```100:102:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
buffered_responses: BinaryHeap<OrderedHeadersResponse<H::Header>>,
```
```567:579:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
let next_response = this.buffered_responses.peek_mut()?;
match next_block_number.cmp(&self.next_chain_tip_block_number) {
    Ordering::Equal => { ... self.process_next_headers(...) }
    Ordering::Greater => { PeekMut::pop(next_response); }
}
```
- Lineage validation uses `validate_header_download` for each parent/child pair; detached head detected when the last header cannot attach to local head.```261:273:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
if let Err(error) = self.validate(validated_header, &parent) { ... }
```
```281:324:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
if let Some((last_header, head)) = validated.last_mut().zip(self.local_head.as_ref()).filter(|(last, head)| last.number() == head.number() + 1) {
    if let Err(error) = self.consensus.validate_header_against_parent(&*last_header, head) {
        self.reset();
        return Err(HeadersDownloaderError::DetachedHead { ... }.into())
    }
}
```
- Retry on bad responses: penalize peer and re-submit with `Priority::High`.```539:560:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
self.client.report_bad_message(peer_id);
...
self.submit_request(request, Priority::High);
```
- HeaderStage consumes the downloader stream and maps `DetachedHead` to `StageError::DetachedHead`.```192:266:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/stages/headers.rs
match ready!(self.downloader.poll_next_unpin(cx)) {
    Some(Err(HeadersDownloaderError::DetachedHead { local_head, header, error })) => {
        return Poll::Ready(Err(StageError::DetachedHead { ... }))
    }
    ...
}
```

## Corrections / Caveats
- The "long-range attack prevention" rationale is not stated in code; it's an interpretation of reverse syncing.
- The downloader issues concurrent requests, but peer selection is internal to `HeadersClient`; "different peers" is not guaranteed here.
- Detached head is detected when the last downloaded header should attach to `local_head + 1` but fails `validate_header_against_parent`, not by directly comparing hashes at the same block number.
- `EitherWriter`/pipeline unwind behavior is outside this file; the stage only surfaces `StageError::DetachedHead`.

## Actionable Pointers
- Downloader algorithm: `reth/crates/net/downloaders/src/headers/reverse_headers.rs`.
- Header stage integration: `reth/crates/stages/stages/src/stages/headers.rs`.
- Request types/errors: `reth/crates/net/network-p2p/src/headers/client.rs` and `reth/crates/net/network-p2p/src/headers/error.rs` (via imports).
