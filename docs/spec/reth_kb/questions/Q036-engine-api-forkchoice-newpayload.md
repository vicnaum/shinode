# Q036 - Engine API Handling (forkchoiceUpdated / newPayload)

Status: verified

## Question
Engine API handling: how do forkchoiceUpdated and newPayload drive tree updates and sync?
I want:
1) the decision flow inside `on_forkchoice_updated` (canonical head, reorg, missing head),
2) how `on_new_payload` validates, inserts, and returns Syncing/Valid/Invalid,
3) how invalid ancestors are tracked and how that affects responses,
4) where and when `DownloadRequest` or `BackfillAction` is emitted from these paths.

## Context Packs
- `spec/reth_repo_context/context-engine.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-engine.xml

Question (Q036):
Engine API handling: how do forkchoiceUpdated and newPayload drive tree updates and sync?
I want:
1) the decision flow inside `on_forkchoice_updated` (canonical head, reorg, missing head),
2) how `on_new_payload` validates, inserts, and returns Syncing/Valid/Invalid,
3) how invalid ancestors are tracked and how that affects responses,
4) where and when `DownloadRequest` or `BackfillAction` is emitted from these paths.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Highlight the key branches for Syncing vs Valid vs Invalid.
3. If a detail is outside this pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. ForkchoiceUpdated path (step-by-step)
4. NewPayload path (step-by-step)
5. Invalid ancestor tracking (step-by-step)
6. Download/backfill triggers (step-by-step)
7. Key abstractions/types (with file paths)
8. "Where to look next"
9. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-engine.xml` pack, here is the architectural analysis of how `forkchoiceUpdated` and `newPayload` drive tree updates and synchronization in Reth.

### 1. Summary
*   **Central Brain**: The `EngineApiTreeHandler` running in a dedicated thread is the decision-maker. It processes incoming CL requests sequentially to maintain consistency.
*   **Forkchoice Strategy**: `forkchoiceUpdated` attempts to make the requested head canonical immediately if it exists in memory. If missing, it triggers a `DownloadRequest` (Live Sync) or `BackfillAction` (Pipeline) depending on the distance from the local tip.
*   **Payload Strategy**: `newPayload` executes the block immediately. If the parent is missing (`Disconnected`), the block is buffered, and a download for the missing parent is triggered.
*   **Sync Mode Interlock**: If the node is in `BackfillSyncState::Active` (running the pipeline), incoming payloads are buffered (`SYNCING`), and FCU updates are skipped or queue the backfill target, preventing concurrent database writes.
*   **Invalidity Propagation**: Invalid blocks are cached in `InvalidHeaderCache`. Any future payload building on an invalid ancestor is immediately rejected without execution.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/engine/tree/src/tree/mod.rs` | `EngineApiTreeHandler` | The actor processing all Engine API requests. |
| `crates/engine/tree/src/tree/mod.rs` | `on_forkchoice_updated` | Handler for switching the canonical head and updating safe/finalized points. |
| `crates/engine/tree/src/tree/mod.rs` | `on_new_payload` | Handler for validating and executing new blocks provided by the CL. |
| `crates/engine/tree/src/tree/mod.rs` | `handle_missing_block` | Fallback logic when FCU requests a block we don't have. |
| `crates/engine/tree/src/tree/invalid_headers.rs` | `InvalidHeaderCache` | Stores hashes of blocks deemed invalid to reject descendants fast. |

---

### 3. ForkchoiceUpdated Flow (`on_forkchoice_updated`)

Located in `crates/engine/tree/src/tree/mod.rs`.

#### Step 1: Pre-Validation
Calls `validate_forkchoice_state`.
*   If `head_block_hash` is zero -> Returns `INVALID`.
*   If `backfill_sync_state` is Active -> Returns `SYNCING` (paused).
*   If the head connects to a known invalid ancestor -> Returns `INVALID`.

#### Step 2: Canonical Check (`handle_canonical_head`)
Checks if `state.head_block_hash` is already the canonical head.
*   **If Yes**: Updates safe/finalized blocks (`ensure_consistent_forkchoice_state`) and processes payload attributes (if any). Returns `VALID`.

#### Step 3: Chain Update / Reorg (`apply_chain_update`)
If the block exists in `TreeState` (in-memory) or DB:
*   Calls `on_new_head(state.head_block_hash)`.
*   This calculates the path from the current head to the new head.
*   **Commit**: Simple extension.
*   **Reorg**: Finds common ancestor, unwinds old chain, applies new chain.
*   Calls `on_canonical_chain_update` to apply changes.
*   Returns `VALID`.

#### Step 4: Missing Block (`handle_missing_block`)
If the block is not found in memory or DB:
*   It calculates the sync target (usually `head_block_hash`, or `safe_block_hash` for initial sync).
*   Emits `TreeEvent::Download(DownloadRequest::single_block(target))`.
*   Returns `SYNCING`.

---

### 4. NewPayload Flow (`on_new_payload`)

Located in `crates/engine/tree/src/tree/mod.rs`.

#### Step 1: Ancestor Check
Calls `find_invalid_ancestor`. If the parent (or further up) is in `InvalidHeaderCache`, returns `INVALID` immediately without execution.

#### Step 2: Backfill Guard
If `backfill_sync_state` is Active:
*   Calls `try_buffer_payload`.
*   Validates structure (seal), buffers the block in `BlockBuffer`, returns `SYNCING`.

#### Step 3: Insert & Execute (`try_insert_payload`)
Calls `insert_payload` -> `insert_block_or_payload`.
1.  **Missing Parent**: If parent state is unavailable, buffers block.
    *   Returns `InsertPayloadOk::Inserted(BlockStatus::Disconnected)`.
    *   Outcome becomes `PayloadStatus::Syncing`.
    *   *Trigger*: Later triggers `on_disconnected_downloaded_block` to fetch parent.
2.  **Execution**: Calls `payload_validator.validate_block_with_state`.
    *   Executes transactions. checks state root.
    *   If failure: `on_insert_block_error` marks block invalid. Returns `INVALID`.
3.  **Success**: Returns `InsertPayloadOk::Inserted(BlockStatus::Valid)`.
    *   Outcome becomes `PayloadStatus::Valid`.
    *   Tries to connect buffered children (`try_connect_buffered_blocks`).

---

### 5. Invalid Ancestor Tracking

*   **Definition**: `InvalidHeaderCache` in `crates/engine/tree/src/tree/invalid_headers.rs`.
*   **Insertion**: When execution fails in `on_insert_block_error`, `self.state.invalid_headers.insert(block)` is called.
*   **Detection**:
    *   `find_invalid_ancestor` walks the `BlockBuffer` parents.
    *   It checks if any ancestor hash exists in `InvalidHeaderCache`.
*   **Response**:
    *   If detected, `handle_invalid_ancestor_payload` is called.
    *   It calculates `latestValidHash` (the last valid ancestor).
    *   Returns `PayloadStatus::Invalid { validation_error: "LinksToRejectedPayload" }`.

---

### 6. Download / Backfill Triggers

The decision to triggers "Live Sync" (BlockDownloader) vs "Backfill" (Pipeline) happens in `on_disconnected_downloaded_block` or `handle_missing_block`.

#### A. Backfill Trigger (Pipeline)
1.  **Check**: `backfill_sync_target` calls `exceeds_backfill_run_threshold`.
2.  **Threshold**: Checks if `target_block - local_tip > 32` (`MIN_BLOCKS_FOR_PIPELINE_RUN`).
3.  **Emission**:
    ```rust
    // crates/engine/tree/src/tree/mod.rs
    self.emit_event(EngineApiEvent::BackfillAction(BackfillAction::Start(target.into())));
    ```
4.  **Effect**: The `ChainOrchestrator` receives this, pauses the Engine Handler, and starts the Pipeline.

#### B. Download Trigger (Live Sync)
1.  **Check**: If the gap is small (< 32 blocks) or we just need a specific missing parent.
2.  **Calculation**: `distance_from_local_tip`.
3.  **Emission**:
    ```rust
    // crates/engine/tree/src/tree/mod.rs
    self.emit_event(EngineApiEvent::Download(DownloadRequest::BlockRange(missing_parent, distance)));
    ```
4.  **Effect**: The `BasicBlockDownloader` fetches these specific blocks and feeds them back to `on_downloaded_block`.

---

### 7. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `TreeState` | `crates/engine/tree/src/tree/state.rs` | Holds all executed blocks (`blocks_by_hash`) and the `current_canonical_head`. |
| `BlockBuffer` | `crates/engine/tree/src/tree/block_buffer.rs` | Stores blocks waiting for their parents to arrive. |
| `InvalidHeaderCache` | `crates/engine/tree/src/tree/invalid_headers.rs` | LRU cache of invalid block hashes to reject bad chains early. |
| `ForkchoiceStateTracker` | `crates/engine/primitives/src/forkchoice.rs` | Tracks the latest head/safe/finalized hashes received from CL. |
| `TreeOutcome` | `crates/engine/tree/src/tree/mod.rs` | Wrapper returned by handlers containing the `PayloadStatus` and optional `TreeEvent` (Download/Backfill). |

---

### 8. Where to Look Next
1.  **`crates/engine/tree/src/tree/mod.rs` -> `on_new_payload`**: Read this function to see the exact order of validation checks (timestamp, invalid ancestor, execution).
2.  **`crates/engine/tree/src/tree/payload_validator.rs`**: Read `validate_block_with_state` to see the actual EVM execution and state root verification logic.
3.  **`crates/engine/tree/src/tree/mod.rs` -> `find_disk_reorg`**: To see how the engine handles the case where the canonical head in memory diverges from what was persisted to disk.

---

### 9. Suggested Follow-up Questions
*   "How does `EngineApiTreeHandler` ensure exclusive database access when `BackfillAction` is triggered?"
*   "What happens to the `TreeState` in-memory blocks when `on_backfill_sync_finished` is called?"
*   "How does `validate_block_with_state` handle state root calculation? Does it always use the `StateRootTask`?"

## Corrections / Caveats
- `on_forkchoice_updated` does not emit a backfill action when the head is missing. The fallback is always `handle_missing_block`, which emits a `DownloadRequest::single_block` for the target; backfill decisions happen later (e.g., in `on_disconnected_downloaded_block` or after backfill completion).```1212:1243:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `on_new_payload` does not trigger a download when the parent is missing; it buffers and returns `SYNCING`. Download/backfill events are emitted by the downloaded-block path (`on_disconnected_downloaded_block`) or by FCU handling.```641:666:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- The execution path uses `payload_validator.validate_payload`/`validate_block` via `insert_block_or_payload`; there is no `validate_block_with_state` symbol in this module.```2493:2514:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `find_invalid_ancestor` does not walk the entire `BlockBuffer` lineage; it uses `lowest_buffered_ancestor_or` and checks only the invalid-header cache.```2026:2046:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- The backfill threshold check is strict: `block > local_tip && block - local_tip > MIN_BLOCKS_FOR_PIPELINE_RUN` (with `MIN_BLOCKS_FOR_PIPELINE_RUN = EPOCH_SLOTS`).```85:85:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
```2175:2177:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```

## Additional Verified Details
- `on_engine_message` applies the FCU result, updates `ForkchoiceStateTracker`, emits a `ConsensusEngineEvent::ForkchoiceUpdated`, and only then processes any resulting `TreeEvent` (download/backfill/canonicalization).```1451:1477:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- For `newPayload`, the response is sent first, and any `TreeEvent` (e.g., make canonical) is handled afterward via `on_maybe_tree_event`.```1496:1523:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```

## Verification
- `on_forkchoice_updated` runs pre-validation, canonical-head handling, chain update, then missing-block fallback.```1000:1024:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Pre-validation returns invalid for zero head, invalid for known invalid ancestor, and syncing while backfill is active (exclusive DB access).```1031:1054:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `handle_missing_block` chooses safe hash on initial FCU if safe is missing, otherwise the head hash, then emits a download event and returns syncing.```1212:1243:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `apply_chain_update` makes the new head canonical via `on_new_head`, updates safe/finalized, and processes payload attributes when provided.```1130:1206:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `on_new_payload` checks invalid ancestors, then either inserts payloads or buffers them during backfill; valid sync-target heads are turned into `MakeCanonical` events.```596:625:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `try_insert_payload` yields `SYNCING` for disconnected parents and does not emit downloads; invalid payloads route through `on_insert_block_error`/`on_new_payload_error`.```641:675:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `try_buffer_payload` converts payloads to blocks and buffers them during backfill, returning `SYNCING` for well-formed payloads.```688:704:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Missing parent during insert buffers the block and returns `BlockStatus::Disconnected`, with no download event in this path.```2562:2585:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Invalid block insertion records the block in `InvalidHeaderCache` and returns `PayloadStatusEnum::Invalid` with a latest valid hash.```2631:2665:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Invalid ancestor handling uses `InvalidHeaderCache` to mark descendants and respond with `LinksToRejectedPayload`.```1994:2023:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `InvalidHeaderCache` stores invalid headers as `BlockWithParent` entries with LRU eviction.```17:79:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/invalid_headers.rs
```
- Backfill vs download decisions for disconnected downloads happen in `on_disconnected_downloaded_block`, using `backfill_sync_target` and `distance_from_local_tip`.```2379:2418:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- After backfill completion, the tree may re-run backfill or download remaining blocks to reach the FCU target.```1619:1677:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```

## Actionable Pointers
- FCU and newPayload control flow: `reth/crates/engine/tree/src/tree/mod.rs`.
- Invalid header cache: `reth/crates/engine/tree/src/tree/invalid_headers.rs`.
- Buffering of disconnected blocks: `reth/crates/engine/tree/src/tree/block_buffer.rs`.
- Forkchoice tracking: `reth/crates/engine/primitives/src/forkchoice.rs`.
