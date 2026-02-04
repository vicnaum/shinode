# src

## Purpose
Implements `reth-cli-commands`: a shared collection of concrete `reth ...` CLI subcommands (node launch, import/export, DB tools, stage/pipeline tooling, network/P2P debug, etc.) plus common environment/bootstrap helpers used by reth-based binaries.

## Contents (one hop)
### Subdirectories
- [x] `db/` - `reth db` command suite: DB/static-file inspection, diff/checksum/clear, trie verification/repair, and a TUI for browsing tables.
- [x] `init_state/` - `reth init-state`: initialize a datadir from a state dump (including `--without-evm` helper path).
- [x] `p2p/` - `reth p2p`: P2P/network debugging commands (fetching, RLPx, bootnode/discovery).
- [x] `stage/` - `reth stage`: stage/pipeline debugging and maintenance (run/unwind/drop/dump).
- [x] `test_vectors/` - (feature-gated) `reth test-vectors`: generate micro vectors for DB tables and codec/Compact types.

### Files
- `common.rs` - shared CLI environment/bootstrap used by multiple commands: config+datadir resolution, DB/static-file opening, consistency checks/healing, and common CLI node type bounds.
  - **Key items**: `EnvironmentArgs<C>`, `Environment<N>`, `AccessRights`, `CliNodeTypes`
- `config_cmd.rs` - `reth config`: prints either a specified config file or the default config as TOML.
  - **Key items**: `config_cmd::Command`, `Command::execute()`
- `download.rs` - `reth download`: snapshot download + streaming extraction (with configurable default snapshot sources).
  - **Key items**: `DownloadCommand<C>`, `DownloadDefaults`, `stream_and_extract()`, `DownloadProgress`
- `dump_genesis.rs` - `reth dump-genesis`: prints the selected chain's genesis JSON.
  - **Key items**: `DumpGenesisCommand<C>`
- `export_era.rs` - `reth export-era`: exports blocks from the local DB into ERA1 files.
  - **Key items**: `ExportEraCommand<C>`, `ExportArgs`, `ERA1_EXPORT_FOLDER_NAME`
- `import.rs` - `reth import`: imports RLP-encoded blocks from one or more files, running the pipeline after file-backed header/body ingestion.
  - **Key items**: `ImportCommand<C>`, `ImportConfig`, `import_blocks_from_file()`, `build_import_pipeline`
- `import_core.rs` - core import logic without clap dependencies (shared by CLI and tests).
  - **Key items**: `ImportConfig`, `ImportResult`, `import_blocks_from_file()`, `build_import_pipeline_impl()`
- `import_era.rs` - `reth import-era`: imports ERA1 blocks from a local directory or remote host (HTTP), starting from the current static-file tip.
  - **Key items**: `ImportEraCommand<C>`, `ImportArgs`, `EraStream`, `EraStreamConfig`
- `init_cmd.rs` - `reth init`: initializes the database with the genesis block.
  - **Key items**: `InitCommand<C>`
- `launcher.rs` - node launch abstraction: trait-based launcher interface and a closure adapter for backward-compatible wiring.
  - **Key items**: `Launcher<C, Ext>`, `FnLauncher<F>`
- `lib.rs` - crate module wiring and re-exports.
  - **Key items**: module list; `pub use node::NodeCommand`
- `node.rs` - `reth node`: main node command (argument surface for network/rpc/txpool/pruning/etc.) and the generic launch entrypoint.
  - **Key items**: `NodeCommand<C, Ext>`, `NodeCommand::execute()`, `NoArgs`
- `prune.rs` - `reth prune`: runs configured pruning without enforced limits (after copying eligible DB data into static files).
  - **Key items**: `PruneCommand<C>`, `PrunerBuilder`, `StaticFileProducer`
- `re_execute.rs` - `reth re-execute`: parallel re-execution of historical blocks to validate sync correctness.
  - **Key items**: `re_execute::Command<C>`, `CancellationToken`, `JoinSet`, `Executor`

## Key APIs (no snippets)
- **Environment/bootstrap**: `EnvironmentArgs<C>`, `Environment<N>`, `AccessRights`
- **Node launch**: `NodeCommand<C, Ext>`, `Launcher<C, Ext>`, `FnLauncher<F>`
- **Data movement**: `ImportCommand<C>`, `ImportEraCommand<C>`, `ExportEraCommand<C>`, `DownloadCommand<C>`
- **Maintenance/debug**: `db::Command<C>`, `stage::Command<C>`, `PruneCommand<C>`, `re_execute::Command<C>`

## Relationships
- **Used by**: reth-based CLI binaries (e.g. `reth-ethereum-cli` and downstream CLIs) that want a standard set of subcommands.
- **Depends on**: `reth-cli` (chainspec parsing), `reth-cli-runner`/`reth-cli-util` (runner and helpers), and many core reth subsystems (DB/provider/static files/stages/network/evm).

## Notes
- `test_vectors` is behind the crate `arbitrary` feature; the directory exists in-tree but is not always compiled.
