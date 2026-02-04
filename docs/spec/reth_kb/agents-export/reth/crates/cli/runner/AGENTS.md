# runner

## Purpose
`reth-cli-runner` crate: provides a tokio-based runner for reth CLI commands, integrating `reth-tasks` for task lifecycle management and graceful shutdown on exit signals.

## Contents (one hop)
### Subdirectories
- [x] `src/` - `CliRunner` implementation and helpers.

### Files
- `Cargo.toml` - crate manifest.

## Key APIs (no snippets)
- **Types**: `CliRunner`, `CliContext`, `CliRunnerConfig`
- **Functions**: `tokio_runtime()`

## Relationships
- **Used by**: higher-level CLI crates to execute commands consistently (async or blocking) and to ensure spawned tasks are shut down cleanly.
