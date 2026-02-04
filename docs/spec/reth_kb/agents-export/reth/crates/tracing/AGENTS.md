# tracing

## Purpose
`reth-tracing` crate: logging/tracing configuration, formatters, and layer builders.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Tracer config, layer builders, formatter, and throttle macro.

### Files
- `Cargo.toml` - Manifest for tracing dependencies and optional OTLP/Tracy support.
  - **Key items**: features `otlp`, `otlp-logs`, `tracy`
