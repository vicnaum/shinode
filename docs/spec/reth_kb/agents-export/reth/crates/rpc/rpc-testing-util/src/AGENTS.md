# src

## Purpose
Testing utilities and extension traits for RPC debug and trace clients.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `lib.rs`
- **Role**: Module wiring for RPC testing utilities.
- **Key items**: `debug`, `trace`, `utils`

### `debug.rs`
- **Role**: Debug API test helpers and stream utilities.
- **Key items**: `DebugApiExt`, `TraceTransactionResult`, `DebugTraceBlockResult`, `JsTracerBuilder`
- **Interactions**: Uses JS tracer templates from assets.

### `trace.rs`
- **Role**: Trace API stream helpers and result types.
- **Key items**: `TraceApiExt`, `TraceBlockResult`, `TraceCallStream`, `TraceFilterStream`

### `utils.rs`
- **Role**: Test utility helpers for RPC endpoint discovery.
- **Key items**: `parse_env_url()`

## End-to-end flow (high level)
- Build HTTP clients from env URLs.
- Use `DebugApiExt` and `TraceApiExt` to run RPC traces.
- Stream results and capture errors with typed wrappers.

## Key APIs (no snippets)
- `DebugApiExt`, `TraceApiExt`, `JsTracerBuilder`
