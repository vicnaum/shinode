//! Terminal UI module for progress bars and status display.
//!
//! This module provides a unified interface for displaying progress
//! during different phases of the sync process:
//!
//! - **Startup** (Yellow): Opening storage, connecting to peers, etc.
//! - **Syncing** (Cyan): Actively downloading blocks
//! - **Compacting** (Teal): Compacting database shards
//! - **Sealing** (Teal): Sealing completed shards
//! - **Following** (Green): Synced and following the chain head
//! - **Recovery** (Red): Recovering failed blocks (overlay bar)

mod bars;
mod progress;
mod state;

pub use bars::{format_startup_segment, BAR_WIDTH};
pub use progress::{spawn_progress_updater, UIController};
// Used in tests
#[allow(unused_imports)]
pub use progress::format_progress_message;

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
