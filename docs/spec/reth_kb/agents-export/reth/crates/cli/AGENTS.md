# cli

## Purpose
CLI subsystem crates for reth: shared abstractions for building CLIs (`reth-cli`), the concrete command suite (`reth-cli-commands`), a tokio-based execution runner (`reth-cli-runner`), and small shared parsing/runtime utilities (`reth-cli-util`).

## Contents (one hop)
### Subdirectories
- [x] `cli/` - `reth-cli`: shared CLI traits and chainspec parsing helpers used by reth-based CLIs.
- [x] `commands/` - `reth-cli-commands`: implementations of `reth ...` subcommands (node/db/stage/import/export/etc.).
- [x] `runner/` - `reth-cli-runner`: tokio runtime + graceful shutdown wrapper for CLI command execution.
- [x] `util/` - `reth-cli-util`: reusable CLI helpers (value parsers, allocator/cancellation helpers, secret key loading).

### Files
- (none)

## Key APIs (no snippets)
- **Traits**: `RethCli`, `ChainSpecParser`, `Launcher`
- **Runners**: `CliRunner`, `CliContext`
- **Command surfaces**: `NodeCommand`, `db::Command`, `stage::Command`

## Relationships
- **Used by**: top-level reth binaries/CLIs (e.g. `reth`, `op-reth`) to share consistent argument parsing and command execution patterns.
- **Connects**: CLI argument parsing (`clap`) <-> node configuration (`reth-node-core`/`reth-node-builder`) <-> runtime/task management (`reth-tasks`).
