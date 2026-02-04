# src

## Purpose
Engine API implementation for consensus-layer interaction (payloads, forkchoice, blobs).

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Crate entrypoint and re-exports for engine API implementation.
- **Key items**: `EngineApi`, `EngineApiSender`, `EngineCapabilities`, `EngineApiError`

### `capabilities.rs`
- **Role**: Supported engine capabilities list and management.
- **Key items**: `CAPABILITIES`, `EngineCapabilities`, `list()`, `add_capability()`, `remove_capability()`

### `engine_api.rs`
- **Role**: Core Engine API handler implementation and methods.
- **Key items**: `EngineApi`, `EngineApiSender`, `new_payload_v1()`, `fork_choice_updated_v3()`, `get_payload_v5()`
- **Knobs / invariants**: Enforces payload/body/blob request limits.

### `error.rs`
- **Role**: Engine API error types and RPC error mapping.
- **Key items**: `EngineApiError`, `EngineApiResult`, `INVALID_PAYLOAD_ATTRIBUTES`, `REQUEST_TOO_LARGE_CODE`

### `metrics.rs`
- **Role**: Metrics for engine API latency and blob request counts.
- **Key items**: `EngineApiMetrics`, `EngineApiLatencyMetrics`, `BlobMetrics`

## End-to-end flow (high level)
- Construct `EngineApi` with provider, payload store, and consensus handle.
- Validate payloads and forkchoice updates per engine spec versions.
- Serve payload retrieval and blob queries with size limits.
- Emit metrics for latency and blob cache hits/misses.

## Key APIs (no snippets)
- `EngineApi`, `EngineApiError`, `EngineCapabilities`
