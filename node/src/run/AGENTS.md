# run

## Purpose
Run orchestration module that organizes the node's runtime logic. Extracted from the original
monolithic `main()` to provide clear separation of concerns between startup, sync execution,
follow mode tracking, and cleanup/finalization.

## Contents (one hop)
### Files
- `mod.rs` - Module exports and public API surface.
  - **Key items**: `run_sync()`, `handle_db_stats()`, `handle_repair()`
- `session.rs` - Session types for managing sync state.
  - **Key items**: `IngestProgress`, `FollowModeResources`
- `commands.rs` - Subcommand handlers for db stats and repair.
  - **Key items**: `handle_db_stats()`, `handle_repair()`
- `startup.rs` - Initialization functions for storage, P2P, and UI setup.
  - **Key items**: `build_run_context()`, `init_storage()`, `connect_p2p()`, `wait_for_min_peers()`, `wait_for_peer_head()`, `setup_ui()`, `UiSetup`
- `trackers.rs` - Head tracking and tail feeding for follow mode.
  - **Key items**: `spawn_head_tracker()`, `spawn_tail_feeder()`, `build_tail_config()`, `HeadTrackerHandles`, `TailFeederHandles`
- `cleanup.rs` - Unified cleanup and finalization logic.
  - **Key items**: `finalize_session()`, `FinalizeContext`, `apply_cached_peer_limits()`, `flush_peer_cache_with_limits()`
- `sync_runner.rs` - Main sync orchestration including fast-sync and follow mode.
  - **Key items**: `run_sync()`, `run_follow_mode()`, `log_follow_transition()`

## Key APIs (no snippets)
- `run_sync(config, argv)` - Main entry point; orchestrates the entire sync lifecycle from startup through follow mode or completion.
- `handle_db_stats(args, config)` - Handles `db stats` subcommand to print storage statistics.
- `handle_repair(config)` - Handles `--repair` flag to scan and recover interrupted shards.
- `finalize_session(ctx, logs_total)` - Unified cleanup replacing 3 duplicated paths; generates reports and finalizes logs.

## Relationships
- **Used by**: `main.rs` delegates all sync execution to `run_sync()` and subcommand handling to `handle_db_stats()`/`handle_repair()`.
- **Uses**: `cli`, `logging`, `p2p`, `rpc`, `storage`, `sync`, `ui` modules.
- **Data/control flow**:
  1. `run_sync()` validates config and builds run context for log artifacts.
  2. `init_storage()` opens storage and compacts any dirty shards from previous runs.
  3. `connect_p2p()` starts devp2p networking and waits for minimum peers.
  4. `setup_ui()` creates progress bars and event loggers.
  5. For follow mode, `spawn_head_tracker()` and `spawn_tail_feeder()` manage chain tip tracking.
  6. Ingest pipeline runs via `sync::historical::run_ingest_pipeline()`.
  7. On completion or shutdown, `finalize_session()` generates reports and flushes peer cache.
