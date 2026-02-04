# src

## Purpose
Defines `reth-config` core configuration types (pipeline stages, pruning, peers/sessions, static files) and, when the `serde` feature is enabled, TOML load/save helpers for node/CLI configuration files.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: wires the `config` module and re-exports the primary configuration types.
- **Key items**: `config` module, `Config`, `PruneConfig`, `BodiesConfig`
- **Interactions**: re-export surface consumed by `reth-node-core`/CLI crates so they don't import deep module paths.
- **Knobs / invariants**: none (pure wiring/re-exports).

#### `config.rs`
- **Role**: The actual configuration schema for reth: a top-level `Config` object with nested per-subsystem settings (stages, pruning, peers/sessions, static files) and many stage-specific config structs. Under `serde`, includes TOML read/write helpers and duration parsing.
- **Key items**: `Config`, `StageConfig`, `EraConfig`, `HeadersConfig`, `BodiesConfig`, `ExecutionConfig`, `EtlConfig`, `PruneConfig`, `StaticFilesConfig`, `DEFAULT_BLOCK_INTERVAL`
- **Interactions**: converts/feeds stage-related thresholds into pipeline code (e.g. `ExecutionConfig` -> `ExecutionStageThresholds`); composes external config types like `PeersConfig`, `SessionsConfig`, `PruneModes`, `StaticFileMap`.
- **Knobs / invariants**: `Config::from_path()` creates a default file if missing (TOML, `serde` feature); `Config::save()` enforces `.toml` extension.

## Key APIs (no snippets)
- **Top-level**: `Config`
- **Stage grouping**: `StageConfig`
- **Stage configs**: `EraConfig`, `HeadersConfig`, `BodiesConfig`, `SenderRecoveryConfig`, `ExecutionConfig`, `PruneStageConfig`, `HashingConfig`, `MerkleConfig`, `TransactionLookupConfig`, `IndexHistoryConfig`, `EtlConfig`
- **Pruning/static files**: `PruneConfig`, `StaticFilesConfig`, `BlocksPerFileConfig`

## Relationships
- **Used by**: node configuration plumbing (node/CLI loads config and passes stage/prune/static-file knobs into pipeline and storage).
- **Depends on**: `reth-*-types` crates for value-object config types (`reth-network-types`, `reth-stages-types`, `reth-prune-types`, `reth-static-file-types`).

## End-to-end flow (high level)
- Construct `Config` (default or from TOML when `serde` is enabled).
- Read nested knobs for stages/pruning/peers/sessions/static files.
- Convert specific config parts into execution thresholds or stage configs (e.g. `ExecutionConfig` -> `ExecutionStageThresholds`).
- Downstream subsystems consume those settings to configure pipeline behavior, pruning strategy, and storage layout.
