# src

## Purpose
Basic payload builder implementation that periodically builds payloads from the txpool with concurrency limits and cache reuse.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Core job generator and payload builder trait definitions plus job execution logic.
- **Key items**: `BasicPayloadJobGenerator`, `BasicPayloadJobGeneratorConfig`, `BasicPayloadJob`, `PayloadBuilder`, `PayloadConfig`, `BuildOutcome`
- **Interactions**: Uses `TaskSpawner` for build tasks, `CachedReads` for state reuse, and `CanonStateNotification` to seed pre-cached state.
- **Knobs / invariants**: `max_payload_tasks`, `interval`, `deadline`; `BuildOutcome::Freeze` stops further building.

### `better_payload_emitter.rs`
- **Role**: Wraps a `PayloadBuilder` to broadcast better/frozen payloads to subscribers.
- **Key items**: `BetterPayloadEmitter`
- **Interactions**: Emits payloads on a `tokio::sync::broadcast` channel.

### `metrics.rs`
- **Role**: Metrics for payload build attempts and empty responses.
- **Key items**: `PayloadBuilderMetrics`, `inc_requested_empty_payload()`, `inc_failed_payload_builds()`

### `stack.rs`
- **Role**: Builder composition utilities for chaining multiple builders.
- **Key items**: `PayloadBuilderStack`, `Either`
- **Interactions**: Implements `PayloadBuilderAttributes`/`BuiltPayload` for `Either` to allow fallback chains.

## End-to-end flow (high level)
- Configure a `BasicPayloadJobGenerator` with a `PayloadBuilder` and limits.
- Spawn `BasicPayloadJob` instances that periodically attempt payload builds.
- Use cached reads and best-payload tracking to avoid redundant work.
- Optionally wrap builders with `BetterPayloadEmitter` or `PayloadBuilderStack` for emission/fallback.

## Key APIs (no snippets)
- `BasicPayloadJobGenerator`, `BasicPayloadJob`
- `PayloadBuilder`, `BuildOutcome`, `PayloadConfig`
- `BetterPayloadEmitter`, `PayloadBuilderStack`
