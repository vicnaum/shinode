# src

## Purpose
Payload builder service abstractions: job traits, service orchestration, and handle APIs for engine integration.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate wiring and re-exports for payload builder service types and Ethereum payload primitives.
- **Key items**: `PayloadBuilderService`, `PayloadBuilderHandle`, `PayloadJob`, `PayloadJobGenerator`, `PayloadId`
- **Interactions**: Re-exports `EthBuiltPayload`/`EthPayloadBuilderAttributes` for convenience.

### `service.rs`
- **Role**: Core service that manages payload jobs and handles engine-facing commands.
- **Key items**: `PayloadBuilderService`, `PayloadBuilderHandle`, `PayloadStore`, `PayloadServiceCommand`
- **Interactions**: Drives `PayloadJob` futures and emits `PayloadEvents`.
- **Knobs / invariants**: Tracks active jobs, best/resolved payloads, and handles resolve semantics.

### `traits.rs`
- **Role**: Trait definitions for payload jobs and job generators.
- **Key items**: `PayloadJob`, `PayloadJobGenerator`, `KeepPayloadJobAlive`
- **Knobs / invariants**: `resolve_kind()` must return quickly; jobs must be cancel safe.

### `metrics.rs`
- **Role**: Metrics for payload builder service activity and revenues.
- **Key items**: `PayloadBuilderServiceMetrics`

### `noop.rs`
- **Role**: No-op payload builder service for configurations that disable payload building.
- **Key items**: `NoopPayloadBuilderService`, `PayloadBuilderHandle::noop()`

### `test_utils.rs`
- **Role**: Test helpers for creating and spawning a payload builder service.
- **Key items**: `test_payload_service()`, `spawn_test_payload_service()`, `TestPayloadJobGenerator`

## End-to-end flow (high level)
- Engine requests trigger `PayloadBuilderHandle` commands to the service.
- `PayloadBuilderService` creates a `PayloadJob` via the configured generator.
- The job is polled to produce better payloads until resolved or cancelled.
- Clients query best payloads or resolve with `PayloadKind` semantics.

## Key APIs (no snippets)
- `PayloadBuilderService`, `PayloadBuilderHandle`, `PayloadStore`
- `PayloadJob`, `PayloadJobGenerator`, `KeepPayloadJobAlive`
