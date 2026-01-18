# Q014 - Missing Ranges (Holes) Representation

Status: verified

## Question
How are missing ranges/holes represented and handled during sync?

## Context Packs
- `spec/reth_repo_context/context-sync.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-sync.xml

Question (Q014):
How are missing ranges/holes represented and handled during sync?
I want the data structures, how gaps are detected, how they are persisted (if at all), and how they drive retries or stage logic.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Explain how gaps are tracked and updated over time.
3. Describe how gaps affect downloader behavior and pipeline stages.
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
Based on the `context-sync.xml` pack, here is the analysis of how missing ranges (holes) are represented and handled in Reth.

### 1. Summary
*   **Implicit Representation**: Reth does not persist explicit "gap" or "hole" data structures in the database. Instead, a "hole" is implicitly defined as the range between a stage's **Persistent Checkpoint** (local head) and the **Sync Target** (network tip).
*   **Sequential Filling**: The pipeline forces stages to run sequentially. A stage cannot process range `N..M` until the previous stage has successfully completed that range. This ensures holes only exist at the *tip* of the chain, never in the middle.
*   **Header Gap Strategy**: The `HeaderStage` identifies the gap between the local database head and the network tip using the `HeaderSyncGap` struct. It fills this gap by downloading in **reverse** (from tip down to local head) to cryptographically guarantee the chain links before committing.
*   **Transient Gaps (Network)**: During body downloads, if a peer returns a partial response (creating a transient hole in a requested batch), the `BodiesRequestFuture` detects the missing items and immediately issues a high-priority re-request to fill the gap before yielding results to the stage.
*   **Inconsistency Detection**: The `ensure_consistency` logic checks for "gaps" between the database (MDBX) and Static Files (e.g., if a write was interrupted). If the database is ahead of static files (missing static data), it forces an unwind to align them.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/stages/stages/src/stages/headers.rs` | `HeaderSyncGap` | In-memory struct representing the range `(local_head..target]` that needs to be filled. |
| `crates/net/downloaders/src/headers/reverse_headers.rs` | `ReverseHeadersDownloader` | The engine that closes the header gap by fetching backwards from the target to the local head. |
| `crates/net/downloaders/src/bodies/request.rs` | `BodiesRequestFuture` | Handles transient holes in network responses by re-requesting missing bodies within a batch. |
| `crates/stages/api/src/stage.rs` | `ExecInput` | Defines the current working range (`checkpoint..target`) for any stage. |
| `crates/stages/stages/src/stages/bodies.rs` | `ensure_consistency` | Detects and handles storage holes/inconsistencies between MDBX and Static Files. |

---

### 3. Algorithm Flow: Handling the "Big Hole" (Sync)

The primary "hole" is the sync gap. Reth handles this differently for Headers vs. Bodies.

#### A. Headers: The Reverse Fill (Closing the Gap)
1.  **Detection**: `HeaderStage::poll_execute_ready` compares the `input.checkpoint()` (Local Head) with `self.tip` (Network Tip). It creates a `HeaderSyncGap` struct.
2.  **Assignment**: The gap is passed to the `ReverseHeadersDownloader` via `update_sync_gap`.
3.  **Reverse Fetch**: The downloader requests headers starting from the **Target** moving backwards.
    *   *Why?* This ensures the new chain is valid and rooted in a trusted tip, preventing long-range attacks.
4.  **Connection Check**: When the downloader reaches the `local_head` block number, it verifies if `downloaded_header.parent_hash == local_head.hash`.
    *   **Match**: The hole is bridged. Headers are yielded to the stage.
    *   **Mismatch**: A `StageError::DetachedHead` is returned.
5.  **Resolution**: If `DetachedHead` occurs, the pipeline triggers an **Unwind**. The checkpoint is moved *backwards* (widening the hole) to find a common ancestor, effectively rejecting the local fork.

#### B. Bodies: The Forward Fill (Populating Content)
1.  **Range**: `BodyStage` calculates the range `checkpoint + 1 .. target`.
2.  **Request**: It calls `downloader.set_download_range(range)`.
3.  **Batching**: `BodiesDownloader` reads headers from the DB for that range and issues requests.
4.  **Partial Responses**:
    *   If a peer returns 50 bodies but 100 were requested (a "hole" in the batch), `BodiesRequestFuture::try_buffer_blocks` buffers the 50.
    *   It immediately calculates the remaining hashes and calls `submit_request` with `Priority::High` to fill the hole.
    *   The stage never sees the hole; it only receives the full batch once the future completes.

---

### 4. Key Abstractions & Data Types

*   **`HeaderSyncGap`** (`crates/stages/stages/src/stages/headers.rs`):
    *   Contains `local_head` (SealedHeader) and `target` (SyncTarget).
    *   Represents the "work to be done" for the header stage.
*   **`StageCheckpoint`** (`crates/stages/types/src/checkpoints.rs`):
    *   Persisted in the database.
    *   The `block_number` field marks the *end* of valid data. The "hole" implicitly starts at `block_number + 1`.
*   **`SyncTarget`** (`crates/net/network-p2p` - re-exported in `headers/mod.rs`):
    *   Enum: `Tip(B256)`, `Gap(BlockWithParent)`, or `TipNum(u64)`.
    *   Defines the upper bound of the hole.
*   **`DetachedHead`** (`crates/stages/api/src/error.rs`):
    *   Error variant indicating the "hole" could not be cleanly filled because the edges (Local Head and Downloaded Chain) do not link.

---

### 5. Where to Look Next

1.  **`crates/net/downloaders/src/headers/reverse_headers.rs`**: Look at `process_next_headers`. This is where the logic verifies if the downloaded stream connects to the `local_head`, effectively checking if the "hole" is perfectly filled.
2.  **`crates/stages/stages/src/stages/execution.rs`**: Look at `ensure_consistency`. This shows how Reth handles "holes" caused by data corruption or partial writes (e.g., Static Files having fewer blocks than the Database).
3.  **`crates/net/downloaders/src/bodies/request.rs`**: Look at `on_block_response`. This shows the retry logic for filling transient holes when a peer sends incomplete data.

---

### 6. Suggested Follow-up Questions
*   "How does the `HeaderStage` determine the `SyncTarget` if the network tip is unknown or conflicting?"
*   "What happens to the `BodyStage` if it encounters a header in the database that references a transaction not found in the network (data availability gap)?"
*   "How does `ensure_consistency` in `ExecutionStage` decide whether to unwind the database or the static files when their heights mismatch?"

## Verification
- A header "gap" is explicitly represented as `HeaderSyncGap { local_head, target }`, and the target is a `SyncTarget` (Tip, Gap, TipNum).```45:101:/Users/vicnaum/github/stateless-history-node/reth/crates/net/p2p/src/headers/downloader.rs
pub enum SyncTarget { Tip(B256), Gap(BlockWithParent), TipNum(u64) }
pub struct HeaderSyncGap<H: Sealable = Header> { pub local_head: SealedHeader<H>, pub target: SyncTarget }
```
- `HeaderStage::poll_execute_ready` builds the gap from the local head and network tip, and passes it to the downloader via `update_sync_gap`.```204:230:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/stages/headers.rs
let local_head = self.provider.local_tip_header(current_checkpoint.block_number)?;
let target = SyncTarget::Tip(*self.tip.borrow());
let gap = HeaderSyncGap { local_head, target };
...
self.downloader.update_sync_gap(gap.local_head, gap.target);
```
- Per-stage "gap" is represented by `ExecInput` checkpoint/target and `next_block_range()` which starts at `checkpoint.block_number + 1` and ends at `target`.```12:84:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/stage.rs
pub struct ExecInput { pub target: Option<BlockNumber>, pub checkpoint: Option<StageCheckpoint> }
...
let start = current_block.block_number + 1;
let target = self.target();
let end = min(target, current_block.block_number.saturating_add(threshold));
```
- Bodies re-requests missing items within a batch by resubmitting with `Priority::High` whenever pending headers remain after a response.```116:155:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/request.rs
// Buffer block responses
self.try_buffer_blocks(bodies)?;
// Submit next request if any
if let Some(req) = self.next_request() { self.submit_request(req, Priority::High); }
```
- `ensure_consistency` checks static file vs DB transaction heights and returns a missing-static-data error when static files are behind.```70:137:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/stages/bodies.rs
match next_static_file_tx_num.cmp(&next_tx_num) {
    Ordering::Greater => { ... prune_transactions ... }
    Ordering::Less => { return Err(missing_static_data_error(..., StaticFileSegment::Transactions)?) }
    Ordering::Equal => {}
}
```
- Detached head is surfaced as a `StageError::DetachedHead` when the downloader cannot attach to the local head.```30:64:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/error.rs
DetachedHead { local_head: Box<BlockWithParent>, header: Box<BlockWithParent>, error: Box<ConsensusError> }
```

## Corrections / Caveats
- `HeaderSyncGap` is defined in `reth/crates/net/p2p/src/headers/downloader.rs`, not in `headers.rs`.
- "Holes only exist at the tip" is not guaranteed; range resets, unwinds, or static file mismatches can create gaps outside the tip.
- The long-range attack rationale is not stated in code; it is an interpretation of reverse syncing.
- `ensure_consistency` referenced here is in `bodies.rs`, not `execution.rs`.
- `StageCheckpoint` is a struct used for progress tracking; persistence is handled elsewhere (e.g., pipeline checkpointing), not shown in this file set.

## Actionable Pointers
- Header gap representation: `reth/crates/net/p2p/src/headers/downloader.rs`.
- Header gap detection and downloader wiring: `reth/crates/stages/stages/src/stages/headers.rs`.
- Bodies transient gap handling: `reth/crates/net/downloaders/src/bodies/request.rs`.
- Static file consistency check: `reth/crates/stages/stages/src/stages/bodies.rs`.
