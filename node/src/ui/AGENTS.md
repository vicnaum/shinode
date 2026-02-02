# ui

## Purpose
Terminal UI module providing two rendering modes:
1. **Progress bars** (via indicatif) - Lightweight progress display for non-TUI environments
2. **TUI dashboard** (via ratatui) - Full-featured interactive dashboard with sync statistics,
   real-time speed chart, coverage map, and multiple information panels

Phase-aware layouts adapt to sync/compact/seal/follow stages.

## Files (detailed)

### `mod.rs` (~127 lines)
- **Role**: Module root, status bar utilities, DB stats output, TUI mode global flag.
- **Key items**: `TUI_MODE_ACTIVE` (global `AtomicBool`), `set_tui_mode()`, `is_tui_mode()`, `print_status_bar()`, `clear_status_bar()`, `print_db_stats()`, `human_bytes()`
- **Interactions**: Status bar functions no-op when TUI mode is active. `print_db_stats` formats `StorageDiskStats` as table or JSON.

### `state.rs` (~47 lines)
- **Role**: High-level UI state enum for phase tracking.
- **Key items**: `UIState` (Startup, Syncing, Compacting, Sealing, Following), `from_sync_snapshot`
- **Interactions**: Maps `SyncStatus` + `FinalizePhase` to `UIState`. Used by `UIController` and TUI for phase transitions.

### `bars.rs` (~117 lines)
- **Role**: indicatif progress bar factory functions and ANSI color formatting.
- **Key items**: `create_sync_bar()`, `create_compacting_bar()`, `create_sealing_bar()`, `create_follow_bar()`, `create_failed_bar()`, `format_colored_segment()`, `format_follow_segment()`, `format_startup_segment()`
- **Interactions**: Used by `UIController` to create bars for each phase.
- **Knobs / invariants**: `BAR_WIDTH = 40`, themed colors per phase (cyan/magenta/green/red), 2-10 Hz refresh rates

### `progress.rs` (~617 lines)
- **Role**: Legacy progress bar controller and updater tasks.
- **Key items**: `UIController`, `spawn_progress_updater()`, `spawn_tui_progress_updater()`, `format_progress_message()`
- **Interactions**: `UIController` is the state machine for indicatif mode. `spawn_tui_progress_updater` drives TUI mode at 100ms ticks: updates state, polls keyboard, drains logs, handles quit with graceful shutdown (3s timeout, flush log writers).
- **Knobs / invariants**:
  - Update interval: 100ms
  - Peer update interval: 500ms
  - Speed window: 1s (current), 30s (avg)
  - Quit timeout: 3s

### `tui.rs` (~2407 lines, largest file in project)
- **Role**: Ratatui fullscreen dashboard with comprehensive rendering system.
- **Key items**:
  - **State**: `Phase` (Startup/Sync/Retry/Compact/Seal/Follow with ordering), `TuiState`, `TuiController`, `LogEntry`, `LogLevel`
  - **Rendering**: `render_ui`, `render_sync_ui`, `render_compact_ui`, `render_follow_ui`, `render_startup_ui`
  - **Components**: `render_phase_indicator`, `render_blocks_map` (braille coverage), `render_shards_map` (block chars), `render_speed_chart` (gradient braille line graph), `render_network_panel` (peer dots), `render_synced_status` (big ASCII numbers + sparkle animation), `render_logs_panel`
  - **Splash**: DOS-style blue/gold ASCII art with animated status dots
- **Interactions**: `TuiController` manages alternate screen + raw mode. Consumes `TuiLogBuffer` for log panel. Phase enum ordering prevents backwards transitions.
- **Knobs / invariants**:
  - Speed history: 60 samples (1 minute)
  - Avg speed window: 30 seconds (300 entries at 100ms)
  - Log capacity: 100 entries
  - Follow staleness threshold: 30s
  - Chart height scales: 50/200/500/2000/5000+
  - Braille: 4 vertical dots per character cell

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

- **Depends on**: `sync::{SyncProgressStats, SyncProgressSnapshot, SyncStatus, FinalizePhase, CoverageTracker}`, `storage::StorageDiskStats`, `p2p::PeerPool`, `sync::historical::PeerHealthTracker`, `logging::{TuiLogBuffer, JsonLogWriter}`
- **Used by**: `run/startup.rs` (initializes controllers, creates TUI early for splash), `run/sync_runner.rs` (spawns updater tasks)
- **Integrates with**: `indicatif::MultiProgress` (legacy mode), `ratatui` + `crossterm` (TUI mode)
- **Data/control flow**:
  1. `SyncProgressStats` polled every 100ms for snapshot
  2. Snapshot drives phase transitions and metric updates
  3. TUI mode: renders to alternate screen via ratatui; drains `TuiLogBuffer` for log panel
  4. Legacy mode: updates indicatif progress bars with phase-specific templates
  5. Quit ('q'): signals shutdown, waits for completion, flushes log writers, restores terminal

## End-to-end flow (high level)

1. `run/startup.rs` determines TTY status and creates `TuiController` or `UIController`
2. Spawns corresponding updater task (TUI or progress bar) at 100ms tick rate
3. Updater polls `SyncProgressStats::snapshot()`, calculates speed, updates peer counts
4. Phase transitions: Startup -> Sync -> Compact -> Seal -> Follow (no backwards)
5. TUI renders phase-specific layout with appropriate panels (speed chart during sync, compaction panel during compact, big numbers during follow)
6. On quit: send shutdown signal, wait for completion or 3s timeout, flush logs, restore terminal
