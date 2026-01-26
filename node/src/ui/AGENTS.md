# ui

## Purpose
Terminal UI module for progress bars and status display during sync operations. Provides a unified
interface for displaying progress during different phases: startup, syncing, finalizing, following,
and failed block recovery.

## Contents (one hop)
### Files
- `mod.rs` - Main module exposing public API and simple status bar functions.
  - **Key items**: `print_status_bar()`, `clear_status_bar()`, `create_multi_progress()`
- `state.rs` - UI state machine types for tracking current display phase.
  - **Key items**: `UIState`, `StartupPhase`
- `bars.rs` - Progress bar creation helpers and color definitions.
  - **Key items**: `colors`, `create_sync_bar()`, `create_finalizing_bar()`, `create_follow_bar()`, `create_failed_bar()`, `create_recovery_bar()`, `format_colored_segment()`, `format_recovery_segment()`
- `progress.rs` - Main progress tracking logic and background updater.
  - **Key items**: `ProgressUI`, `format_progress_message()`, `spawn_progress_updater()`

## Key APIs (no snippets)
- **Types**: `UIState`, `StartupPhase`, `ProgressUI`
- **Functions**: `print_status_bar()`, `clear_status_bar()`, `format_progress_message()`, `spawn_progress_updater()`
- **Bar creators**: `create_sync_bar()`, `create_finalizing_bar()`, `create_follow_bar()`, `create_failed_bar()`

## Color Scheme
| Phase | Color | Description |
|-------|-------|-------------|
| Startup | Yellow (255,200,0) | Opening storage, connecting, waiting for peers |
| Recovery | Orange (255,140,0) | Recovering shards from interrupted compaction |
| Syncing | Cyan/Blue | Actively downloading blocks (shows "retry N" for escalation count) |
| Finalizing | Teal (0,200,200) | Compacting database shards |
| Following | Green (0,128,0) | Synced and following chain head |
| Block Recovery | Red | Recovering difficult blocks (only shown when normal queue is empty and only escalation blocks remain) |

## Recovery Bar Behavior
The red recovery bar only appears when:
1. Normal queue is empty (queue == 0)
2. Escalation queue has blocks (escalation > 0)
3. Fetch is not complete (fetch_complete == false)
4. Condition persists for 2+ seconds (to avoid flicker)

During normal sync, escalation count is shown as "retry N" in the status message (not a separate bar).

## Relationships
- **Used by**: `node/src/main.rs` for all terminal progress display.
- **Depends on**: `sync::SyncStatus`, `sync::SyncProgressStats`, `sync::SyncProgressSnapshot` for status information.
- **Integrates with**: `indicatif::MultiProgress` for coordinated multi-bar rendering.
