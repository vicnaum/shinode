# src

## Purpose
Implements `reth-ethereum-cli`: the Ethereum-flavoured `reth` CLI surface, including the top-level `Cli` parser/runner, chain-spec parsing for Ethereum networks, and the glue that executes `reth-cli-commands` using Ethereum node components.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Module wiring + public re-exports.
- **Key items**: `CliApp`, `ExtendedCommand`, `Cli`, `Commands`, `NoSubCmd`
- **Interactions**: consumers generally use `Cli::parse_args()` / `Cli::run()` from `interface`.

#### `interface.rs`
- **Role**: Defines the main clap `Cli` type and helpers to run commands with tracing/metrics initialized, plus methods to run with custom runners and/or component builders.
- **Key items**: `Cli<...>`, `Cli::run()`, `Cli::run_with_components()`, `Cli::with_runner()`, `Cli::with_runner_and_components()`, `Cli::init_tracing()`
- **Interactions**: dispatches to `reth_cli_commands` command implementations; uses `reth_node_builder` to launch Ethereum node types.

#### `app.rs`
- **Role**: Defines `CliApp` wrapper (parsed CLI + optional runner/tracing layers) and the shared `run_commands_with()` dispatch implementation used by both `CliApp` and `Cli` methods.
- **Key items**: `CliApp`, `CliApp::run()`, `CliApp::run_with_components()`, `run_commands_with()`, `ExtendedCommand`
- **Interactions**: installs Prometheus recorder; validates RPC module selections via `RpcModuleValidator`; executes each `Commands` variant on the configured `CliRunner`.

#### `chainspec.rs`
- **Role**: Ethereum chain spec parser for clap args: maps known chain names to embedded specs (mainnet/sepolia/holesky/hoodi/dev) or parses a genesis JSON (file path or inline JSON).
- **Key items**: `SUPPORTED_CHAINS`, `chain_value_parser()`, `EthereumChainSpecParser`
- **Interactions**: used by `reth_cli_commands::NodeCommand` parsing and other CLI flows that need `ChainSpecParser`.

## Key APIs (no snippets)
- **CLI**: `Cli`, `Commands`
- **Runner wrapper**: `CliApp`, `run_commands_with()`
- **Chain parsing**: `EthereumChainSpecParser`, `chain_value_parser()`

## Relationships
- **Builds on**: generic CLI crates (`reth-cli`, `reth-cli-commands`, `reth-cli-runner`) plus Ethereum node wiring (`reth-node-ethereum`).
