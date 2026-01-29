# run

## Purpose
Run orchestration module that organizes the node's runtime logic. Extracted from the original
monolithic `main()` to provide clear separation of concerns between startup, sync execution,
follow mode tracking, and cleanup/finalization. Manages the early TUI splash screen during
startup and wires storage/DB stats into the dashboard.

## Contents (one hop)
### Files
- `mod.rs` - Module exports and public API surface.
  - **Key items**: `run_sync()`, `handle_db_stats()`, `handle_repair()`
- `session.rs` - Session types for managing sync state.
  - **Key items**: `IngestProgress`, `FollowModeResources`
- `commands.rs` - Subcommand handlers for db stats and repair.
  - **Key items**: `handle_db_stats()`, `handle_repair()`
- `startup.rs` - Initialization functions for storage, P2P, UI setup, and TUI splash animation.
  - **Key items**: `build_run_context()`, `init_storage()`, `connect_p2p()`, `wait_for_min_peers()`, `wait_for_peer_head()`, `setup_ui()`, `update_tui_startup()`, `update_tui_startup_head()`, `update_tui_startup_peers()`, `handle_startup_quit()`, `EarlyTui`, `UiSetup`, `LogWriters`
- `trackers.rs` - Head tracking and tail feeding for follow mode.
  - **Key items**: `spawn_head_tracker()`, `spawn_tail_feeder()`, `build_tail_config()`, `HeadTrackerHandles`, `TailFeederHandles`
- `cleanup.rs` - Unified cleanup and finalization logic.
  - **Key items**: `finalize_session()`, `FinalizeContext`, `apply_cached_peer_limits()`, `flush_peer_cache_with_limits()`
- `sync_runner.rs` - Main sync orchestration including fast-sync, follow mode, and TUI lifecycle.
  - **Key items**: `run_sync()`, `run_follow_mode()`, `log_follow_transition()`, `FollowModeConfig`, `FollowModeTrackers`, `FollowModeState`

## Key APIs (no snippets)
- `run_sync(config, argv)` - Main entry point; orchestrates the entire sync lifecycle from startup through follow mode or completion. Creates early TUI with splash screen, seeds storage stats, handles fast-sync and follow-mode transitions.
- `handle_db_stats(args, config)` - Handles `db stats` subcommand.
- `handle_repair(config)` - Handles `--repair` flag.
- `finalize_session(ctx, logs_total)` - Unified cleanup; generates reports and finalizes logs.
- `setup_ui(...)` - Sets up UI controller, reuses early TUI, spawns progress updater.

## Relationships
- **Used by**: `main.rs` delegates all sync execution to `run_sync()` and subcommand handling to `handle_db_stats()`/`handle_repair()`.
- **Uses**: `cli`, `logging`, `p2p`, `rpc`, `storage`, `sync`, `ui` modules.
- **Data/control flow**:
  1. `run_sync()` validates config and builds run context for log artifacts.
  2. Determines TUI mode early (TTY check + `--no-tui` flag) before tracing init.
  3. Creates early TUI splash screen if in TUI mode; spawns background redraw ticker.
  4. `init_storage()` opens storage, compacts dirty shards, seeds storage stats into early TUI.
  5. `connect_p2p()` starts devp2p networking with status updates to TUI.
  6. `wait_for_min_peers()` and `wait_for_peer_head()` block until ready, updating TUI in realtime.
  7. Computes missing ranges, pre-fills TUI coverage map with already-present blocks.
  8. `setup_ui()` reuses early TUI (transitions Startup -> Sync) or creates indicatif controller.
  9. Sets up follow-mode resources (head tracker, tail feeder) if `--end-block` not specified.
  10. Runs ingest pipeline via `sync::historical::run_ingest_pipeline()`.
  11. Enters follow mode if not shutdown; RPC starts after first "synced" edge.
  12. `finalize_session()` generates reports, flushes peer cache.
