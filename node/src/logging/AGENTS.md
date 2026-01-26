# logging

## Purpose
Logging, reporting, and resource monitoring module. Provides structured JSON logging with async file
writing, run reports for benchmarking and diagnostics, and resource monitoring for CPU, memory, and
disk I/O. Also handles SIGUSR1 signal for on-demand state dumps.

## Contents (one hop)
### Files
- `mod.rs` - Public exports and `init_tracing()` for setting up the tracing subscriber.
  - **Key items**: `TracingGuards`, `init_tracing()`
- `json.rs` - JSON log writer and tracing layer for structured logging to files.
  - **Key items**: `LogRecord`, `JsonLogWriter`, `JsonLogLayer`, `JsonLogFilter`, `LOG_BUFFER`
- `report.rs` - Run report types and generation for benchmarking runs.
  - **Key items**: `RunReport`, `RunContext`, `PeerHealthSummary`, `generate_run_report()`, `finalize_log_files()`, `run_timestamp_utc()`
- `resources.rs` - Resource monitoring with platform-specific implementations.
  - **Key items**: `spawn_resource_logger()`, `spawn_usr1_state_logger()`

## Key APIs (no snippets)
- **Types**: `TracingGuards`, `RunContext`, `RunReport`, `LogRecord`, `JsonLogWriter`, `JsonLogLayer`
- **Functions**: `init_tracing()`, `generate_run_report()`, `finalize_log_files()`, `spawn_resource_logger()`, `spawn_usr1_state_logger()`

## Resource Monitoring
Platform-specific implementations:
- **Linux**: Reads `/proc/self/status`, `/proc/stat`, `/proc/diskstats` for detailed memory breakdown (RSS, anon, file, shmem), CPU usage with iowait, and disk I/O rates.
- **Non-Linux** (macOS, etc.): Uses `sysinfo` crate for cross-platform metrics with reduced granularity.

Both emit `BenchEvent::ResourcesSample` events and `debug!("resources", ...)` log entries every second.

## SIGUSR1 Handler (Unix only)
`spawn_usr1_state_logger()` installs a signal handler that dumps:
1. Current sync progress stats (status, queue, inflight, peers)
2. Peer pool summary (connected, banned, available)
3. Top 3 and worst 3 peers by quality score

Usage: `kill -USR1 <pid>`

## Relationships
- **Used by**: `node/src/main.rs` for tracing initialization, resource logging, and report generation.
- **Depends on**: `sync::SyncProgressStats`, `sync::historical::BenchEventLogger`, `sync::historical::PeerHealthTracker`, `p2p::PeerPool`.
- **Emits**: JSON log files (`.logs.jsonl`), resource logs (`.resources.jsonl`), run reports (`.json`), Chrome traces (`.trace.json`).
