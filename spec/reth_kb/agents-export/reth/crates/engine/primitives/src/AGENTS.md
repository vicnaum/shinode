# src

## Purpose
Defines the shared Engine API surface types for reth's engine subsystem: request/response message wrappers, forkchoice tracking, engine events, config knobs, and extension hooks used by the engine-tree and engine service.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `config.rs` - `TreeConfig` and related constants for engine-tree tuning (buffers, caching, workers, proof options).
- `error.rs` - Engine API-facing error types (`BeaconOnNewPayloadError`, `BeaconForkChoiceUpdateError`).
- `event.rs` - `ConsensusEngineEvent`: events emitted by the consensus/engine layer (forkchoice, block added/received, invalid block).
- `forkchoice.rs` - `ForkchoiceStateTracker` + `ForkchoiceStatus` and helpers for tracking latest/valid/syncing forkchoice states.
- `invalid_block_hook.rs` - `InvalidBlockHook` trait and helpers (`NoopInvalidBlockHook`, `InvalidBlockHooks`).
- `lib.rs` - crate module wiring and core traits (`EngineTypes`, `EngineApiValidator`, `PayloadValidator`) + re-exports.
- `message.rs` - `BeaconEngineMessage`, `ConsensusEngineHandle`, `OnForkChoiceUpdated` future used to handle/await FCU outcomes.

## Key APIs (no snippets)
- **Traits**: `EngineTypes`, `EngineApiValidator`, `PayloadValidator`, `InvalidBlockHook`
- **Types**: `BeaconEngineMessage<T>`, `ConsensusEngineHandle<T>`, `ConsensusEngineEvent<N>`, `ForkchoiceStateTracker`, `ForkchoiceStatus`, `TreeConfig`
- **Helpers**: `InvalidBlockHooks`, `NoopInvalidBlockHook`, `ForkchoiceStateHash`

## Relationships
- **Used by**: `reth-engine-tree` (engine handler/event plumbing + config + forkchoice tracking), `reth-engine-service` (service stream emits `ConsensusEngineEvent`), and Engine API RPC wiring (via `ConsensusEngineHandle` / `BeaconEngineMessage` stream).
- **Depends on**: `reth-payload-primitives` (payload + versioned Engine API types), `reth-primitives-traits` (block/header abstractions), `alloy-rpc-types-engine` (Engine API data types).

## Notes
- `TreeConfig` lives here (engine primitives) even though it configures engine-tree; it's treated as part of the public engine surface.
