//! Run orchestration module.
//!
//! This module organizes the node's runtime logic into focused submodules:
//! - `session`: Session types for managing sync state
//! - `commands`: Subcommand handlers (db stats, repair)
//! - `startup`: Initialization functions (storage, P2P, UI)
//! - `trackers`: Head tracking and tail feeding for follow mode
//! - `cleanup`: Unified finalization and cleanup logic
//! - `sync_runner`: Main sync orchestration

mod cleanup;
mod commands;
mod session;
mod startup;
mod sync_runner;
mod trackers;

pub use commands::{handle_db_compact, handle_db_rebuild_cache, handle_db_stats, handle_repair};
pub use sync_runner::run_sync;

// Re-exports for potential external use (not currently used from main.rs)
#[expect(unused_imports, reason = "re-exported API for external use")]
pub use cleanup::{finalize_session, FinalizeContext};
#[expect(unused_imports, reason = "re-exported API for external use")]
pub use session::{FollowModeResources, IngestProgress};
#[expect(unused_imports, reason = "re-exported API for external use")]
pub use startup::{
    build_run_context, connect_p2p, init_storage, setup_ui, wait_for_min_peers, wait_for_peer_head,
};
#[expect(unused_imports, reason = "re-exported API for external use")]
pub use trackers::{spawn_head_tracker, spawn_tail_feeder, HeadTrackerHandles, TailFeederHandles};
