# Q018 - Canonical Chain and Reorg Detection

Status: verified

## Question
How is the canonical chain represented and how are reorgs detected?

## Context Packs
- `spec/reth_repo_context/context-engine.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-engine.xml

Question (Q018):
How is the canonical chain represented and how are reorgs detected?
I want the core data structures, how canonical head is tracked, and how reorg detection flows into pipeline or engine logic.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe how canonical state is updated on new blocks.
3. Explain how reorg detection is surfaced (events, errors, or unwind requests).
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
Based on the `context-engine.xml` pack, here is the architecture of how the canonical chain is represented and how reorgs are detected in the Reth engine.

### 1. Summary
*   **In-Memory Tree Structure**: The canonical chain is tracked within `TreeState`, which maintains a collection of executed blocks (`blocks_by_hash`) and their parent-child relationships (`parent_to_child`).
*   **Head Tracking**: The engine explicitly tracks a `current_canonical_head`. Any block not strictly following this head is part of a sidechain until chosen.
*   **Reorg Detection via Walk-Back**: Reorgs are detected in `on_new_head` by walking backwards from the proposed new head and the current canonical head simultaneously until a common ancestor (fork point) is found.
*   **Classification**: A chain update is classified as either a `Commit` (simple extension) or a `Reorg` (switching to a different fork) via the `NewCanonicalChain` enum.
*   **Preservation of History**: When a reorg occurs, blocks from the "old" canonical chain are re-inserted into the tree state as sidechains (`reinsert_reorged_blocks`), ensuring they are available if the chain reorgs back.
*   **Pipeline Integration**: Deep reorgs or invalid blocks detected during backfill trigger a `ControlFlow::Unwind` in the pipeline, which the engine handles by resetting the tree state to the unwind target.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/engine/tree/src/tree/state.rs` | `TreeState` | The central data structure holding all known executed blocks and the pointer to the `current_canonical_head`. |
| `crates/engine/tree/src/tree/mod.rs` | `EngineApiTreeHandler::on_new_head` | The logic core that calculates the path from the current head to a proposed head, detecting reorgs. |
| `crates/engine/tree/src/tree/mod.rs` | `EngineApiTreeHandler::on_canonical_chain_update` | The function that actually applies the reorg, updates metrics, and emits events. |
| `crates/engine/tree/src/tree/mod.rs` | `EngineApiTreeHandler::on_backfill_sync_finished` | Handles reorgs/unwinds triggered by the historical sync pipeline (e.g., bad blocks found during backfill). |

---

### 3. Algorithm Flow: Reorg Detection (`on_new_head`)

The core reorg detection logic resides in `on_new_head` within `crates/engine/tree/src/tree/mod.rs`.

#### Step 1: Identify New Head
The engine receives a request to make a specific block hash canonical (e.g., via Forkchoice Updated). It retrieves the `ExecutedBlock` from `TreeState`.

#### Step 2: Connect to Tree
The function walks backwards from the `new_head` until it finds a block hash that exists in the current `TreeState` or reaches the canonical head.

#### Step 3: Determine Relationship
*   **Extension (Commit)**: If the walk-back leads directly to `current_canonical_head`, it is a simple append. Returns `NewCanonicalChain::Commit`.
*   **Reorg**: If the walk-back hits a block that is *not* the current head, or if the current head is at a higher block number than the intersection point, it is a reorg.

#### Step 4: Find Common Ancestor (The Fork Point)
In the reorg case:
1.  The logic walks back the **Current Canonical Chain** (`old_chain`) and the **New Chain** (`new_chain`) simultaneously.
2.  It identifies the **Common Ancestor** (where `old_hash == current_hash`).
3.  It collects all blocks from the Old Chain that are being discarded and all blocks from the New Chain that are being enacted.

#### Step 5: Construct Notification
It constructs a `NewCanonicalChain::Reorg { new, old }` struct containing the vectors of added and removed blocks.

#### Step 6: Application (`on_canonical_chain_update`)
1.  **Update Head**: Sets `tree_state.current_canonical_head` to the new tip.
2.  **Reinsert Old**: Calls `reinsert_reorged_blocks` to move the `old` blocks back into the general `blocks_by_hash` pool (effectively turning them into sidechains).
3.  **Emit**: Emits `ConsensusEngineEvent::CanonicalChainCommitted`.

---

### 4. Key Abstractions & Data Types

*   **`TreeState`** (`crates/engine/tree/src/tree/state.rs`):
    *   Holds `blocks_by_hash: HashMap<B256, ExecutedBlock<N>>`.
    *   Holds `parent_to_child` to track tree topology.
    *   Tracks `current_canonical_head`.
*   **`NewCanonicalChain`** (`crates/chain-state/src/in_memory.rs`):
    *   Variants: `Commit { new }` and `Reorg { new, old }`.
*   **`TreeAction`** (`crates/engine/tree/src/tree/mod.rs`):
    *   Enum used to signal internal intent, specifically `TreeAction::MakeCanonical`.
*   **`ForkchoiceStateTracker`** (`crates/engine/primitives/src/forkchoice.rs`):
    *   Tracks the `latest`, `last_valid`, and `last_syncing` forkchoice states received from the Beacon node to determine if we are in sync or need to trigger downloads/reorgs.

---

### 5. Where to Look Next

1.  **`crates/engine/tree/src/tree/mod.rs`**: Specifically `on_new_head` and `on_canonical_chain_update`, which implement reorg detection and canonical updates.
2.  **`crates/engine/tree/src/tree/state.rs`**: The `TreeState` struct definition and its invariants for what is kept in memory.
3.  **`crates/engine/tree/src/tree/metrics.rs`**: `TreeMetrics` and reorg counters such as `reorgs` and `latest_reorg_depth`.

---

### 6. Suggested Follow-up Questions

*   "How does `EngineApiTreeHandler` decide to trigger a backfill sync instead of handling a reorg in-memory if the gap is too large?"
*   "How does `PersistenceService` handle disk reorgs (`find_disk_reorg`) differently from in-memory reorgs?"
*   "What happens to the `InvalidHeaderCache` if a reorg switches to a chain that contains a block previously marked invalid?"

## Verification
- TreeState stores executed blocks by hash/number, parent-child edges, and a canonical head pointer.```24:39:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/state.rs
pub struct TreeState<N: NodePrimitives = EthPrimitives> {
    pub(crate) blocks_by_hash: HashMap<B256, ExecutedBlock<N>>,
    pub(crate) blocks_by_number: BTreeMap<BlockNumber, Vec<ExecutedBlock<N>>>,
    pub(crate) parent_to_child: HashMap<B256, HashSet<B256>>,
    pub(crate) current_canonical_head: BlockNumHash,
}
```
- on_new_head walks back the new chain to the canonical height, then either commits or walks both chains to produce a reorg.```714:803:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_new_head(&self, new_head: B256) -> ProviderResult<Option<NewCanonicalChain<N>>> {
    ... walk back new chain ...
    if current_hash == self.state.tree_state.current_canonical_head.hash { return Ok(Some(NewCanonicalChain::Commit { new: new_chain })) }
    ... walk old and new chains until common ancestor ...
    Ok(Some(NewCanonicalChain::Reorg { new: new_chain, old: old_chain }))
}
```
- NewCanonicalChain defines Commit/Reorg variants and is in chain-state.```907:922:/Users/vicnaum/github/stateless-history-node/reth/crates/chain-state/src/in_memory.rs
pub enum NewCanonicalChain<N: NodePrimitives = EthPrimitives> {
    Commit { new: Vec<ExecutedBlock<N>> },
    Reorg { new: Vec<ExecutedBlock<N>>, old: Vec<ExecutedBlock<N>> },
}
```
- Canonical updates set the tree's canonical head, reinsert reorged blocks, update in-memory state, and emit CanonicalChainCommitted.```2317:2355:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_canonical_chain_update(&mut self, chain_update: NewCanonicalChain<N>) {
    self.state.tree_state.set_canonical_head(chain_update.tip().num_hash());
    if let NewCanonicalChain::Reorg { new, old } = &chain_update {
        self.reinsert_reorged_blocks(new.clone());
        self.reinsert_reorged_blocks(old.clone());
    }
    self.canonical_in_memory_state.update_chain(chain_update);
    self.emit_event(ConsensusEngineEvent::CanonicalChainCommitted(...));
}
```
- Backfill unwind resets the tree state to the unwind target and updates canonical head tracking.```1558:1617:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
if ctrl.is_unwind() {
    self.state.tree_state.reset(backfill_num_hash)
} else {
    self.state.tree_state.remove_until(...)
}
...
self.state.tree_state.set_canonical_head(new_head.num_hash());
```
- Canonical chain events are emitted via ConsensusEngineEvent.```20:35:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/primitives/src/event.rs
pub enum ConsensusEngineEvent { ... CanonicalChainCommitted(...), ... }
```

## Corrections / Caveats
- TreeState also tracks `blocks_by_number`, not only `blocks_by_hash` and `parent_to_child`.
- The "sidechain until chosen" statement is implicit; TreeState only stores executed blocks connected to the canonical chain (including forks).
- The backfill unwind handling is in `EngineApiTreeHandler::on_backfill_sync_finished`, not a direct pipeline hook.
- ForkchoiceStateTracker usage is present in tree state, but its internal logic is outside this pack.

## Actionable Pointers
- Reorg detection: `reth/crates/engine/tree/src/tree/mod.rs` (`on_new_head`).
- Canonical update and reorg handling: `reth/crates/engine/tree/src/tree/mod.rs` (`on_canonical_chain_update`, `reinsert_reorged_blocks`).
- Tree storage invariants: `reth/crates/engine/tree/src/tree/state.rs`.
- Canonical chain type: `reth/crates/chain-state/src/in_memory.rs` (`NewCanonicalChain`).
