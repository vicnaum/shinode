# src

## Purpose
Payload builder event primitives: broadcast event types and stream adapters.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring and public re-exports for payload events.
- **Key items**: `Events`, `PayloadEvents`, `PayloadBuilderError`
- **Interactions**: Re-exports definitions from `events.rs`.

### `events.rs`
- **Role**: Event enum and stream adapters for payload attribute and payload outputs.
- **Key items**: `Events`, `PayloadEvents`, `BuiltPayloadStream`, `PayloadAttributeStream`
- **Interactions**: Wraps `tokio::sync::broadcast` into `tokio_stream` adapters.

## End-to-end flow (high level)
- Payload builder service broadcasts `Events` through a channel.
- `PayloadEvents` wraps the receiver and offers typed stream adapters.
- Consumers subscribe to payload attributes or built payloads as streams.

## Key APIs (no snippets)
- `Events`, `PayloadEvents`
