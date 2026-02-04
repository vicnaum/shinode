# src

## Purpose
Implements `reth-cli-runner`: a tokio-based runner for executing reth CLI commands with task management and graceful shutdown on `SIGINT`/`SIGTERM`.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - `CliRunner` + `CliContext` and helpers for running async / blocking commands until completion or ctrl-c, with `TaskManager`-backed shutdown.

## Key APIs (no snippets)
- **Types**: `CliRunner`, `CliContext`, `CliRunnerConfig`
- **Functions**: `tokio_runtime()`
- **Methods**: `CliRunner::try_default_runtime()`, `CliRunner::run_command_until_exit()`, `CliRunner::run_blocking_command_until_exit()`

## Relationships
- **Depends on**: `reth-tasks` (task manager/executor), `tokio` (runtime + signals).
