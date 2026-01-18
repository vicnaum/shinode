# stage

## Purpose
Implements `reth stage` CLI tooling for pipeline/stage debugging: run a single stage over a block range, unwind stages/pipeline state, dump stage data into a new DB, or drop stage-related data from the DB/static files.

## Contents (one hop)
### Subdirectories
- [x] `dump/` - Stage "dump" utilities that export stage data for a range into a new database.

### Files
- `mod.rs` - `reth stage` command dispatcher and chain-spec plumbing.
  - **Key items**: `Command<C>`, `Subcommands<C>`
- `run.rs` - run a single stage over a range (optionally with metrics and network-backed downloaders for headers/bodies stages).
  - **Key items**: `Command<C>`, `StageEnum`, `ExecutionStageThresholds`, `HeaderStage`, `BodyStage`
- `drop.rs` - "drop stage" tooling: clears stage tables/checkpoints and coordinates pruning of static-file segments when applicable.
  - **Key items**: `Command<C>`, `StageEnum`, `reset_stage_checkpoint()`, `reset_prune_checkpoint()`, `StaticFileSegment`
- `unwind.rs` - pipeline unwind command (supports an "offline" unwind mode that avoids headers/bodies/senders).
  - **Key items**: `Command<C>`, `Subcommands`, `Pipeline`, `DefaultStages`, `OfflineStages`

## Key APIs (no snippets)
- **CLI types**: `stage::Command<C>`, `stage::Subcommands<C>`
- **Stage commands**: `run::Command<C>`, `drop::Command<C>`, `unwind::Command<C>`, `dump::Command<C>`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` via the `stage` module (as part of the `reth` CLI command suite).
- **Depends on**: `reth-stages` (stage implementations/pipeline), `reth-db*` + `reth-db-common` (DB operations/checkpoints), `reth-static-file` (static file producer) and network/downloaders for online stages.
