# commands

## Purpose
`reth-cli-commands` crate: provides the concrete `reth ...` command implementations (node launch, DB tooling, stage/pipeline debugging, import/export, etc.) that are shared by reth-based binaries.

## Contents (one hop)
### Subdirectories
- [x] `src/` - command implementations and shared environment/bootstrap helpers.

### Files
- `Cargo.toml` - crate manifest for `reth-cli-commands` (feature flags and broad subsystem dependencies).
  - **Key items**: `features.arbitrary`, `features.edge`

## Key APIs (no snippets)
- **Command entrypoints**: `NodeCommand`, `db::Command`, `stage::Command`, `ImportCommand`, `ImportEraCommand`, `ExportEraCommand`, `DownloadCommand`

## Relationships
- **Used by**: higher-level CLI crates/binaries that want the standard reth command set.
- **Depends on**: most core reth subsystems (DB/provider/static files/stages/network/evm) to implement operational commands.
