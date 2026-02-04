# src

## Purpose
Node metrics utilities: Prometheus recorder setup, metrics server, and metric hooks for chain/version info and system stats.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Module wiring and re-exports of Prometheus/process metrics crates.
  - **Key items**: `chain`, `hooks`, `recorder`, `server`, `version`
- `chain.rs` - Exposes chain-spec info as Prometheus metrics.
  - **Key items**: `ChainSpecInfo`, `register_chain_spec_metrics()`
- `hooks.rs` - Hook registry for periodic metric collectors (process, memory, IO).
  - **Key items**: `Hook`, `HooksBuilder`, `Hooks`
- `recorder.rs` - Installs Prometheus recorder and spawns upkeep for the global metrics recorder.
  - **Key items**: `PrometheusRecorder`, `install_prometheus_recorder()`, `spawn_upkeep()`
- `server.rs` - Metrics HTTP server and optional push gateway task.
  - **Key items**: `MetricServer`, `MetricServerConfig`, `serve()`
- `version.rs` - Exposes build/version info as Prometheus metrics.
  - **Key items**: `VersionInfo`, `register_version_metrics()`

## Key APIs (no snippets)
- `MetricServer`, `MetricServerConfig`
- `PrometheusRecorder`
