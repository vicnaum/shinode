# ui

## Purpose
Terminal UI module providing two rendering modes:
1. **Progress bars** (via indicatif) - Lightweight progress display for non-TUI environments
2. **TUI dashboard** (via ratatui) - Full-featured interactive dashboard with sync statistics,
   real-time speed chart, coverage map, and multiple information panels

## Contents (one hop)
### Files
- `mod.rs` - Main module exposing TUI mode detection, status bars, and db stats output.
  - **Key items**: `set_tui_mode()`, `is_tui_mode()`, `print_status_bar()`, `clear_status_bar()`, `print_db_stats()`, `human_bytes()`
- `state.rs` - Legacy UI state enum for progress bar-based rendering.
  - **Key items**: `UIState` (Startup, Syncing, Compacting, Sealing, Following)
- `bars.rs` - Progress bar creation helpers and color definitions.
  - **Key items**: `colors`, `create_sync_bar()`, `create_compacting_bar()`, `create_sealing_bar()`, `create_follow_bar()`, `create_failed_bar()`, `format_colored_segment()`
- `progress.rs` - Progress bar controller and background updater tasks.
  - **Key items**: `UIController`, `spawn_progress_updater()`, `spawn_tui_progress_updater()`
- `tui.rs` - Ratatui-based fullscreen TUI dashboard with comprehensive rendering system.
  - **Key items**: `TuiState`, `TuiController`, `Phase`, `LogEntry`, `LogLevel`, rendering functions

## Key APIs (no snippets)
- **Types**: `UIState`, `UIController`, `TuiState`, `TuiController`, `Phase`, `LogEntry`, `LogLevel`
- **Functions**: `set_tui_mode()`, `is_tui_mode()`, `print_status_bar()`, `clear_status_bar()`, `spawn_progress_updater()`, `spawn_tui_progress_updater()`, `print_db_stats()`, `human_bytes()`
- **Bar creators**: `create_sync_bar()`, `create_compacting_bar()`, `create_sealing_bar()`, `create_follow_bar()`, `create_failed_bar()`

## Two-Mode Architecture

### Progress bars (indicatif)
Used when `--no-tui` is set or stderr is not a TTY. `UIController` manages state transitions and
progress bars. `spawn_progress_updater()` polls stats every 100ms.

### TUI dashboard (ratatui)
Default mode. `TuiController` manages terminal raw mode and alternate screen. `spawn_tui_progress_updater()`
polls stats every 100ms, calculates speed, draws the dashboard, and handles 'q' key quit.

## TUI Dashboard (tui.rs)

### Phase Enum
Ordered enum (0-5) preventing backwards transitions:
- **Startup** (Yellow) - Initial connection, splash screen
- **Sync** (Cyan) - Active block fetching
- **Retry** (Red) - Escalation queue active
- **Compact** (Magenta) - Compacting shards
- **Seal** (Green) - Sealing completed shards
- **Follow** (Light Green) - Following chain head

### TuiState
Complete display state derived from `SyncProgressSnapshot`:
- Phase, progress (0.0-1.0), block range, speed metrics (current/avg/peak + history)
- Peer counts (connected, max, stale, peak_total), queue state (pending, inflight, retry)
- Compaction/sealing counters, storage bytes breakdown, DB record counts
- RPC stats (active, requests/sec, method counters, errors)
- Coverage buckets for blocks map, logs buffer (100 entries), animation frame
- Config display fields, startup status, synced status

**Key methods**: `new()`, `seed_storage_stats()`, `set_config()`, `update_from_snapshot()`, `add_log()`, `drain_log_buffer()`, `is_synced()`, `eta_string()`

### TuiController
Manages terminal I/O and rendering loop:
- `new()` - Enter alternate screen, enable raw mode
- `poll_quit()` - Non-blocking 'q' key check
- `draw()` - Render current state, increment animation frame
- `restore()` - Exit alternate screen (also called on Drop)

### Rendering Panels

**Layout varies by phase:**

- **Startup (0 peers)**: Full splash screen with DOS-style ASCII art, animated status
- **Startup (with peers)**: Standard startup UI with config panel
- **Sync/Retry**: Header, phase indicator, progress bar, blocks map (braille), speed chart (braille line graph), network/queue/storage/DB panels, logs
- **Compact/Seal**: Same layout but shard map replaces blocks map, compaction panel replaces speed chart
- **Follow**: Synced status with big ASCII number digits, network/storage/DB/RPC panels, logs

**Panel details:**
- **Blocks map**: 2-row braille grid, column-major, color gradient (gray->yellow->green)
- **Speed chart**: 7-row braille line graph with Y-axis labels, filled area gradient, stats line (Cur/Avg/Peak/ETA)
- **Network**: Peer dots (active/idle/stale/gone), never-shrinking visualization
- **Queue**: Remaining/inflight/retry with color coding
- **Storage**: Headers/Txns/Receipts breakdown in bytes, total GiB, write rate MB/s
- **DB**: Blocks/Txns/Logs counts, shards (compacted/total)
- **RPC** (follow only): Active status, req/s, total, getLogs/getBlock counts, errors
- **Logs**: Timestamped entries with level coloring (ERROR red, WARN yellow, INFO cyan, DEBUG gray)

### Follow Mode Staleness
`is_synced()` checks if `last_block_received_ms` is older than 30 seconds. If stale, shows
"CATCHING UP" with blocks behind count instead of "SYNCED".

## UIController State Machine (indicatif)
| State | Color | Format |
|-------|-------|--------|
| Startup | Yellow | `Opening storage...` or `P2P connected \| N peers` |
| Syncing | Cyan/Blue | `[====..] 45% 4500/10000 \| status \| peers \| ...` |
| Compacting | Magenta | `[====..] 80% 8/10 \| Compacting: 2 shards left` |
| Sealing | Bright Green | `[====..] 80% 8/10 \| Sealing: 2 shards left` |
| Following | Green | `[ 12345678 ] Synced \| head N \| peers N/M \| ...` |
| Recovery | Red | `[====..] 40% 4/10 \| Recovering failed blocks` |

## Relationships
- **Used by**: `node/src/run/startup.rs` creates TUI early for splash; `node/src/run/sync_runner.rs` creates UI controller.
- **Depends on**: `sync::SyncStatus`, `sync::SyncProgressStats`, `sync::SyncProgressSnapshot`, `logging::TuiLogBuffer` for log capture.
- **Integrates with**: `indicatif::MultiProgress` (legacy mode), `ratatui` + `crossterm` (TUI mode).
