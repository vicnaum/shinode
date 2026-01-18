# Q011 - Pipeline Orchestration and Checkpoints

Status: verified

## Question
How does the pipeline orchestrate stage execution and checkpoints?

## Context Packs
- `spec/reth_repo_context/context-sync.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-sync.xml

Question:
How does the pipeline orchestrate stage execution and checkpoints?
I want the flow for execute/unwind, checkpoint persistence, and how stages are scheduled/ordered.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Identify the stage entry points and where checkpoints are read/written.
3. Explain execute vs unwind control flow and how progress is persisted.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Lifecycle flow (step-by-step)
4. Key abstractions/types (with file paths)
5. “Where to look next”
6. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-sync.xml` pack, here is the architectural map of how the Reth pipeline orchestrates execution and checkpoints.

### 1. Summary
*   **Serial Execution Loop**: The pipeline runs stages sequentially (e.g., Headers → Bodies → Execution) in a loop. The output block number of Stage N becomes the target block number for Stage N+1.
*   **Checkpoint-Driven**: Every stage persists a `StageCheckpoint` (block number + stage-specific metrics) to the database upon completing a batch. If the node restarts, it resumes from these checkpoints.
*   **Two-Phase Execution**: Stages implement `poll_execute_ready` (for async work like downloading) and `execute` (for synchronous database writing).
*   **Reverse Unwind**: When a reorg is detected or a validation error occurs, the pipeline pauses execution and iterates through the stages in *reverse order*, calling `unwind` to rollback database changes.
*   **Transaction granularity**: Database commits happen after every successful stage execution batch, ensuring that a stage doesn't save a checkpoint without saving the data associated with it.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/stages/api/src/pipeline/mod.rs` | `Pipeline::run` | The top-level entry point that starts the infinite sync loop. |
| `crates/stages/api/src/pipeline/mod.rs` | `Pipeline::run_loop` | The logic that iterates through the list of stages once. |
| `crates/stages/api/src/pipeline/builder.rs` | `PipelineBuilder` | Constructs the pipeline, defining the order of stages. |
| `crates/stages/api/src/stage.rs` | `trait Stage` | The interface every stage (Headers, Bodies, Execution) must implement. |
| `crates/stages/stages/src/sets.rs` | `DefaultStages` | Defines the standard order of stages for a full node (Headers -> ... -> Finish). |

---

### 3. Lifecycle Flow

#### A. Pipeline Construction
The pipeline is built via `PipelineBuilder`. The order is determined by the order in which `add_stage` is called.
*   **Source**: `crates/stages/stages/src/sets.rs` (See `DefaultStages::builder`).
*   **Standard Order**: `Headers` -> `Bodies` -> `SenderRecovery` -> `Execution` -> `Merkle` -> `Hashing` -> `History` -> `Finish`.

#### B. The Run Loop (`Pipeline::run_loop`)
Located in `crates/stages/api/src/pipeline/mod.rs`, this function iterates through the registered stages list index `0..N`.

1.  **Determine Target**: For the first stage (Headers), the target is usually the network tip (communicated via `tip_tx` channel). For subsequent stages, the target is the block number reached by the *previous* stage.
2.  **Load Checkpoint**: `prev_checkpoint = provider.get_stage_checkpoint(stage_id)`.
3.  **Execute Stage**: Calls `self.execute_stage_to_completion`.

#### C. Stage Execution (`execute_stage_to_completion`)
Located in `crates/stages/api/src/pipeline/mod.rs`.

1.  **Check Readiness**: Calls `stage.poll_execute_ready(cx, input)`.
    *   This allows stages like `HeaderStage` or `BodyStage` to drive async downloaders and buffer data before the DB transaction opens.
2.  **Execute**: Calls `stage.execute(&provider_rw, input)`.
    *   This performs the actual logic (e.g., inserting headers, running EVM).
    *   Returns `ExecOutput { checkpoint, done }`.
3.  **Persist**:
    *   `provider_rw.save_stage_checkpoint(stage_id, checkpoint)` updates the progress.
    *   `provider_rw.commit()` commits the database transaction (data + checkpoint atomically).
4.  **Loop/Next**: If `done` is false, it repeats execution (for batched processing). If `done` is true, the pipeline moves to the next stage.

#### D. Unwind Flow (`Pipeline::unwind`)
Triggered if a stage returns `StageError::Validation`, `StageError::DetachedHead`, or `ControlFlow::Unwind`.

1.  **Reverse Iteration**: The pipeline iterates through stages in **reverse order** (from Finish back to Headers).
2.  **Unwind Logic**: Calls `stage.unwind(&provider_rw, input)`.
    *   The stage deletes data above the `unwind_to` block.
    *   Example: `ExecutionStage` deletes state changes and receipts.
3.  **Update Checkpoint**: The checkpoint is reset to the `unwind_to` block.
4.  **Commit**: The unwind is committed to the DB immediately.

---

### 4. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `ExecInput` | `crates/stages/api/src/stage.rs` | Input passed to `execute`. Contains `target` (block number to reach) and `checkpoint` (where we are now). |
| `ExecOutput` | `crates/stages/api/src/stage.rs` | Output from `execute`. Contains new `checkpoint` and `done` (bool). |
| `StageCheckpoint` | `crates/stages/types/src/checkpoints.rs` | Struct stored in DB. Contains `block_number` and optional stage-specific metrics (e.g., `entities_processed`). |
| `ControlFlow` | `crates/stages/api/src/pipeline/ctrl.rs` | Enum returned by the loop: `Continue`, `NoProgress`, or `Unwind`. |
| `StageId` | `crates/stages/types/src/id.rs` | Unique identifier for each stage (e.g., `StageId::Headers`), used as the DB key for checkpoints. |

---

### 5. Where to look next

1.  **`crates/stages/api/src/pipeline/mod.rs`**: Read `run_loop` and `execute_stage_to_completion` to see the exact sequence of locking the provider, executing, and committing.
2.  **`crates/stages/stages/src/sets.rs`**: Look at `DefaultStages` to see the authoritative ordering of stages for a standard node.
3.  **`crates/stages/stages/src/stages/headers.rs`**: A good example of a stage that implements `poll_execute_ready` (to download) and `execute` (to write).
4.  **`crates/stages/stages/src/stages/execution.rs`**: Look at how `execute` handles batching (using `thresholds`) and how `unwind` uses the provider to revert state.

---

### 6. Suggested Follow-up Questions
*   "How does the `HeaderStage` specifically use `poll_execute_ready` to bridge the gap between async network downloaders and synchronous database writes?"
*   "How are `PruneCheckpoints` handled differently from standard `StageCheckpoints` in the pipeline?"
*   "What happens to the `StaticFileProducer` during a pipeline unwind?"

## Verification
- `Pipeline::run` loops, calling `run_loop` repeatedly.```182:209:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
pub async fn run(&mut self) -> Result<(), PipelineError> {
    loop {
        let next_action = self.run_loop().await?;
        if next_action.is_unwind() && self.fail_on_unwind {
            return Err(PipelineError::UnexpectedUnwind)
        }
        ...
    }
}
```
- `run_loop` executes stages in order and uses the previous stage checkpoint as the next stage’s target.```223:256:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
for stage_index in 0..self.stages.len() {
    let stage_id = stage.id();
    let next = self.execute_stage_to_completion(previous_stage, stage_index).await?;
    ...
    previous_stage = Some(
        self.provider_factory
            .provider()?
            .get_stage_checkpoint(stage_id)?
            .unwrap_or_default()
            .block_number,
    );
}
```
```420:446:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
let target = self.max_block.or(previous_stage);
let exec_input = ExecInput { target, checkpoint: prev_checkpoint };
```
- `execute_stage_to_completion` calls `poll_execute_ready`, then `execute`, then saves checkpoint and commits.```458:486:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
if let Err(err) = self.stage(stage_index).execute_ready(exec_input).await { ... }
...
match self.stage(stage_index).execute(&provider_rw, exec_input) {
    Ok(out @ ExecOutput { checkpoint, done }) => {
        provider_rw.save_stage_checkpoint(stage_id, checkpoint)?;
        provider_rw.commit()?;
        self.stage(stage_index).post_execute_commit()?;
        ...
    }
}
```
- `unwind` iterates stages in reverse order and commits after each unwind batch.```311:395:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
let unwind_pipeline = self.stages.iter_mut().rev();
...
let output = stage.unwind(&provider_rw, input);
...
provider_rw.save_stage_checkpoint(stage_id, checkpoint)?;
...
provider_rw.commit()?;
stage.post_unwind_commit()?;
```
- `Stage` trait defines `poll_execute_ready`, `execute`, `unwind`, and post‑commit hooks.```241:306:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/stage.rs
pub trait Stage<Provider>: Send {
    fn id(&self) -> StageId;
    fn poll_execute_ready(&mut self, _cx: &mut Context<'_>, _input: ExecInput) -> Poll<Result<(), StageError>> { ... }
    fn execute(&mut self, provider: &Provider, input: ExecInput) -> Result<ExecOutput, StageError>;
    fn unwind(&mut self, provider: &Provider, input: UnwindInput) -> Result<UnwindOutput, StageError>;
    fn post_execute_commit(&mut self) -> Result<(), StageError> { ... }
    fn post_unwind_commit(&mut self) -> Result<(), StageError> { ... }
}
```
- Checkpoint and output types are in `stage.rs`/`checkpoints.rs`.```14:18:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/stage.rs
pub struct ExecInput { pub target: Option<BlockNumber>, pub checkpoint: Option<StageCheckpoint> }
```
```201:208:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/stage.rs
pub struct ExecOutput { pub checkpoint: StageCheckpoint, pub done: bool }
```
```375:379:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/types/src/checkpoints.rs
pub struct StageCheckpoint { pub block_number: BlockNumber, pub stage_checkpoint: Option<StageUnitCheckpoint> }
```
- `ControlFlow` describes Continue/NoProgress/Unwind.```4:28:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/ctrl.rs
pub enum ControlFlow { Unwind { ... }, Continue { ... }, NoProgress { ... } }
```
- Default stage order is assembled via stage sets: Online (Headers/Bodies, optionally Era) → ExecutionStages (SenderRecovery, Execution) → HashingStages (Merkle unwind, AccountHashing, StorageHashing, Merkle execute) → HistoryIndexingStages (TransactionLookup, IndexStorageHistory, IndexAccountHistory) → optional prune sender recovery → PruneStage → Finish.```139:179:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
Self::add_offline_stages(self.online.builder(), ...).add_stage(FinishStage)
```
```256:287:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
builder
    .add_stage(HeaderStage::new(...))
    .add_stage(BodyStage::new(...))
```
```384:393:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
StageSetBuilder::default()
    .add_stage(SenderRecoveryStage::new(...))
    .add_stage(ExecutionStage::from_config(...))
```
```416:431:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
StageSetBuilder::default()
    .add_stage(MerkleStage::default_unwind())
    .add_stage(AccountHashingStage::new(...))
    .add_stage(StorageHashingStage::new(...))
    .add_stage(MerkleStage::new_execution(...))
```
```450:466:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
StageSetBuilder::default()
    .add_stage(TransactionLookupStage::new(...))
    .add_stage(IndexStorageHistoryStage::new(...))
    .add_stage(IndexAccountHistoryStage::new(...))
```
```335:350:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
ExecutionStages::new(...).builder()
    .add_stage_opt(self.prune_modes.sender_recovery.map(|prune_mode| { ... }))
    .add_set(HashingStages { ... })
    .add_set(HistoryIndexingStages { ... })
    .add_stage(PruneStage::new(...))
```

## Corrections / Caveats
- The pipeline’s `ExecInput.target` is `max_block` or the *previous stage’s checkpoint*; the network tip is used by the headers stage itself (via the downloader), not directly by the pipeline target logic.
- Stage ordering can include optional `EraStage` and optional prune sender recovery; it’s not always a fixed linear list.
- Checkpoints are persisted per batch (loop in `execute_stage_to_completion`), not only once at stage end.

## Actionable Pointers
- Pipeline loop and commit/unwind: `reth/crates/stages/api/src/pipeline/mod.rs`.
- Stage contract: `reth/crates/stages/api/src/stage.rs`.
- Stage checkpoints: `reth/crates/stages/types/src/checkpoints.rs`.
- Default stage ordering: `reth/crates/stages/stages/src/sets.rs`.
