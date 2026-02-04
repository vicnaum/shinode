# src

## Purpose
Task management utilities: task spawners, manager, shutdown signals, and metrics.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `lib.rs` - Core task manager and spawner traits plus executor helpers.
  - **Key items**: `TaskSpawner`, `TokioTaskExecutor`, `TaskManager`, `TaskExecutor`
- `metrics.rs` - Task executor metrics counters and helpers.
  - **Key items**: `TaskExecutorMetrics`, `IncCounterOnDrop`
- `shutdown.rs` - Shutdown signaling primitives for graceful shutdown.
  - **Key items**: `GracefulShutdown`, `GracefulShutdownGuard`, `Shutdown`, `Signal`, `signal()`
- `pool.rs` - Rayon-based blocking task pool (feature `rayon`).
  - **Key items**: `BlockingTaskPool`, `BlockingTaskGuard`, `BlockingTaskHandle`

## Key APIs (no snippets)
- `TaskManager`, `TaskSpawner`, `TaskExecutor`
- `Shutdown`, `GracefulShutdown`
