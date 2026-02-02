# logging

## Purpose

Logging, reporting, and resource monitoring module. Provides structured JSON logging with async file
writing, run reports for benchmarking and diagnostics, resource monitoring for CPU, memory, and
disk I/O, TUI log capture for dashboard display, and SIGUSR1 signal handler for on-demand state dumps.
Integrates with `tracing`/`tracing-subscriber` for multi-layer log routing.

## Files (detailed)

### `mod.rs`
- **Role**: Public API and tracing subscriber initialization (~163 lines).
- **Key items**: `TracingGuards`, `init_tracing`, `TuiLogBuffer`, `TuiLogLayer`
- **Interactions**: Called by `run/sync_runner.rs` at startup. Returns `TracingGuards` that must be held until shutdown. Creates layered subscriber: fmt OR TUI layer (never both), JSON log layer, resources layer, optional Chrome trace layer.
- **Knobs / invariants**:
  - Verbosity 0=INFO, 1=DEBUG, 2=TRACE, 3=verbose TRACE
  - TUI mode suppresses stdout (uses `TuiLogBuffer` instead)
  - JSON and resources logs written to separate files via `JsonLogFilter`

### `json.rs`
- **Role**: Async JSON log writer with deduplication, plus TUI log capture (~488 lines).
- **Key items**: `JsonLogWriter`, `LogRecord`, `JsonLogLayer`, `JsonLogVisitor`, `JsonLogFilter`, `TuiLogBuffer`, `TuiLogLayer`, `TuiLogEntry`, `hash_record()`, `format_tui_field_value()`
- **Interactions**: `JsonLogWriter` spawns background thread. `TuiLogBuffer` shared with TUI dashboard via `Arc`. WARN events suppressed unless `-v` or higher.
- **Knobs / invariants**:
  - `LOG_BUFFER = 10,000` channel capacity (drops events when full, never blocks)
  - `FLUSH_COUNT = 4096`, `FLUSH_INTERVAL = 2s`, `RECV_TIMEOUT = 500ms`
  - `DEDUP_WINDOW = 1s` (collapses identical consecutive records)
  - `TUI_LOG_BUFFER_SIZE = 100` ring buffer entries
  - Long hex strings truncated to `0x1234...cdef`, long strings to `prefix...`

### `report.rs`
- **Role**: Run report generation for benchmarking and diagnostics (~434 lines).
- **Key items**: `RunReport`, `RunContext`, `RunMeta`, `RunDerived`, `BuildInfo`, `EnvInfo`, `PeerHealthSummary`, `generate_run_report`, `finalize_log_files`, `run_timestamp_utc`
- **Interactions**: Called by `run/cleanup.rs` at shutdown. Reads `PeerHealthTracker` snapshot and `IngestBenchSummary`. Renames temp log files to final names.
- **Knobs / invariants**:
  - Top/worst peer count: 10
  - Custom UTC formatter (no chrono dependency)
  - Atomic file renaming for crash safety

### `resources.rs`
- **Role**: Platform-specific resource monitoring and SIGUSR1 handler (~628 lines).
- **Key items**: `spawn_resource_logger`, `spawn_usr1_state_logger`, `ProcMemSample`, `CpuSample`, `DiskSample`, `read_proc_status_mem_kb`, `read_proc_cpu_sample`, `read_proc_disk_sample`
- **Interactions**: Spawned by `run/sync_runner.rs`. Linux: parses `/proc/self/status`, `/proc/stat`, `/proc/diskstats`. Non-Linux: uses `sysinfo` crate. SIGUSR1 logs sync progress + top/worst 3 peers.
- **Knobs / invariants**:
  - Sampling interval: 1s
  - Linux: zero-dependency proc parsing; non-Linux: sysinfo crate (no iowait, no RSS breakdown)
  - Disk device filtering: excludes loop, ram, dm-; supports nvme, sd, vd, xvd, md

## Key APIs (no snippets)

- **Types**: `TracingGuards`, `JsonLogWriter`, `TuiLogBuffer`, `TuiLogLayer`, `TuiLogEntry`, `LogRecord`, `RunContext`, `RunReport`, `RunMeta`, `RunDerived`, `BuildInfo`, `EnvInfo`, `PeerHealthSummary`
- **Functions**: `init_tracing()`, `generate_run_report()`, `finalize_log_files()`, `spawn_resource_logger()`, `spawn_usr1_state_logger()`, `hash_record()`, `format_tui_field_value()`

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

## Log Deduplication

The JSON log writer's background thread collapses consecutive identical records (same message + fields hash)
within a 1-second window into a single record with `"count": N`. This prevents multi-GB log files from
spin-loop scenarios (e.g., stale-peer banning). The dedup buffer is flushed on message change, timeout,
flush interval, and channel close.

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

- **Depends on**: `cli::NodeConfig`, `sync::SyncProgressStats`, `sync::historical::{BenchEventLogger, PeerHealthTracker, IngestBenchSummary}`, `p2p::{PeerPool, P2pStats}`, `tracing`/`tracing-subscriber`, `sysinfo` (non-Linux)
- **Used by**: `run/sync_runner.rs` (init + spawn), `run/cleanup.rs` (report + finalize), `ui/tui.rs` (reads `TuiLogBuffer`)
- **Emits**: JSON log files (`.logs.jsonl`), resource logs (`.resources.jsonl`), run reports (`.json`), Chrome traces (`.trace.json`)
- **Data/control flow**:
  1. Tracing events flow through layers to multiple destinations (stdout/TUI, JSON file, resources file, Chrome trace)
  2. Resource monitor emits `BenchEvent::ResourcesSample` every 1s
  3. SIGUSR1 handler dumps sync + peer state on demand
  4. Shutdown: generate report, flush writers, rename temp files to final names

## End-to-end flow (high level)

1. `init_tracing()` creates layered subscriber with conditional layers based on config
2. `spawn_resource_logger()` starts 1s sampling loop (CPU, memory, disk I/O)
3. `spawn_usr1_state_logger()` registers Unix signal handler for debug dumps
4. During runtime, all tracing events route through layers with filtering
5. `JsonLogWriter` background thread batches and deduplicates log records
6. `TuiLogBuffer` captures formatted entries for dashboard log panel
7. On shutdown, `generate_run_report()` produces summary JSON with peer health
8. `finalize_log_files()` flushes writers and renames temp files atomically
