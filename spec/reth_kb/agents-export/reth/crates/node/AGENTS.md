# node

## Purpose
Node-level crates: builder and core configuration utilities, node APIs/types, metrics/events, and optional EthStats reporting.

## Contents (one hop)
### Subdirectories
- [x] `api/` - Node component traits and add-on interfaces (`FullNodeComponents`, `NodeAddOns`).
- [x] `builder/` - Type-safe node builder, launchers, and RPC wiring (`NodeBuilder`, `EngineNodeLauncher`).
- [x] `core/` - Core node config, CLI args, paths, and version metadata (`NodeConfig`).
- [x] `ethstats/` - EthStats client service and protocol message types (`EthStatsService`).
- [x] `events/` - Node event handlers and CL health event stream (`ConsensusLayerHealthEvents`).
- [x] `metrics/` - Prometheus recorder and metrics server utilities (`MetricServer`).
- [x] `types/` - Node type traits and adapters (`NodeTypes`).

### Files
- (none)

## Notes
- This directory aggregates node-focused crates; details are in each subcrate's `AGENTS.md`.
