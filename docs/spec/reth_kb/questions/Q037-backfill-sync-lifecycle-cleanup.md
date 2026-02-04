# Q037 - Backfill Sync Lifecycle and Cleanup

Status: verified

## Question
Backfill completion + state cleanup: what happens when backfill starts and finishes, and how does it reset tree/persistence state?
I want:
1) how BackfillSync start/finish events reach EngineApiTreeHandler (who sends them, who receives),
2) the exact steps in `on_backfill_sync_finished` (invalid header handling, tree reset, buffer cleanup, canonical head updates),
3) when a second backfill run is triggered or a live download is resumed,
4) how backfill interacts with forkchoice tracking and persistence state.

## Context Packs
- `spec/reth_repo_context/context-engine.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-engine.xml

Question (Q037):
Backfill completion + state cleanup: what happens when backfill starts and finishes, and how does it reset tree/persistence state?

I want:
1) how BackfillSync start/finish events reach EngineApiTreeHandler (who sends them, who receives),
2) the exact steps in `on_backfill_sync_finished` (invalid header handling, tree reset, buffer cleanup, canonical head updates),
3) when a second backfill run is triggered or a live download is resumed,
4) how backfill interacts with forkchoice tracking and persistence state.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Call out all state mutations (tree state, buffer, invalid headers, canonical in‑memory state).
3. If something is outside this pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Backfill start path (step-by-step)
4. Backfill finish path (step-by-step)
5. State cleanup and canonical head reset
6. Download/backfill re‑trigger logic
7. Key abstractions/types (with file paths)
8. "Where to look next"
9. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-engine.xml` pack, here is the detailed analysis of the backfill synchronization lifecycle, state cleanup, and coordination logic.

### 1. Summary
*   **Orchestrator-Driven**: The `ChainOrchestrator` manages the mutual exclusion between the `Pipeline` (Backfill) and `EngineHandler` (Live Sync). It prioritizes the pipeline when active.
*   **Circular Event Flow**: The Tree requests a backfill -> Bubbles to Orchestrator -> Starts Pipeline -> Pipeline Finishes -> Orchestrator notifies Tree -> Tree cleans up.
*   **Destructive Cleanup**: Upon backfill completion, the in-memory `TreeState` is aggressively pruned (`remove_until`) or reset (`reset` on unwind) because the data is now safely persisted in the database.
*   **Memory State Reset**: The `CanonicalInMemoryState` is completely cleared (`clear_state()`) after backfill, forcing the node to reload the canonical head from the database.
*   **Persistence Synchronization**: The `PersistenceState` is updated via `finish()` to align the "Last Persisted" pointer with the new pipeline tip, ensuring subsequent writes don't overlap or gap.
*   **Gap Assessment**: After cleanup, the engine immediately calculates the distance to the *next* target (Finalized or Head). It may trigger another Backfill run (large gap) or switch to Live Download (small gap).

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/engine/tree/src/tree/mod.rs` | `on_backfill_sync_finished` | The critical logic handling the post-pipeline state reset and decision making. |
| `crates/engine/tree/src/chain.rs` | `ChainOrchestrator::poll_next_event` | The loop that detects pipeline completion and forwards the result to the handler. |
| `crates/engine/tree/src/backfill.rs` | `PipelineSync::poll` | The future driving the actual pipeline task and emitting `BackfillEvent::Finished`. |
| `crates/engine/tree/src/tree/mod.rs` | `on_engine_message` | The handler receiving `FromOrchestrator::BackfillSyncFinished` on the tree thread. |

---

### 3. Backfill Start Path

This sequence shows how the system enters backfill mode.

1.  **Detection (Tree)**: `EngineApiTreeHandler` detects a large gap (e.g., in `handle_missing_block`).
2.  **Emission**: It emits `EngineApiEvent::BackfillAction(Start(target))` via `self.outgoing`.
3.  **Routing**: `EngineHandler` receives this, pauses the `BlockDownloader` (`DownloadAction::Clear`), and forwards it to the `ChainOrchestrator`.
4.  **Activation (Orchestrator)**: `ChainOrchestrator` calls `self.backfill_sync.on_action(Start)`.
5.  **Notification**: The Orchestrator immediately sends `FromOrchestrator::BackfillSyncStarted` back down to the handler.
6.  **State Lock (Tree)**: `EngineApiTreeHandler` receives this event and sets `self.backfill_sync_state = BackfillSyncState::Active`.
    *   *Effect*: This causes `validate_forkchoice_state` to return `SYNCING` for all future requests, effectively pausing Engine API processing.

---

### 4. Backfill Finish Path

This sequence shows how the system exits backfill mode.

1.  **Completion**: The `Pipeline` task completes and sends the result via a `oneshot` channel to `PipelineSync`.
2.  **Poll**: `PipelineSync::poll` returns `Poll::Ready(BackfillEvent::Finished(Result<ControlFlow>))`.
3.  **Orchestrator Handling**:
    *   **File**: `crates/engine/tree/src/chain.rs`
    *   **Logic**: `poll_next_event` matches `BackfillEvent::Finished`.
    *   **Action**: Calls `this.handler.on_event(FromOrchestrator::BackfillSyncFinished(ctrl))`.
4.  **Tree Handling**:
    *   **File**: `crates/engine/tree/src/tree/mod.rs`
    *   **Logic**: `on_engine_message` calls `self.on_backfill_sync_finished(ctrl)`.

---

### 5. State Cleanup and Canonical Head Reset

The `on_backfill_sync_finished` function (`crates/engine/tree/src/tree/mod.rs`) performs these specific mutations:

#### A. Unwind Handling (Bad Block)
If `ctrl` is `ControlFlow::Unwind { bad_block, .. }`:
1.  **Invalidate**: `self.state.invalid_headers.insert(bad_block)`.
2.  **Tree Reset**: `self.state.tree_state.reset(backfill_num_hash)`.
    *   *Effect*: **Destructive**. Wipes `blocks_by_hash` entirely. The tree forgets all in-memory blocks and points the head to the valid ancestor.

#### B. Successful Sync (Continue)
If `ctrl` is `ControlFlow::Continue`:
1.  **Tree Pruning**: `self.state.tree_state.remove_until(backfill_num_hash, ...)`.
    *   *Effect*: Removes all executed blocks from memory that are now in the database (below the backfill height). Keeps sidechains that might fork *after* the backfill height.

#### C. Universal Cleanup (Both Cases)
1.  **Buffer**: `self.state.buffer.remove_old_blocks(backfill_height)`.
    *   *Effect*: Evicts pending blocks that are now too old.
2.  **In-Memory State**: `self.canonical_in_memory_state.clear_state()`.
    *   *Effect*: Clears the `MemoryOverlayStateProvider` cache. The next state lookup will force a database read.
3.  **Head Update**:
    *   `self.state.tree_state.set_canonical_head(new_head)`.
    *   `self.persistence_state.finish(hash, number)`. (Updates `last_persisted_block` to align with DB).
    *   `self.canonical_in_memory_state.set_canonical_head(new_head)`.

---

### 6. Download / Backfill Re-trigger Logic

After cleanup, `on_backfill_sync_finished` checks if we are actually done or just finished *one stage* of syncing.

#### Step 1: Check Finalized
It checks `state.forkchoice_state_tracker.sync_target_state()`.

#### Step 2: Distance Check
It calls `self.backfill_sync_target(progress, finalized_number, ...)`.
*   If `progress` is still far behind `finalized`:
    *   **Action**: `emit_event(BackfillAction::Start(new_target))`.
    *   **Result**: Pipeline runs again immediately.

#### Step 3: Check Head (Live Sync Handoff)
If we are close to finalized but behind the **Head**:
1.  It finds the `lowest_buffered_ancestor` of the Head hash.
2.  Calculates `distance_from_local_tip`.
3.  **Action**: `emit_event(Download(DownloadRequest::BlockRange(parent, distance)))`.
4.  **Result**: The `BlockDownloader` starts fetching the remaining small gap (Live Sync).

#### Step 4: Connect Buffered
Finally, it calls `try_connect_buffered_blocks` to execute any blocks that were sitting in `BlockBuffer` waiting for the parent that backfill just provided.

---

### 7. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `BackfillSyncState` | `crates/engine/tree/src/backfill.rs` | Enum (`Idle`, `Pending`, `Active`) tracking pipeline status. Guards FCU processing. |
| `ControlFlow` | `reth-stages-api` (referenced) | Result from Pipeline: `Continue` (Success) or `Unwind` (Reorg/Bad Block). |
| `PersistenceState` | `crates/engine/tree/src/tree/persistence_state.rs` | Tracks `last_persisted_block`. Updated via `finish()` to match pipeline output. |
| `CanonicalInMemoryState` | `reth-chain-state` (referenced) | The in-memory overlay. `clear_state()` is called to invalidate it after DB writes. |
| `TreeState` | `crates/engine/tree/src/tree/state.rs` | The core block tree. `reset()` or `remove_until()` is called to prune it. |

---

### 8. Where to Look Next
1.  **`crates/engine/tree/src/tree/mod.rs`**: Read `on_backfill_sync_finished` (approx line ~1350). This is the definitive convergence point for state management.
2.  **`crates/engine/tree/src/chain.rs`**: Read `poll_next_event`. This shows how the orchestrator suppresses the `handler` when the `backfill_sync` returns `Ready`.
3.  **`crates/engine/tree/src/tree/state.rs`**: Read `remove_until`. This explains exactly how the tree decides which blocks to drop from memory vs keep as potential sidechains.

---

### 9. Suggested Follow-up Questions
*   "How does `CanonicalInMemoryState` populate itself after a `clear_state()` call? Does it lazily load from DB or wait for new blocks?"
*   "What happens if `on_backfill_sync_finished` triggers a `DownloadRequest` but the peer network is empty?"
*   "Why does `EngineApiTreeHandler` need to `reinsert_reorged_blocks` during a normal reorg, but `reset` the tree completely during a backfill unwind?"

## Corrections / Caveats
- Backfill does not originate from `handle_missing_block`: that path always emits a `DownloadRequest::single_block`. Backfill actions are emitted from `on_disconnected_downloaded_block` (when a downloaded block is still missing parents) or from `on_backfill_sync_finished` (re-run).```1212:1243:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
```2379:2418:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
```1619:1651:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Backfill is prioritized but not strictly exclusive: `ChainOrchestrator` polls backfill first and still polls the handler when backfill returns `Pending`. The handler relies on `BackfillSyncState` gating to avoid DB writes.```75:135:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/chain.rs
```
```1031:1054:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Emitting a backfill action sets `backfill_sync_state` to `Pending` and is skipped entirely while a persistence task is in progress.```1726:1745:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `remove_until` removes canonical blocks up to the upper bound and prunes sidechains that fork below the finalized hash; it does not remove all non-canonical branches above that fork point.```249:290:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/state.rs
```
- `persistence_state.finish(...)` runs only if the backfill height header is found; otherwise the function returns early.```1579:1617:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```

## Verification
- `PipelineSync::poll` emits `BackfillEvent::Started` when it spawns the pipeline and `BackfillEvent::Finished` when the oneshot resolves.```132:176:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/backfill.rs
```
- `ChainOrchestrator::poll_next_event` forwards Backfill start/finish events to the handler as `FromOrchestrator`.```78:104:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/chain.rs
```
- `EngineHandler` forwards `FromOrchestrator` events to the tree via `EngineApiRequestHandler`.```77:80:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/engine.rs
```
```198:201:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/engine.rs
```
- The tree sets `backfill_sync_state = Active` on start and calls `on_backfill_sync_finished` on finish.```1405:1412:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- `on_backfill_sync_finished` sets `backfill_sync_state` to idle, records invalid headers on unwind, resets or prunes `TreeState`, clears buffer and in-memory state, and aligns canonical head and persistence state with the backfill tip.```1555:1617:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- Backfill re-run logic uses `forkchoice_state_tracker.sync_target_state()` plus `backfill_sync_target`, and emits another `BackfillAction::Start` when the gap is still large.```1619:1651:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```
- If the remaining gap is small, it emits `DownloadRequest::BlockRange` to resume live download; otherwise it connects buffered blocks.```1655:1682:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
```

## Actionable Pointers
- Backfill state machine and events: `reth/crates/engine/tree/src/backfill.rs`.
- Orchestrator routing: `reth/crates/engine/tree/src/chain.rs`.
- Backfill cleanup and re-trigger logic: `reth/crates/engine/tree/src/tree/mod.rs`.
- Pruning semantics: `reth/crates/engine/tree/src/tree/state.rs`.
