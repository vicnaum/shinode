# src

## Purpose
Implements the `reth-engine-service` runtime wrapper that wires `reth-engine-tree` into a pollable service (stream), connecting consensus messages, on-demand downloads, pipeline backfill, and persistence.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - crate entrypoint; re-exports the `service` module.
- `service.rs` - `EngineService` definition and constructor that wires `EngineHandler` + `ChainOrchestrator` + `PipelineSync` + persistence.

## Key APIs (no snippets)
- **Types**: `EngineService<N, Client>`, `EngineMessageStream<T>`
- **Re-exports**: `ChainOrchestrator`, `ChainEvent`, `EngineApiEvent` (from `reth-engine-tree`)

## Relationships
- **Depends on**: `reth-engine-tree` (orchestrator/handler/tree/persistence), `reth-network-p2p` (block client), `reth-provider` (provider factory), `reth-payload-builder` (payload jobs), `reth-consensus` (validation), `reth-engine-primitives` (events/messages).
- **Used by**: node wiring (engine task/service that drives the chain forward from Engine API messages).
