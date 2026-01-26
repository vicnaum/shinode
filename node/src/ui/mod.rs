//! Terminal UI module for progress bars and status display.
//!
//! This module provides a unified interface for displaying progress
//! during different phases of the sync process:
//!
//! - **Startup** (Yellow): Opening storage, connecting to peers, etc.
//! - **Syncing** (Cyan): Actively downloading blocks
//! - **Finalizing** (Teal): Compacting database shards
//! - **Following** (Green): Synced and following the chain head
//! - **Failed Recovery** (Red): Recovering failed blocks (overlay)

// Allow unused items - these are exported for future use and API completeness
#![allow(dead_code)]
#![allow(unused_imports)]

mod bars;
mod progress;
mod state;

pub use bars::{
    colors, create_failed_bar, create_finalizing_bar, create_follow_bar, create_sync_bar,
    format_colored_segment, format_finalizing_segment, format_follow_segment,
    format_startup_segment, BAR_WIDTH,
};
pub use progress::{format_progress_message, spawn_progress_updater, ProgressUI};
pub use state::{StartupPhase, UIState};

use indicatif::MultiProgress;
use std::io::Write;

/// Print a yellow status bar to stderr (used during startup phases).
/// This is a simple one-line status that overwrites itself.
pub fn print_status_bar(message: &str) {
    let bar = format_startup_segment(message);
    eprint!("\r{}", bar);
    let _ = std::io::stderr().flush();
}

/// Clear the status bar line.
pub fn clear_status_bar() {
    eprint!("\r{:width$}\r", "", width = BAR_WIDTH + 10);
    let _ = std::io::stderr().flush();
}

/// Create a new MultiProgress coordinator for managing multiple bars.
pub fn create_multi_progress() -> MultiProgress {
    MultiProgress::new()
}
