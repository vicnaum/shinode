# Q019 - Engine Tree Backfill Handoff

Status: verified

## Question
How does the engine tree hand off to the pipeline and handle backfill?

## Context Packs
- `spec/reth_repo_context/context-engine.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-engine.xml

Question (Q019):
How does the engine tree hand off to the pipeline and handle backfill?
I want the decision criteria, event flow, and how backfill results are applied to the tree state.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe the control flow between EngineApiTreeHandler and pipeline backfill.
3. Explain how tree state changes on backfill start/finish, including unwind cases.
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
Based on the `context-engine.xml` pack, here is the architecture of how the Engine Tree hands off synchronization to the Pipeline and handles the backfill process.

### 1. Summary
*   **Gap-Based Trigger**: The engine switches to backfill mode when the gap between the local tip and the target block (from a Forkchoice Update or downloaded block) exceeds `MIN_BLOCKS_FOR_PIPELINE_RUN` (32 blocks).
*   **Orchestrated Handoff**: The `ChainOrchestrator` sits above both the `EngineHandler` and the `BackfillSync` (Pipeline). The engine requests a backfill, and the orchestrator activates the pipeline.
*   **Mutually Exclusive**: When backfill is active (`BackfillSyncState::Active`), the engine pauses processing new Forkchoice Updates, returning `SYNCING` to the Consensus Layer.
*   **State Reset**: Upon backfill completion, the engine purges in-memory blocks that are now persisted in the database and resets its in-memory canonical state to match the new database tip.
*   **Unwind Handling**: If the pipeline detects a bad block, it returns a `ControlFlow::Unwind`. The engine handles this by marking the block invalid and resetting the in-memory tree to the unwind point.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/engine/tree/src/chain.rs` | `ChainOrchestrator` | The top-level state machine that polls both the `handler` (Engine) and `backfill_sync` (Pipeline). |
| `crates/engine/tree/src/tree/mod.rs` | `EngineApiTreeHandler::backfill_sync_target` | Calculates the distance to a target and decides if backfill is needed. |
| `crates/engine/tree/src/backfill.rs` | `PipelineSync` | Wraps the actual stages pipeline and spawns it on a background task. |
| `crates/engine/tree/src/tree/mod.rs` | `on_backfill_sync_finished` | The callback logic that updates the tree state after the pipeline finishes. |

---

### 3. Algorithm Flow

#### Step 1: Detection (Engine Tree)
When `on_forkchoice_updated` or `on_downloaded_block` runs, the engine checks the distance between the local head and the new block.
*   **Function**: `backfill_sync_target` in `crates/engine/tree/src/tree/mod.rs`.
*   **Logic**: It calls `exceeds_backfill_run_threshold`. If `target_block - local_tip > MIN_BLOCKS_FOR_PIPELINE_RUN` (32), it returns `Some(target_hash)`.

#### Step 2: Handoff (Tree -> Orchestrator)
The tree emits an event to request backfill.
*   **Action**: `emit_event(EngineApiEvent::BackfillAction(BackfillAction::Start(target)))`.
*   **State Change**: `backfill_sync_state` is set to `Pending`.
*   **Routing**: The event bubbles up through `EngineApiRequestHandler` -> `EngineHandler` -> `ChainOrchestrator`.

#### Step 3: Execution (Orchestrator -> Pipeline)
The `ChainOrchestrator` receives the request and commands the pipeline.
*   **File**: `crates/engine/tree/src/chain.rs`.
*   **Logic**: `poll_next_event` receives `HandlerEvent::BackfillAction`. It calls `self.backfill_sync.on_action(action)`.
*   **Spawn**: `PipelineSync::try_spawn_pipeline` (`crates/engine/tree/src/backfill.rs`) spawns the pipeline on a generic `TaskSpawner`.

#### Step 4: Active Syncing (Engine Wait)
While the pipeline runs, `BackfillSyncState` is `Active`.
*   **Impact**: In `validate_forkchoice_state` (`tree/mod.rs`), if `backfill_sync_state` is not idle, it immediately returns `PayloadStatus::Syncing` to the Consensus Layer, preventing the engine from touching the database while the pipeline writes to it.

#### Step 5: Completion & State Update
The pipeline finishes and returns a `ControlFlow`.
*   **Orchestrator**: Receives `BackfillEvent::Finished`. Calls `handler.on_event(FromOrchestrator::BackfillSyncFinished)`.
*   **Tree Handler**: `on_backfill_sync_finished` (`tree/mod.rs`) executes:
    1.  **State Cleanup**: Calls `tree_state.remove_until` to delete in-memory blocks that the pipeline just persisted to disk.
    2.  **Canonical Reset**: `canonical_in_memory_state.clear_state()` is called because the database is now the authority.
    3.  **Head Update**: `tree_state.set_canonical_head` updates the local tip to the pipeline's result.

#### Step 6: Handling Unwinds
If the pipeline returns `ControlFlow::Unwind` (e.g., due to a bad block found during sync):
*   **Logic**: `on_backfill_sync_finished` detects `ctrl.is_unwind()`.
*   **Action**: It inserts the bad block into `invalid_headers`.
*   **Reset**: Calls `tree_state.reset(backfill_num_hash)` to completely clear the in-memory tree above the unwind point, forcing the node to re-sync from the valid ancestor.

---

### 4. Key Abstractions & Types

*   **`BackfillSync` (Trait)**: Abstract interface for the backfill process. Implemented by `PipelineSync`.
*   **`PipelineSync`** (`crates/engine/tree/src/backfill.rs`): Wraps the `Pipeline`. It uses a `oneshot` channel to receive the result (`PipelineWithResult`) from the spawned task.
*   **`BackfillSyncState`**: Enum (`Idle`, `Pending`, `Active`) stored in `EngineApiTreeHandler`. Used to guard database access.
*   **`ControlFlow`**: Return type from the pipeline. Variants: `Continue` (sync success), `Unwind` (reorg/bad block), `NoProgress`.
*   **`MIN_BLOCKS_FOR_PIPELINE_RUN`**: Constant (`32`) defining the boundary between "live sync" (downloading single blocks via `BlockDownloader`) and "backfill sync" (running the Pipeline).

---

### 5. Where to Look Next

1.  **`crates/engine/tree/src/tree/mod.rs`**: Specifically `on_backfill_sync_finished`. This is the critical convergence point where the "old" world (in-memory tree) reconciles with the "new" world (database state after pipeline).
2.  **`crates/engine/tree/src/chain.rs`**: The `poll_next_event` loop shows the priority inversion: the orchestrator polls the backfill sync *before* the handler, ensuring the pipeline drives the process when active.
3.  **`crates/engine/tree/src/backfill.rs`**: `try_spawn_pipeline` shows how the `TaskSpawner` is used to run the pipeline without blocking the engine's event loop.

---

### 6. Suggested Follow-up Questions

*   "How does `PipelineSync` ensure exclusive write access to the database, given that `EngineApiTreeHandler` also writes to the DB via `PersistenceHandle`?"
*   "What happens to `pending_events` in the `BlockDownloader` when a backfill action triggers `DownloadAction::Clear`?"
*   "How does the `CanonicalInMemoryState` populate itself after a backfill finishes and clears the previous state?"

## Verification
- Backfill threshold uses `MIN_BLOCKS_FOR_PIPELINE_RUN = EPOCH_SLOTS` and is checked via `exceeds_backfill_run_threshold`.```76:85:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
pub(crate) const MIN_BLOCKS_FOR_PIPELINE_RUN: u64 = EPOCH_SLOTS;
```
```2170:2177:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
const fn exceeds_backfill_run_threshold(&self, local_tip: u64, block: u64) -> bool {
    block > local_tip && block - local_tip > MIN_BLOCKS_FOR_PIPELINE_RUN
}
```
- Backfill target selection uses forkchoice sync target state and may select the finalized hash or head (optimistic) when exceeding the threshold.```2190:2261:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn backfill_sync_target(&self, canonical_tip_num: u64, target_block_number: u64, downloaded_block: Option<BlockNumHash>) -> Option<B256> { ... }
```
- Backfill actions are emitted by the tree, set `backfill_sync_state` to Pending, and are blocked if persistence is in progress.```1726:1746:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
if event.is_backfill_action() {
    if self.persistence_state.in_progress() { ... return }
    self.backfill_sync_state = BackfillSyncState::Pending;
}
```
- Forkchoice updates return Syncing while backfill is active.```1035:1055:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
if !self.backfill_sync_state.is_idle() {
    return Ok(Some(OnForkChoiceUpdated::syncing()));
}
```
- ChainOrchestrator polls backfill sync first, then handler; it forwards backfill actions and start/finish events.```74:127:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/chain.rs
match this.backfill_sync.poll(cx) { ... BackfillEvent::Started/Finished ... }
...
match handler_event { HandlerEvent::BackfillAction(action) => this.backfill_sync.on_action(action); }
```
- PipelineSync spawns the pipeline with a oneshot result and marks it active.```132:176:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/backfill.rs
fn try_spawn_pipeline(&mut self) -> Option<BackfillEvent> { ... spawn_critical_blocking ... pipeline.run_as_fut(Some(target)) ... }
```
- EngineHandler forwards backfill actions and clears the downloader before handoff.```82:93:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/engine.rs
HandlerEvent::BackfillAction(target) => {
    self.downloader.on_action(DownloadAction::Clear);
    Poll::Ready(HandlerEvent::BackfillAction(target))
}
```
- Backfill finish handling clears/updates tree state; unwind resets the tree to the backfill height and marks invalid headers.```1551:1617:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
if let ControlFlow::Unwind { bad_block, target } = &ctrl { ... self.state.invalid_headers.insert(**bad_block); }
...
if ctrl.is_unwind() { self.state.tree_state.reset(backfill_num_hash) } else { self.state.tree_state.remove_until(...) }
...
self.canonical_in_memory_state.clear_state();
self.state.tree_state.set_canonical_head(new_head.num_hash());
```

## Corrections / Caveats
- The threshold is `> MIN_BLOCKS_FOR_PIPELINE_RUN` (strictly greater), not `>=`.
- Backfill is tied to forkchoice state and finalized hash handling, not just "tip vs target"; optimistic mode can pick the head hash when finalized is unknown.
- The orchestrator does not directly poll the tree; it polls the handler, which emits backfill actions via `EngineApiEvent`.
- PipelineSync notes that running the pipeline acquires exclusive DB write access; the handler skips writes during backfill to avoid deadlocks.

## Actionable Pointers
- Backfill decision logic: `reth/crates/engine/tree/src/tree/mod.rs` (`backfill_sync_target`, `exceeds_backfill_run_threshold`).
- Orchestration flow: `reth/crates/engine/tree/src/chain.rs` and `reth/crates/engine/tree/src/engine.rs`.
- Pipeline execution wrapper: `reth/crates/engine/tree/src/backfill.rs`.
