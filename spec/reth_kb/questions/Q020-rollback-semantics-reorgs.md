# Q020 - Rollback Semantics on Reorgs

Status: verified

## Question
What are rollback semantics on reorgs across engine + persistence?

## Context Packs
- `spec/reth_repo_context/context-engine.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-engine.xml

Question (Q020):
What are rollback semantics on reorgs across engine + persistence?
I want how in-memory state and persisted blocks are rolled back, and which paths trigger disk reorg handling.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Describe how reorgs affect TreeState vs CanonicalInMemoryState.
3. Explain how on-disk reorg detection works (find_disk_reorg / remove_blocks).
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
Based on the `context-engine.xml` pack, here is the analysis of rollback semantics and reorg handling in Reth.

### 1. Summary
*   **Two-Layer Reorgs**: Reth handles reorgs in two distinct layers: **In-Memory** (instant, frequent) and **Persistence** (background, infrequent).
*   **In-Memory Swapping**: The `TreeState` keeps all executed blocks (canonical and sidechains). When an in-memory reorg occurs, the engine simply updates the `current_canonical_head` pointer and swaps the relevant blocks in `CanonicalInMemoryState`. No re-execution is needed if the blocks are already in the tree.
*   **Preservation of History**: When a reorg happens, the "old" canonical blocks are not deleted; they are re-inserted into `TreeState` as sidechains (`reinsert_reorged_blocks`), ensuring they are available if the chain reorgs back.
*   **Async Disk Reorgs**: Because persistence happens in the background, the database might contain blocks that are no longer canonical. The engine detects this via `find_disk_reorg`.
*   **Destructive Disk Rollback**: If a disk reorg is detected, the engine issues a `PersistenceAction::RemoveBlocksAbove` command. The persistence service physically deletes the invalid blocks and execution data from the database to align it with the canonical in-memory state.
*   **Unwind on Backfill**: If a bad block is detected during backfill/pipeline sync, the engine triggers a "Reset," clearing the in-memory tree state entirely above the valid ancestor.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/engine/tree/src/tree/mod.rs` | `on_new_head` | Core logic to detect if a new head requires a `Commit` (extend) or `Reorg` (switch fork). |
| `crates/engine/tree/src/tree/mod.rs` | `find_disk_reorg` | Determines if the block persisted on disk diverges from the current in-memory canonical chain. |
| `crates/engine/tree/src/tree/mod.rs` | `advance_persistence` | The loop that checks for disk reorgs and triggers removal if found. |
| `crates/engine/tree/src/persistence.rs` | `PersistenceService::on_remove_blocks_above` | The handler that executes the physical deletion of blocks from the DB. |
| `crates/engine/tree/src/tree/state.rs` | `TreeState::reinsert_reorged_blocks` | Ensures blocks removed from the canonical chain are kept as sidechains. |

---

### 3. Reorg Algorithm Flow

#### A. In-Memory Reorg (High Frequency)
Triggered by `on_forkchoice_updated` when the new head is not a direct child of the current head.

1.  **Detection**: `on_new_head` walks backwards from the new head and the current head until a common ancestor is found.
2.  **Classification**: Returns `NewCanonicalChain::Reorg { new, old }`.
3.  **Application**: `on_canonical_chain_update` is called.
    *   **Update Head**: `tree_state.set_canonical_head(new_tip)`.
    *   **Swap**: `canonical_in_memory_state.update_chain(...)` replaces the chain view.
    *   **Preserve Old**: `reinsert_reorged_blocks(old)` ensures the discarded blocks remain in `blocks_by_hash` as sidechains.
    *   **Emit**: Emits `CanonicalChainCommitted`.

#### B. On-Disk Reorg (Background Correction)
Triggered during the `advance_persistence` check in the main engine loop.

1.  **Check**: `find_disk_reorg` compares `persistence_state.last_persisted_block` against `state.tree_state.current_canonical_head`.
2.  **Walk Back**: It walks the in-memory canonical chain backwards.
    *   If `last_persisted_block` is found in the canonical ancestry: **No Reorg**.
    *   If not found: **Reorg Detected**.
3.  **Find Ancestor**: It walks both chains (persisted vs canonical) backwards until `common_ancestor` is found.
4.  **Trigger Removal**: It calls `self.remove_blocks(common_ancestor.number)`.
5.  **Execution**:
    *   Sends `PersistenceAction::RemoveBlocksAbove` to the `PersistenceService`.
    *   Service calls `provider_rw.remove_block_and_execution_above`.
    *   Blocks/Receipts/State above the common ancestor are deleted from the DB.
6.  **Cleanup**: Once the persistence service confirms deletion, `on_new_persisted_block` ensures in-memory state is consistent.

#### C. Backfill Unwind (Bad Block)
Triggered if the pipeline returns `ControlFlow::Unwind`.

1.  **Trigger**: `on_backfill_sync_finished` receives the unwind signal.
2.  **Mark Invalid**: The bad block is added to `invalid_headers` cache.
3.  **Reset**: `tree_state.reset(valid_ancestor)` is called.
    *   This completely wipes the `blocks_by_hash` map in `TreeState`, effectively dropping all sidechains and pending blocks.
    *   The engine restarts synchronization from the valid ancestor.

---

### 4. Key Abstractions & Types

*   **`TreeState`** (`crates/engine/tree/src/tree/state.rs`):
    *   The "God View" of all executed blocks. It doesn't care about canonical vs sidechain, it stores everything validated. Reorgs here are just pointer updates.
*   **`CanonicalInMemoryState`** (`reth-chain-state` - referenced in `tree/mod.rs`):
    *   Represents the specific chain the node currently follows. It must always align with the tip of `TreeState`.
*   **`PersistenceState`** (`crates/engine/tree/src/tree/persistence_state.rs`):
    *   Tracks `last_persisted_block`. This pointer lags behind the in-memory head. The gap between this and the in-memory head creates the possibility for "Disk Reorgs".
*   **`NewCanonicalChain`** (Enum):
    *   Variants: `Commit` (Append) or `Reorg` (Switch). Used to signal changes to the rest of the system.

---

### 5. Where to Look Next

1.  **`crates/engine/tree/src/tree/mod.rs`**: Read `find_disk_reorg` (approx line 1500). It contains the exact logic for determining if the database has drifted from the memory state.
2.  **`crates/engine/tree/src/persistence.rs`**: Read `on_remove_blocks_above`. This is the "destructive" part of the rollback where data is actually purged.
3.  **`crates/engine/tree/src/tree/state.rs`**: Read `prune_finalized_sidechains`. This shows how old forks (history that wasn't chosen) are eventually garbage collected from memory.

---

### 6. Suggested Follow-up Questions

*   "How does `MemoryOverlayStateProvider` behave during a reorg? Does it have to discard cached state lookups?"
*   "What happens if the `PersistenceService` fails (crashes) during a `RemoveBlocksAbove` operation? Is the database left in an inconsistent state?"
*   "How does the `InvalidHeaderCache` prevent the engine from re-downloading and re-executing a block that caused a reorg/unwind previously?"

## Verification
- `on_new_head` walks back the new and old chains, returning `NewCanonicalChain::Commit` or `Reorg`.```708:802:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_new_head(&self, new_head: B256) -> ProviderResult<Option<NewCanonicalChain<N>>> { ... }
```
- Canonical update sets the canonical head, reinserts reorged blocks, updates canonical in-memory state, and emits `CanonicalChainCommitted`.```2320:2353:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_canonical_chain_update(&mut self, chain_update: NewCanonicalChain<N>) { ... }
```
```2364:2375:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn reinsert_reorged_blocks(&mut self, new_chain: Vec<ExecutedBlock<N>>) { ... }
```
- Disk reorg detection walks canonical and persisted chains to a common ancestor and returns the block number to truncate above.```2271:2314:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn find_disk_reorg(&self) -> ProviderResult<Option<u64>> { ... }
```
- Persistence loop checks `find_disk_reorg` and triggers `remove_blocks`; removal sends `PersistenceAction::RemoveBlocksAbove`.```1280:1292:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn advance_persistence(&mut self) -> Result<(), AdvancePersistenceError> { ... }
```
```1246:1255:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn remove_blocks(&mut self, new_tip_num: u64) { ... }
```
- Persistence service deletes above a target by calling `remove_block_and_execution_above`, then commits.```123:137:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/persistence.rs
fn on_remove_blocks_above(&self, new_tip_num: u64) -> Result<Option<BlockNumHash>, PersistenceError> { ... }
```
- Storage provider removal unwinds trie/state, removes blocks above, and updates pipeline stages; static file pruning happens in `remove_blocks_above`.```3095:3106:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/provider.rs
fn remove_block_and_execution_above(&self, block: BlockNumber) -> ProviderResult<()> { ... }
```
```3200:3253:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/provider.rs
fn remove_blocks_above(&self, block: BlockNumber) -> ProviderResult<()> { ... }
```
- Backfill unwind marks invalid headers and resets the tree to the unwind target.```1558:1592:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
if let ControlFlow::Unwind { bad_block, target } = &ctrl { ... }
...
if ctrl.is_unwind() { self.state.tree_state.reset(backfill_num_hash) }
```
- `TreeState::reset` replaces the state with an empty tree pointing at the provided head.```55:57:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/state.rs
pub(crate) fn reset(&mut self, current_canonical_head: BlockNumHash) {
    *self = Self::new(current_canonical_head, self.engine_kind);
}
```
- After persistence completes, the engine calls `on_new_persisted_block` to reconcile in-memory state.```1823:1836:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/mod.rs
fn on_new_persisted_block(&mut self) -> ProviderResult<()> { ... }
```
- Old forks below finalized are pruned by `prune_finalized_sidechains`.```197:247:/Users/vicnaum/github/stateless-history-node/reth/crates/engine/tree/src/tree/state.rs
pub(crate) fn prune_finalized_sidechains(&mut self, finalized_num_hash: BlockNumHash) { ... }
```

## Corrections / Caveats
- `TreeState` does not keep all executed blocks indefinitely; pruning and backfill cleanup remove canonical blocks and old sidechains.
- Reorg triggering is not explicitly "new head is not a direct child"; `on_new_head` computes commit vs reorg based on chain connectivity and head position.
- Details of what is removed (state vs static files) live in storage provider code; the engine pack only shows the call sites.

## Actionable Pointers
- In-memory reorg algorithm: `reth/crates/engine/tree/src/tree/mod.rs` (`on_new_head`, `on_canonical_chain_update`, `reinsert_reorged_blocks`).
- Disk reorg detection and removal: `reth/crates/engine/tree/src/tree/mod.rs` (`find_disk_reorg`, `advance_persistence`, `remove_blocks`).
- Persistence deletion path: `reth/crates/engine/tree/src/persistence.rs` and `reth/crates/storage/provider/src/providers/database/provider.rs`.
