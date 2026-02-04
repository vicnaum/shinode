# src

## Purpose
Tracing configuration helpers and layer builders for reth logging.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Main tracer configuration API and re-exports.
  - **Key items**: `RethTracer`, `LayerInfo`, `Tracer`, `LogFormat`
- `layers.rs` - Layer builders for stdout, file, journald, samply, and OTLP.
  - **Key items**: `Layers`, `FileInfo`, `FileWorkerGuard`
  - **Knobs / invariants**: Default filter directives suppress noisy deps.
- `formatter.rs` - Log format enum and layer builder for JSON/logfmt/terminal.
  - **Key items**: `LogFormat`, `apply()`
- `throttle.rs` - Throttling macro for rate-limited logging.
  - **Key items**: `throttle!`, `should_run()`
- `test_tracer.rs` - Minimal tracer for tests.
  - **Key items**: `TestTracer`

## Key APIs (no snippets)
- `RethTracer`, `LayerInfo`, `Layers`
- `LogFormat`, `throttle!`
