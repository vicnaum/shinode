# logging

## Purpose
Logging, reporting, and resource monitoring module. Provides structured JSON logging with async file
writing, run reports for benchmarking and diagnostics, resource monitoring for CPU, memory, and
disk I/O, TUI log capture for dashboard display, and SIGUSR1 signal handler for on-demand state dumps.

## Contents (one hop)
### Files
- `mod.rs` - Public exports and `init_tracing()` for setting up the tracing subscriber with optional TUI mode.
  - **Key items**: `TracingGuards`, `init_tracing()`, `TuiLogBuffer`, `TuiLogLayer`
- `json.rs` - JSON log writer, tracing layer, and TUI log capture with consecutive-record deduplication.
  - **Key items**: `LogRecord`, `JsonLogWriter`, `JsonLogLayer`, `JsonLogFilter`, `TuiLogBuffer`, `TuiLogEntry`, `TuiLogLayer`, `LOG_BUFFER`, `DEDUP_WINDOW`, `hash_record()`, `format_tui_field_value()`
- `report.rs` - Run report types and generation for benchmarking runs.
  - **Key items**: `RunReport`, `RunContext`, `PeerHealthSummary`, `generate_run_report()`, `finalize_log_files()`, `run_timestamp_utc()`
- `resources.rs` - Resource monitoring with platform-specific implementations.
  - **Key items**: `spawn_resource_logger()`, `spawn_usr1_state_logger()`

## Key APIs (no snippets)
- **Types**: `TracingGuards`, `RunContext`, `RunReport`, `LogRecord`, `JsonLogWriter`, `JsonLogLayer`, `TuiLogBuffer`, `TuiLogEntry`, `TuiLogLayer`
- **Functions**: `init_tracing()`, `generate_run_report()`, `finalize_log_files()`, `spawn_resource_logger()`, `spawn_usr1_state_logger()`

## Tracing Initialization
`init_tracing()` accepts a `tui_mode` flag:
- When `tui_mode=true`: Suppresses stdout fmt_layer to avoid corrupting TUI display; captures logs
  to a shared `TuiLogBuffer` for dashboard rendering instead.
- When `tui_mode=false`: Writes logs to stdout as normal.

## TUI Log Capture
`TuiLogBuffer` provides a circular buffer (max 100 entries) for the TUI dashboard to display recent logs:
- `TuiLogEntry` contains: `level`, `message`, `timestamp_ms` (Unix epoch)
- `TuiLogLayer` captures logs at the configured minimum level and formats them for display
- Field values are truncated: long hex strings shown as `0x1234...cdef`, long strings as `prefix...`
- Structured fields appended to message as `key=value` pairs separated by pipe `|`

## Resource Monitoring
Platform-specific implementations:
- **Linux**: Reads `/proc/self/status`, `/proc/stat`, `/proc/diskstats` for detailed memory breakdown (RSS, anon, file, shmem), CPU usage with iowait, and disk I/O rates.
- **Non-Linux** (macOS, etc.): Uses `sysinfo` crate for cross-platform metrics with reduced granularity.

Both emit `BenchEvent::ResourcesSample` events and `debug!("resources", ...)` log entries every second.
Includes P2P network stats: reth connected peers, discovered count, genesis mismatches, sessions established/closed.

## SIGUSR1 Handler (Unix only)
`spawn_usr1_state_logger()` installs a signal handler that dumps:
1. Current sync progress stats (status, queue, inflight, peers, head block)
2. Peer pool summary (connected, cooling down, available)
3. Top 3 and worst 3 peers by quality score with backoff/inflight info

Usage: `kill -USR1 <pid>`

## Relationships
- **Used by**: `node/src/main.rs` for tracing initialization (with TUI mode flag), resource logging, and report generation.
- **Depends on**: `sync::SyncProgressStats`, `sync::historical::BenchEventLogger`, `sync::historical::PeerHealthTracker`, `p2p::PeerPool`, `p2p::P2pStats`.
- **Emits**: JSON log files (`.logs.jsonl`), resource logs (`.resources.jsonl`), run reports (`.json`), Chrome traces (`.trace.json`).

## Log Deduplication
The JSON log writer's background thread collapses consecutive identical records (same message + fields hash)
within a 1-second window into a single record with `"count": N`. This prevents multi-GB log files from
spin-loop scenarios (e.g., stale-peer banning). The dedup buffer is flushed on message change, timeout,
flush interval, and channel close.
