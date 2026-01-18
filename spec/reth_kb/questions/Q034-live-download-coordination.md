# Q034 - Live Download Coordination (Engine `download.rs`)

Status: verified

## Question
Live download coordination: in engine `download.rs`, how does it decide what to fetch, when, and from which peers?

## Context Packs
- `spec/reth_repo_context/context-engine.xml`
- `spec/reth_repo_context/context-network.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML packs:
- spec/reth_repo_context/context-engine.xml
- spec/reth_repo_context/context-network.xml

Question (Q034):
Live download coordination: in engine `download.rs`, how does it decide what to fetch, when, and from which peers?

Requirements:
1. Trace the call flow from the engine (tree/orchestrator) into `DownloadAction` / `DownloadRequest` and into the downloader.
2. Explain how `BasicBlockDownloader` handles block sets vs ranges, inflight management, buffering/deduping, and emitted events.
3. Identify how peer selection happens for block requests (BlockClient/FullBlockClient) and where peer choice lives.
4. Include file paths + function/type names for each claim.
5. If peer selection or triggers are outside these packs, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Triggering downloads (step-by-step)
4. `download.rs` mechanics (step-by-step)
5. Peer selection path (step-by-step)
6. Error handling / retries / bad-response handling
7. Key abstractions/types (with file paths)
8. "Where to look next"
9. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-engine.xml` and `context-network.xml` packs, here is the architecture for how the engine triggers and coordinates live block downloads.

### 1. Summary
*   **Triggered by Consensus Gaps**: Downloads are triggered by the `EngineApiTreeHandler` when `on_forkchoice_updated` or `on_new_payload` detects a missing parent or a gap in the canonical chain.
*   **Action Propagation**: The engine emits `EngineApiEvent::Download`, which bubbles up to the `EngineHandler`, which then calls `downloader.on_action(DownloadAction::Download(req))`.
*   **Split Request Strategy**: `BasicBlockDownloader` handles two types of requests: `BlockSet` (specific hashes, e.g., missing parents) and `BlockRange` (filling gaps).
*   **Parallel Fetching**: It uses `FullBlockClient` to fetch headers and bodies in parallel.
*   **In-flight Tracking**: The downloader tracks active futures (`inflight_full_block_requests`, `inflight_block_range_requests`) and buffers results in a min-heap (`set_buffered_blocks`) to yield them in order.
*   **Peer Selection Delegation**: The `BlockDownloader` does **not** select peers directly. It delegates to `FullBlockClient`, which delegates to `FetchClient`, which sends a `DownloadRequest` to the `StateFetcher`. The `StateFetcher` is the component that actually selects the "best" idle peer.
*   **Buffering & Deduping**: Downloaded blocks are stored in a `BinaryHeap` to ensure ordering. The poll loop deduplicates blocks before yielding them back to the engine.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/engine/tree/src/download.rs` | `BasicBlockDownloader` | The main struct managing active downloads and buffering results. |
| `crates/engine/tree/src/tree/mod.rs` | `EngineApiTreeHandler::handle_missing_block` | Determines that a block is missing and constructs a `DownloadRequest`. |
| `crates/engine/tree/src/engine.rs` | `EngineHandler::poll` | The loop that receives `EngineApiEvent`s from the tree and dispatches actions to the downloader. |
| `crates/net/network_p2p/src/full_block.rs` | `FullBlockClient` | Helper that combines `HeadersClient` and `BodiesClient` to fetch a full sealed block. |
| `crates/net/network/src/fetch/mod.rs` | `StateFetcher::next_best_peer` | The logic that actually picks which peer to request data from. |

---

### 3. Triggering Downloads (Step-by-Step)

#### Step 1: Gap Detection (Engine Tree)
The consensus layer sends a Forkchoice Update (FCU) or New Payload.
*   **Context**: `EngineApiTreeHandler` in `crates/engine/tree/src/tree/mod.rs`.
*   **Logic**:
    *   **FCU**: If the head block is missing, `handle_missing_block` is called. It calculates the ancestor and emits `TreeEvent::Download(DownloadRequest::single_block(target))`.
    *   **New Payload**: If `insert_block` returns `BlockStatus::Disconnected`, `on_disconnected_downloaded_block` is called. It calculates distance and emits `TreeEvent::Download(DownloadRequest::BlockRange(missing_parent, distance))`.

#### Step 2: Event Propagation
The event bubbles up the stack.
*   `EngineApiTreeHandler` emits `EngineApiEvent::Download`.
*   `EngineApiRequestHandler` receives this and passes it to `EngineHandler`.
*   `EngineHandler::poll` (`crates/engine/tree/src/engine.rs`) matches `RequestHandlerEvent::Download(req)` and calls `self.downloader.on_action(DownloadAction::Download(req))`.

---

### 4. `download.rs` Mechanics (Downloader Logic)

The logic resides in `crates/engine/tree/src/download.rs`.

#### Step 1: Receiving the Action
`BasicBlockDownloader::on_action` routes the request.
*   `BlockSet(hashes)` -> calls `download_full_block` for each hash.
*   `BlockRange(hash, count)` -> calls `download_block_range`.

#### Step 2: Spawning Futures
*   **Full Block**: Calls `full_block_client.get_full_block(hash)`. Pushes the resulting future to `inflight_full_block_requests`.
*   **Range**: Calls `full_block_client.get_full_block_range(hash, count)`. Pushes the future to `inflight_block_range_requests`.
*   *Note*: It immediately emits a `DownloadOutcome::NewDownloadStarted` event to notify listeners.

#### Step 3: Polling & Buffering
In `BasicBlockDownloader::poll`:
1.  It iterates `inflight_full_block_requests` and `inflight_block_range_requests`.
2.  When a future resolves, the resulting `SealedBlock`s are pushed into `set_buffered_blocks` (a `BinaryHeap<Reverse<OrderedSealedBlock>>`).
    *   *Why Heap?* To ensure blocks are processed in ascending block number order, regardless of which request finished first.

#### Step 4: Yielding Results
If the buffer is not empty:
1.  It pops the smallest block number.
2.  It peeks to remove duplicates (if multiple requests fetched the same block).
3.  It returns `Poll::Ready(DownloadOutcome::Blocks(vec![block]))`.

---

### 5. Peer Selection Path (The "How" & "Where")

The `BasicBlockDownloader` does **not** know about peers. It relies on the `BlockClient` trait (implemented by `FetchClient`).

1.  **Downloader**: Calls `full_block_client.get_full_block(hash)`.
2.  **FullBlockClient** (`crates/net/p2p/src/full_block.rs`): Calls `client.get_header(hash)` and `client.get_block_body(hash)`.
3.  **FetchClient** (`crates/net/network/src/fetch/client.rs`): Sends a `DownloadRequest` to the `StateFetcher` via a channel.
4.  **StateFetcher** (`crates/net/network/src/fetch/mod.rs`): This is where peer selection happens.
    *   **Logic**: `poll_action` calls `next_best_peer`.
    *   **Selection Criteria**:
        1.  Must be `Idle`.
        2.  Prioritizes peers who have *not* recently sent bad responses (`last_response_likely_bad`).
        3.  Checks `is_better` (prefers peers that cover the requested block range).
        4.  Tie-breaker: Lowest latency (`timeout`).

---

### 6. Error Handling & Retries

*   **Validation Failure**: If `FetchFullBlockFuture` (`crates/net/p2p/src/full_block.rs`) detects that the body doesn't match the header, it calls `client.report_bad_message(peer_id)` and **retries** fetching the body.
*   **Network Failure**: If the `FetchClient` request fails (e.g., channel closed), the future resolves to an error. The `BasicBlockDownloader` simply drops the future. It does **not** contain explicit retry loops for failed requests in this code; reliance is placed on the engine re-issuing the download request (e.g., via repeated FCUs or sync cycles).
*   **Peer Penalization**: Occurs in `StateFetcher` via `report_bad_message` calls.

---

### 7. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `BasicBlockDownloader` | `crates/engine/tree/src/download.rs` | Orchestrates parallel downloads and buffers results. |
| `DownloadRequest` | `crates/engine/tree/src/engine.rs` | Enum: `BlockSet(HashSet<B256>)` or `BlockRange(B256, u64)`. |
| `FullBlockClient` | `crates/net/p2p/src/full_block.rs` | Fetches headers and bodies, validates consistency, and returns `SealedBlock`. |
| `FetchFullBlockFuture` | `crates/net/p2p/src/full_block.rs` | The specific future that fetches header+body. Handles the retry loop if the body is invalid. |
| `StateFetcher` | `crates/net/network/src/fetch/mod.rs` | The component responsible for picking which peer to send the request to. |

---

### 8. Where to Look Next
1.  **`crates/net/network/src/fetch/mod.rs`**: Read `next_best_peer`. This is the exact logic for how Reth decides "Peer A is better than Peer B".
2.  **`crates/net/p2p/src/full_block.rs`**: Read `FetchFullBlockFuture::poll`. This shows how it fetches a header, then the body, and validates them against each other.
3.  **`crates/engine/tree/src/tree/mod.rs`**: Look at `on_disconnected_downloaded_block` to see how the engine decides to download a parent vs triggering a full backfill pipeline.

---

### 9. Suggested Follow-up Questions
*   "How does `StateFetcher` handle the case where all peers are busy (`NoPeersAvailable`)? Does the request queue up or fail?"
*   "What happens to the `inflight_block_range_requests` if a reorg occurs mid-download?"
*   "How does `BasicBlockDownloader` limit memory usage if `set_buffered_blocks` grows too large due to out-of-order responses?"
```

## Corrections / Caveats
- `on_new_payload` returns `PayloadStatus::Syncing` for missing parents but does not emit a download event; download triggers here come from `handle_missing_block`, `on_disconnected_downloaded_block`, and post‑backfill gap filling.
- `BasicBlockDownloader` returns **all** buffered unique blocks in one `DownloadOutcome::Blocks` (not just a single block).
- Single-block downloads request header and body concurrently; range downloads are sequential (headers first, then bodies).
- Peer selection logic is not in `download.rs`; it lives in the `BlockClient` implementation. For Reth’s network, this is `FetchClient` → `StateFetcher::next_best_peer` (from `context-network.xml`).

## Verification
- Missing head on FCU triggers `handle_missing_block` → `TreeEvent::Download(DownloadRequest::single_block(...))`.```1212:1244:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn handle_missing_block(...) -> ProviderResult<TreeOutcome<OnForkChoiceUpdated>> {
    ...
    Ok(TreeOutcome::new(OnForkChoiceUpdated::valid(PayloadStatus::from_status(
        PayloadStatusEnum::Syncing,
    )))
    .with_event(TreeEvent::Download(DownloadRequest::single_block(target))))
}
```
- Engine emits `EngineApiEvent::Download` when handling `TreeEvent::Download`.```1705:1720:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_tree_event(&mut self, event: TreeEvent) -> ProviderResult<()> {
    match event {
        ...
        TreeEvent::Download(action) => {
            self.emit_event(EngineApiEvent::Download(action));
        }
    }
    Ok(())
}
```
- `EngineHandler` forwards download requests to the downloader.```82:104:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/engine.rs
match ev {
    RequestHandlerEvent::Download(req) => {
        self.downloader.on_action(DownloadAction::Download(req));
    }
    ...
}
```
- Download requests are `BlockSet` or `BlockRange`.```340:353:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/engine.rs
pub enum DownloadRequest {
    BlockSet(HashSet<B256>),
    BlockRange(B256, u64),
}
```
- Disconnected downloaded blocks trigger follow‑up `DownloadRequest::BlockRange` or `single_block`.```2383:2418:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_disconnected_downloaded_block(...) -> Option<TreeEvent> {
    let request = if let Some(distance) = self.distance_from_local_tip(head.number, missing_parent.number) {
        DownloadRequest::BlockRange(missing_parent.hash, distance)
    } else {
        DownloadRequest::single_block(missing_parent.hash)
    };
    Some(TreeEvent::Download(request))
}
```
- Backfill completion can emit a `DownloadRequest::BlockRange` to reach FCU target.```1662:1676:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
if let Some(distance) = self.distance_from_local_tip(current_head_num, target_head_num) {
    self.emit_event(EngineApiEvent::Download(DownloadRequest::BlockRange(
        lowest_buffered.parent_hash(),
        distance,
    )));
    return Ok(());
}
```
- `BasicBlockDownloader` handles `BlockSet` vs `BlockRange`, tracks inflight futures, buffers blocks in a min‑heap, and emits `DownloadOutcome::NewDownloadStarted`.```103:166:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/download.rs
fn download(&mut self, request: DownloadRequest) {
    match request {
        DownloadRequest::BlockSet(hashes) => self.download_block_set(hashes),
        DownloadRequest::BlockRange(hash, count) => self.download_block_range(hash, count),
    }
}
...
fn download_full_block(&mut self, hash: B256) -> bool {
    if self.is_inflight_request(hash) { return false }
    self.push_pending_event(DownloadOutcome::NewDownloadStarted { remaining_blocks: 1, target: hash });
    let request = self.full_block_client.get_full_block(hash);
    self.inflight_full_block_requests.push(request);
    ...
}
```
- `BasicBlockDownloader::poll` drains inflight requests, buffers blocks in `BinaryHeap`, dedupes by hash, and returns all buffered blocks.```212:256:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/download.rs
for idx in (0..self.inflight_full_block_requests.len()).rev() { ... }
for idx in (0..self.inflight_block_range_requests.len()).rev() { ... }
...
while let Some(block) = self.set_buffered_blocks.pop() {
    while let Some(peek) = self.set_buffered_blocks.peek_mut() {
        if peek.0 .0.hash() == block.0 .0.hash() {
            PeekMut::pop(peek);
        } else { break }
    }
    downloaded_blocks.push(block.0.into());
}
Poll::Ready(DownloadOutcome::Blocks(downloaded_blocks))
```
- Single‑block fetch requests header and body concurrently; range fetch requests headers first, then bodies.```60:78:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/full_block.rs
pub fn get_full_block(&self, hash: B256) -> FetchFullBlockFuture<Client> {
    FetchFullBlockFuture {
        request: FullBlockRequest {
            header: Some(client.get_header(hash.into())),
            body: Some(client.get_block_body(hash)),
        },
        ...
    }
}
```
```90:108:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/full_block.rs
pub fn get_full_block_range(&self, hash: B256, count: u64) -> FetchFullBlockRangeFuture<Client> {
    FetchFullBlockRangeFuture {
        request: FullBlockRangeRequest {
            headers: Some(client.get_headers(HeadersRequest::falling(hash.into(), count))),
            bodies: None,
        },
        ...
    }
}
```
- Range fetch retries invalid header/body responses and reports bad peers.```516:542:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/full_block.rs
if this.headers.is_none() {
    this.request.headers = Some(this.client.get_headers(HeadersRequest {
        start: this.start_hash.into(),
        limit: this.count,
        direction: HeadersDirection::Falling,
    }));
}
```
```409:444:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/full_block.rs
if let Err(err) = self.consensus.validate_body_against_header(resp.data(), header) {
    self.client.report_bad_message(resp.peer_id());
    self.pending_headers.push_back(header.clone());
    ...
    self.request.bodies = Some(self.client.get_block_bodies(hashes));
    return None
}
```

## Actionable Pointers
- Download triggers and events: `reth/crates/engine/tree/src/tree/mod.rs`.
- Downloader internals: `reth/crates/engine/tree/src/download.rs`.
- Peer selection is downstream in `FetchClient`/`StateFetcher`: `reth/crates/net/network/src/fetch/mod.rs`.
