# metrics

## Purpose
Sync pipeline metric events and listeners that update per-stage gauges.

## Contents (one hop)
### Subdirectories
- (none)

## Files (detailed)

### `mod.rs`
- **Role**: Module wiring and exports for metric events and listener.
- **Key items**: `MetricEvent`, `MetricEventsSender`, `MetricsListener`

### `listener.rs`
- **Role**: Event listener that updates metrics on incoming `MetricEvent`s.
- **Key items**: `MetricEvent`, `MetricsListener`
- **Interactions**: Updates `SyncMetrics` and per-stage gauges.

### `sync_metrics.rs`
- **Role**: Metrics definitions for stage progress.
- **Key items**: `SyncMetrics`, `StageMetrics`
- **Knobs / invariants**: Metrics scoped under `sync`.

## End-to-end flow (high level)
- Stage/pipeline code sends `MetricEvent` over an unbounded channel.
- `MetricsListener` polls the channel and updates per-stage metrics.
- `SyncMetrics` lazily creates per-stage `StageMetrics` gauges.

## Key APIs (no snippets)
- `MetricEvent`, `MetricEventsSender`, `MetricsListener`
- `SyncMetrics`, `StageMetrics`
