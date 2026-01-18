# metrics

## Purpose
`reth-node-metrics` crate: Prometheus recorder utilities and a metrics HTTP server for node metrics.

## Contents (one hop)
### Subdirectories
- [x] `src/` - Metrics recorder setup, hooks, and HTTP server.

### Files
- `Cargo.toml` - Manifest for metrics recorder/server and optional jemalloc/profiling features.
  - **Key items**: features `jemalloc`, `jemalloc-prof`; deps `metrics-exporter-prometheus`, `jsonrpsee-server`
