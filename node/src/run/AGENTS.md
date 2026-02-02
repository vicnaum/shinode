# run

## Purpose

Top-level orchestration layer for the node's runtime lifecycle. Coordinates initialization, P2P connection, sync execution, follow mode, RPC server startup, cleanup, and utility commands. Extracted from the original monolithic `main()` to provide clear separation of concerns between startup, sync execution, follow mode tracking, and cleanup/finalization. Manages the early TUI splash screen during startup and wires storage/DB stats into the dashboard.

## Files (detailed)

### `mod.rs`
- **Role**: Module organization and public API surface (~31 lines). Re-exports primary functions and internal APIs.
- **Key items**: `run_sync`, `handle_db_stats`, `handle_db_compact`, `handle_repair`, plus re-exports of startup/cleanup/session/tracker helpers
- **Interactions**: All internal modules are private; public API exposed through re-exports.

### `sync_runner.rs`
- **Role**: Main sync orchestration (~727 lines). Sequences startup, sync, follow, and cleanup phases.
- **Key items**: `run_sync`, `run_follow_mode`, `log_follow_transition`, `FollowModeConfig`, `FollowModeTrackers`, `FollowModeState`
- **Interactions**: Called from `main.rs`. Delegates to all subsystems: cli, logging, p2p, storage, sync, rpc, ui. Spawns SIGINT handler (first=graceful, second=force exit code 130).
- **Knobs / invariants**:
  - `COVERAGE_BUCKETS = 200` (for TUI blocks map)
  - Early TUI reused to avoid screen flicker on transition
  - RPC server spawned only after chain synced (on `synced_rx` signal)
  - Default peer cache: `~/.stateless-history-node`

### `startup.rs`
- **Role**: Initialization functions for storage, P2P, UI, and early TUI splash (~465 lines).
- **Key items**: `build_run_context`, `init_storage`, `connect_p2p`, `wait_for_min_peers`, `wait_for_peer_head`, `setup_ui`, `update_tui_startup`, `update_tui_startup_head`, `update_tui_startup_peers`, `handle_startup_quit`, `EarlyTui`, `UiSetup`, `LogWriters`, `LogBufferRef`
- **Interactions**: Called by `sync_runner.rs`. Creates `Storage`, `NetworkSession`, `TuiController`/`UIController`, `BenchEventLogger`. Shows startup splash with animated dots.
- **Knobs / invariants**:
  - Startup compacts dirty shards from previous incomplete runs
  - Wait for peer head blocks until head >= start_block (with rollback window)
  - `update_tui_startup*` functions return `true` if quit was requested; caller must invoke `handle_startup_quit` to restore terminal and exit

### `cleanup.rs`
- **Role**: Unified finalization and cleanup logic (~135 lines).
- **Key items**: `FinalizeContext`, `finalize_session`, `apply_cached_peer_limits`, `flush_peer_cache_with_limits`, `persist_peer_limits`
- **Interactions**: Called by `sync_runner.rs` at end of fast-sync or follow mode. Generates run report, finalizes log files, flushes peer cache with AIMD batch limits.

### `commands.rs`
- **Role**: Handlers for CLI subcommands (~103 lines).
- **Key items**: `handle_db_stats`, `handle_db_compact`, `handle_repair`
- **Interactions**: Called directly from `main.rs`. `handle_db_stats` prints storage stats (table or JSON). `handle_db_compact` opens storage, compacts all dirty shards, seals completed ones, prints summary. `handle_repair` runs `Storage::repair()`.

### `session.rs`
- **Role**: Session-scoped resource types (~90 lines).
- **Key items**: `IngestProgress` (progress bar wrapper implementing `ProgressReporter`), `FollowModeResources` (container for head/tail tracker tasks with clean shutdown ordering)
- **Interactions**: `IngestProgress` used by sync pipeline. `FollowModeResources` managed by `sync_runner.rs`; `shutdown()` stops tail feeder first, then head tracker.

### `trackers.rs`
- **Role**: Background tasks for follow mode (~141 lines).
- **Key items**: `HeadTrackerHandles`, `TailFeederHandles`, `spawn_head_tracker`, `spawn_tail_feeder`, `build_tail_config`
- **Interactions**: Head tracker polls `PeerPool` every 1s for best head, persists to storage and updates stats. Tail feeder watches head advances, emits new block ranges past rollback window (500ms poll).

## Key APIs (no snippets)

- `run_sync(config, argv)` - Main entry point; orchestrates the entire sync lifecycle from startup through follow mode or completion. Creates early TUI with splash screen, seeds storage stats, handles fast-sync and follow-mode transitions.
- `handle_db_stats(args, config)` - Handles `db stats` subcommand; prints storage statistics (table or JSON).
- `handle_db_compact(args, config)` - Handles `db compact` subcommand; opens storage, compacts all dirty shards, seals completed ones, prints summary.
- `handle_repair(config)` - Handles `--repair` flag; runs `Storage::repair()` and prints per-shard results.
- `finalize_session(ctx, logs_total)` - Unified cleanup; generates reports, finalizes logs, flushes peer cache with AIMD limits.
- `setup_ui(...)` - Sets up UI controller, reuses early TUI, spawns progress updater. Supports both TUI (ratatui) and indicatif modes.
- `build_run_context(config, argv)` - Constructs `RunContext` for log artifact paths (trace, events, JSON logs, resources).
- `init_storage(config, tui, log_buffer)` - Opens storage, compacts dirty shards from previous incomplete runs.
- `connect_p2p(storage, tui, log_buffer)` - Starts devp2p networking with TUI status updates.
- `spawn_head_tracker(pool, storage, stats, initial_head, rollback_window)` - Spawns background task polling peer heads every 1s.
- `spawn_tail_feeder(initial_end, rollback_window, head_seen_rx)` - Spawns background task emitting new block ranges past rollback window.

## Relationships

- **Depends on**: All subsystems - `cli`, `logging`, `p2p`, `rpc`, `storage`, `sync`, `ui`, `metrics`
- **Used by**: `main.rs` calls `run_sync()`, `handle_db_stats()`, `handle_db_compact()`, `handle_repair()`
- **Data/control flow**:
  1. `run_sync()` validates config and builds run context for log artifacts.
  2. Determines TUI mode early (TTY check + `--no-tui` flag) before tracing init.
  3. Creates early TUI splash screen if in TUI mode; spawns background redraw ticker (100ms).
  4. `init_storage()` opens storage, compacts dirty shards, seeds storage stats into early TUI.
  5. `connect_p2p()` starts devp2p networking with status updates to TUI.
  6. `wait_for_min_peers()` and `wait_for_peer_head()` block until ready, updating TUI in realtime.
  7. Computes missing ranges, pre-fills TUI coverage map with already-present blocks.
  8. `setup_ui()` reuses early TUI (transitions Startup -> Sync) or creates indicatif controller.
  9. Sets up follow-mode resources (head tracker, tail feeder) if `--end-block` not specified.
  10. Runs ingest pipeline via `sync::historical::run_ingest_pipeline()`.
  11. Enters follow mode if not shutdown; RPC starts after first "synced" edge.
  12. `finalize_session()` generates reports, flushes peer cache.
  13. Shutdown: SIGINT handler with graceful (first) and force exit code 130 (second).

## Notes
