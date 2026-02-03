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

// UI output requires direct stdout/stderr for terminal control
#![expect(clippy::print_stdout, clippy::print_stderr, reason = "UI output")]

mod bars;
mod beep;
mod progress;
mod state;
pub mod tui;

pub use bars::{format_startup_segment, BAR_WIDTH};
pub use progress::{spawn_progress_updater, spawn_tui_progress_updater, UIController};

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag indicating TUI mode is active.
/// When true, print_status_bar and clear_status_bar should be no-ops.
static TUI_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Set TUI mode active state. Call this before initializing TUI.
pub fn set_tui_mode(active: bool) {
    TUI_MODE_ACTIVE.store(active, Ordering::SeqCst);
}

/// Check if TUI mode is active.
pub fn is_tui_mode() -> bool {
    TUI_MODE_ACTIVE.load(Ordering::SeqCst)
}

/// Print a yellow status bar to stderr (used during startup phases).
/// This is a simple one-line status that overwrites itself.
/// Does nothing if TUI mode is active.
pub fn print_status_bar(message: &str) {
    if is_tui_mode() {
        return;
    }
    let bar = format_startup_segment(message);
    eprint!("\r{bar}");
    let _ = std::io::stderr().flush();
}

/// Clear the status bar line.
/// Does nothing if TUI mode is active.
pub fn clear_status_bar() {
    if is_tui_mode() {
        return;
    }
    eprint!("\r{:width$}\r", "", width = BAR_WIDTH + 10);
    let _ = std::io::stderr().flush();
}

// ============================================================================
// Database stats output
// ============================================================================

use crate::storage::StorageDiskStats;
use std::path::Path;

/// Print database statistics to stdout.
pub fn print_db_stats(
    data_dir: &Path,
    stats: &StorageDiskStats,
    json: bool,
) -> eyre::Result<()> {
    if json {
        let payload = serde_json::to_string_pretty(stats)?;
        println!("{payload}");
        return Ok(());
    }

    println!("DB stats for {}", data_dir.display());
    println!("{:<18} {:>14} {:>12}", "Component", "Bytes", "Size");
    println!("{}", "-".repeat(46));
    print_row("total", stats.total_bytes);
    print_row("static", stats.static_total_bytes);
    print_row("meta", stats.meta_bytes);
    print_row("peers", stats.peers_bytes);

    println!();
    println!(
        "{:<18} {:>14} {:>12} {:>8}",
        "Segment", "Bytes", "Size", "Share"
    );
    println!("{}", "-".repeat(54));
    for segment in &stats.segments {
        let share = if stats.static_total_bytes > 0 {
            segment.bytes as f64 / stats.static_total_bytes as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "{:<18} {:>14} {:>12} {:>7.2}%",
            segment.name,
            segment.bytes,
            human_bytes(segment.bytes),
            share
        );
    }
    Ok(())
}

fn print_row(label: &str, bytes: u64) {
    println!("{:<18} {:>14} {:>12}", label, bytes, human_bytes(bytes));
}

/// Format a byte count as a human-readable string (e.g., "1.23 GiB").
pub fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut idx = 0;
    while value >= 1024.0 && idx < UNITS.len() - 1 {
        value /= 1024.0;
        idx += 1;
    }
    format!("{value:.2} {}", UNITS[idx])
}
