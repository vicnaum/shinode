# Q003 - Sync Pipeline Entry Points & Orchestration

Status: verified

## Question
Where are the sync pipeline entry points (stages, pipeline, downloaders), and how do they connect?

## Context Packs
- `spec/reth_repo_context/context-sync.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-sync.xml

Question:
Where are the sync pipeline entry points (stages, pipeline, downloaders), and how do they connect?
I want a map of the stage pipeline, how checkpoints/progress are tracked, and how downloaders are used.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Identify entry points (pipeline builder, stage sets, downloader interfaces).
3. Provide the lifecycle: pipeline init -> stage execution -> checkpoint update -> next stage.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Lifecycle flow (step-by-step)
4. Key abstractions/types (with file paths)
5. “Where to look next” (top files to read)
6. Suggested follow-up questions
```

## Gemini Answer
Based on the provided `context-sync.xml` file, here is a high-level architecture map of Reth’s synchronization pipeline, including entry points, stage logic, and data flow.

### 1. Summary
*   **Sequential Execution**: Reth syncs using a `Pipeline` that runs a series of `Stage` implementations serially, moving from one task (e.g., Headers) to the next (e.g., Bodies).
*   **Trait-Driven Design**: The `Stage` trait defines how a component "rolls forward" (`execute`) and "rolls back" (`unwind`).
*   **Checkpoint Persistence**: Each stage saves its progress as a `StageCheckpoint` in the database, allowing the node to resume after a restart.
*   **Decoupled Downloading**: Downloaders (for headers and bodies) are abstracted behind traits and often run as standalone tasks (`TaskDownloader`) to prevent blocking the main pipeline.
*   **Unified State Tracking**: The `ChainInfoTracker` and `InMemoryState` manage the latest canonical, safe, and finalized block information.
*   **Hybrid Storage**: Stages interact with both a database (MDBX) and "Static Files" (flat files for immutable historical data like headers and receipts).

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/stages/api/src/pipeline/builder.rs` | `PipelineBuilder` | The main UI for assembling a sync pipeline. It allows adding stages, setting a max block, and configuring tip listeners. |
| `crates/stages/stages/src/sets.rs` | `DefaultStages` | A pre-configured set of all stages required for a full sync (Headers -> Bodies -> Execution -> Merkle, etc.). |
| `crates/stages/api/src/pipeline/mod.rs` | `Pipeline::run` | The primary execution loop that drives the pipeline until the target block is reached. |
| `crates/net/downloaders/src/bodies/bodies.rs` | `BodiesDownloaderBuilder` | Entry point for configuring how block bodies are fetched from the network. |
| `crates/net/downloaders/src/headers/reverse_headers.rs` | `ReverseHeadersDownloaderBuilder` | Configures the downloader that fetches headers in reverse order (from tip to local head). |

---

### 3. Lifecycle Flow

#### Step 1: Initialization (`PipelineBuilder`)
The node starts by creating a `Pipeline` via the `PipelineBuilder`. It registers a `StaticFileProducer` and a `ProviderFactory` (which provides DB access).
*   **File**: `crates/stages/api/src/pipeline/builder.rs`
*   **Function**: `PipelineBuilder::build`

#### Step 2: Loop Initiation (`run_loop`)
The pipeline enters an infinite loop (or until a `max_block` is reached). It iterates through every registered stage in order.
*   **File**: `crates/stages/api/src/pipeline/mod.rs`
*   **Function**: `Pipeline::run_loop`

#### Step 3: Readiness Check (`poll_execute_ready`)
Before a stage executes, the pipeline calls `poll_execute_ready`. For downloader stages (like `HeaderStage`), this is when they fetch data from the network and fill internal buffers.
*   **File**: `crates/stages/api/src/stage.rs`
*   **Function**: `Stage::poll_execute_ready`

#### Step 4: Execution (`execute`)
The stage performs its logic (e.g., `ExecutionStage` runs the EVM, `SenderRecoveryStage` recovers public keys).
*   **File**: `crates/stages/api/src/stage.rs`
*   **Function**: `Stage::execute`

#### Step 5: Checkpoint & Commit
Once a stage completes a batch, it returns an `ExecOutput`. The pipeline then saves the new `StageCheckpoint` to the database and commits the transaction.
*   **File**: `crates/stages/api/src/pipeline/mod.rs`
*   **Function**: `Pipeline::execute_stage_to_completion`

#### Step 6: Control Flow (`ControlFlow`)
The result of a stage determines if the pipeline continues to the next stage, stops (no progress), or must **Unwind** due to a validation error.
*   **File**: `crates/stages/api/src/pipeline/ctrl.rs`
*   **Type**: `ControlFlow`

---

### 4. Key Abstractions/Types

*   **`Pipeline`** (`crates/stages/api/src/pipeline/mod.rs`): The orchestrator that owns the list of stages and manages transitions between them.
*   **`Stage`** (`crates/stages/api/src/stage.rs`): A trait that concrete sync tasks must implement. Note: Actual EVM execution logic (REVM) is used by `ExecutionStage` but defined in crates outside this pack.
*   **`StageCheckpoint`** (`crates/stages/types/src/checkpoints.rs`): A persistent record of how far a stage has progressed (block number + stage-specific metadata).
*   **`TaskDownloader`** (`crates/net/downloaders/src/headers/task.rs`): A wrapper that spawns a downloader onto its own Tokio task, communicating with the pipeline via channels to ensure the pipeline isn't blocked by P2P latency.
*   **`NewCanonicalChain`** (`crates/chain-state/src/in_memory.rs`): Represents a batch of executed blocks that are ready to be signaled as the new canonical head.
*   **`StaticFileSegment`** (`reth_static_file_types` - *referenced in* `crates/stages/stages/src/stages/utils.rs`): Represents different types of immutable data (Headers, Transactions, Receipts) stored outside the database.

---

### 5. Where to Look Next
1.  **`crates/stages/stages/src/stages/execution.rs`**: To see the bridge between the sync pipeline and the actual EVM execution.
2.  **`crates/stages/stages/src/stages/merkle.rs`**: To understand how the state root is calculated and verified at the end of execution.
3.  **`crates/stages/api/src/pipeline/mod.rs`**: To understand the `unwind` logic, which is critical for handling reorgs.
4.  **`docs/crates/stages.md`**: Provides a plain-English narrative of the ordering and purpose of every single stage.

---

### 6. Suggested Follow-up Questions
*   "How does the `HeaderStage` handle a `DetachedHead` error during reverse downloading?"
*   "What is the role of `external_clean_threshold` in the `ExecutionStage`'s decision to prune changesets?"
*   "How do `ExEx` (Execution Extensions) receive notifications from the `ExecutionStage` after a commit?"
*   "How does `MerkleChangeSets` maintain trie updates between the finalized block and the tip?"

## Verification
- Stage trait exposes `poll_execute_ready`, `execute`, and `unwind`; stages are executed serially via the pipeline.```229:299:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/stage.rs
/// Stages are executed as part of a pipeline where they are executed serially.
pub trait Stage<Provider>: Send {
    ...
    fn poll_execute_ready(
        &mut self,
        _cx: &mut Context<'_>,
        _input: ExecInput,
    ) -> Poll<Result<(), StageError>> { ... }
    fn execute(&mut self, provider: &Provider, input: ExecInput) -> Result<ExecOutput, StageError>;
    fn unwind(
        &mut self,
        provider: &Provider,
        input: UnwindInput,
    ) -> Result<UnwindOutput, StageError>;
}
```
- Pipeline run loop executes stages in order; it also calls `move_to_static_files()` each pass.```223:257:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
pub async fn run_loop(&mut self) -> Result<ControlFlow, PipelineError> {
    self.move_to_static_files()?;
    let mut previous_stage = None;
    for stage_index in 0..self.stages.len() {
        let stage = &self.stages[stage_index];
        let stage_id = stage.id();
        ...
        let next = self.execute_stage_to_completion(previous_stage, stage_index).await?;
        ...
    }
    Ok(self.progress.next_ctrl())
}
```
- Execution commits checkpoints and calls `post_execute_commit` after `save_stage_checkpoint`.```479:488:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/api/src/pipeline/mod.rs
match self.stage(stage_index).execute(&provider_rw, exec_input) {
    Ok(out @ ExecOutput { checkpoint, done }) => {
        // Update stage checkpoint.
        provider_rw.save_stage_checkpoint(stage_id, checkpoint)?;
        // Commit processed data to the database.
        provider_rw.commit()?;
        // Invoke stage post commit hook.
        self.stage(stage_index).post_execute_commit()?;
        ...
    }
```
- `StageCheckpoint` persists stage progress (block number + optional per‑stage metadata).```369:381:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/types/src/checkpoints.rs
pub struct StageCheckpoint {
    /// The maximum block processed by the stage.
    pub block_number: BlockNumber,
    /// Stage-specific checkpoint. None if stage uses only block-based checkpoints.
    pub stage_checkpoint: Option<StageUnitCheckpoint>,
}
```
- Downloaders can be run as `TaskDownloader` for both headers and bodies.```24:33:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/task.rs
/// A [HeaderDownloader] that drives a spawned [HeaderDownloader] on a spawned task.
pub struct TaskDownloader<H: Sealable> { ... }
```
```25:33:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/task.rs
/// A [BodyDownloader] that drives a spawned [BodyDownloader] on a spawned task.
pub struct TaskDownloader<B: Block> { ... }
```
- Builders for headers and bodies downloaders are present.```1103:1121:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/headers/reverse_headers.rs
/// The builder for [`ReverseHeadersDownloader`] with
pub struct ReverseHeadersDownloaderBuilder {
```
```508:523:/Users/vicnaum/github/stateless-history-node/reth/crates/net/downloaders/src/bodies/bodies.rs
/// Builder for [`BodiesDownloader`].
pub struct BodiesDownloaderBuilder {
    /// The batch size of non-empty blocks per one request
```
- Default stage set is defined and enumerates the full stage order.```60:83:/Users/vicnaum/github/stateless-history-node/reth/crates/stages/stages/src/sets.rs
/// A set containing all stages to run a fully syncing instance of reth.
/// This expands to the following series of stages:
/// - [`EraStage`] (optional, for ERA1 import)
/// - [`HeaderStage`]
/// - [`BodyStage`]
/// - [`SenderRecoveryStage`]
/// - [`ExecutionStage`]
/// ...
pub struct DefaultStages<Provider, H, B, E>
```
- Chain state tracking types exist in `chain-state` as described.```16:24:/Users/vicnaum/github/stateless-history-node/reth/crates/chain-state/src/chain_info.rs
/// Tracks the chain info: canonical head, safe block, finalized block.
pub struct ChainInfoTracker<N: NodePrimitives> { ... }
```
```55:67:/Users/vicnaum/github/stateless-history-node/reth/crates/chain-state/src/in_memory.rs
pub(crate) struct InMemoryState<N: NodePrimitives = EthPrimitives> { ... }
```

## Corrections / Caveats
- The claim about “stages interacting with static files” is supported in the pipeline via `move_to_static_files()` and via `StaticFileProviderFactory` usage in stage helpers, but the detailed static‑file flow is outside this question’s scope.

## Actionable Pointers
- Pipeline execution and unwind: `reth/crates/stages/api/src/pipeline/mod.rs`.
- Stage trait contract: `reth/crates/stages/api/src/stage.rs`.
- Default stage ordering: `reth/crates/stages/stages/src/sets.rs`.
- Downloaders: `reth/crates/net/downloaders/src/headers/reverse_headers.rs` and `.../bodies/bodies.rs`.
- Task-based downloader execution: `reth/crates/net/downloaders/src/headers/task.rs` and `.../bodies/task.rs`.
