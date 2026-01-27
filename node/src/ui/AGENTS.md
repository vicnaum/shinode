# ui

## Purpose
Terminal UI module for progress bars and status display during sync operations. Provides a unified
interface for displaying progress during different phases: startup, syncing, compacting, sealing,
following, and failed block recovery.

## Contents (one hop)
### Files
- `mod.rs` - Main module exposing public API, status bar functions, and db stats output.
  - **Key items**: `print_status_bar()`, `clear_status_bar()`, `print_db_stats()`, `human_bytes()`
- `state.rs` - UI state machine types for tracking current display phase.
  - **Key items**: `UIState`
- `bars.rs` - Progress bar creation helpers and color definitions.
  - **Key items**: `colors`, `create_sync_bar()`, `create_compacting_bar()`, `create_sealing_bar()`, `create_follow_bar()`, `create_failed_bar()`, `format_colored_segment()`
- `progress.rs` - Main progress tracking logic with UIController and background updater.
  - **Key items**: `UIController`, `spawn_progress_updater()` (note: `format_progress_message()` is now private)

## Key APIs (no snippets)
- **Types**: `UIState`, `UIController`
- **Functions**: `print_status_bar()`, `clear_status_bar()`, `spawn_progress_updater()`, `print_db_stats()`, `human_bytes()` (note: `format_progress_message()` is now private, takes `&SyncProgressSnapshot`)
- **Bar creators**: `create_sync_bar()`, `create_compacting_bar()`, `create_sealing_bar()`, `create_follow_bar()`, `create_failed_bar()`

## UIController State Machine
The `UIController` manages all progress bars and handles state transitions automatically based on
`SyncProgressSnapshot`. States:

| State | Color | Bar Type | Format |
|-------|-------|----------|--------|
| Startup | Yellow | Text status | `Opening storage...` or `P2P connected \| N peers` |
| Syncing | Cyan/Blue | Progress bar | `[████░░] 45% 4500/10000 \| status \| peers \| ...` |
| Compacting | Magenta | Progress bar | `[████░░] 80% 8/10 \| Compacting: 2 shards left` |
| Sealing | Bright Green | Progress bar | `[████░░] 80% 8/10 \| Sealing: 2 shards left` |
| Following | Green | Status segment | `[ 12345678 ] Synced \| head N \| peers N/M \| ...` |
| Recovery | Red | Overlay bar | `[████░░] 40% 4/10 \| Recovering failed blocks` |

## Color Scheme
| Phase | Color | Description |
|-------|-------|-------------|
| Startup | Yellow (255,200,0) | Opening storage, connecting, waiting for peers |
| Syncing | Cyan/Blue | Actively downloading blocks (shows "retry N" for escalation count) |
| Compacting | Magenta | Compacting database shards |
| Sealing | Bright Green | Sealing completed shards |
| Following | Green (0,128,0) | Synced and following chain head |
| Recovery | Red | Recovering difficult blocks (overlay bar) |

## Recovery Bar Behavior
The red recovery bar appears **instantly** (no delay) when:
1. Normal queue is empty (queue == 0)
2. Escalation queue has blocks (escalation > 0)
3. Fetch is not complete (fetch_complete == false)

The bar disappears instantly when any condition becomes false.
During normal sync, escalation count is shown as "retry N" in the status message (not a separate bar).

## Finalize Phase Counts
Both compacting and sealing phases know their totals upfront:
- Compacting: count from dirty shards needing compaction
- Sealing: uses `storage.count_shards_to_seal()` before starting

This enables proper "N shards left" display instead of incremental "sealed N shards".

## Relationships
- **Used by**: `node/src/main.rs` for all terminal progress display.
- **Depends on**: `sync::SyncStatus`, `sync::SyncProgressStats`, `sync::SyncProgressSnapshot` for status information.
- **Integrates with**: `indicatif::MultiProgress` for coordinated multi-bar rendering.
