# src

## Purpose
OpenTelemetry OTLP tracing and logging layers for reth.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Builds OTLP tracing/logging layers and configuration types.
  - **Key items**: `span_layer()`, `log_layer()`, `OtlpConfig`, `OtlpLogsConfig`, `OtlpProtocol`
  - **Knobs / invariants**: `sample_ratio` must be in `[0.0, 1.0]`; endpoint protocol `http`/`grpc`.

## Key APIs (no snippets)
- `span_layer()`, `log_layer()`
- `OtlpConfig`, `OtlpLogsConfig`
