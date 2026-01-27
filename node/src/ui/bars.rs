//! Progress bar rendering helpers and color definitions.

// Progress style templates are compile-time constants that cannot fail
#![expect(clippy::expect_used, reason = "style templates are compile-time constants")]

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

/// Color definitions for different UI states (RGB values).
pub mod colors {
    /// Yellow for startup phases (black text on yellow background).
    pub const YELLOW: (u8, u8, u8) = (255, 200, 0);
    /// Green for synced/following phase (white text on green background).
    pub const GREEN: (u8, u8, u8) = (0, 128, 0);
}

/// Bar width constant used across all bars.
pub const BAR_WIDTH: usize = 40;

/// Create the main sync progress bar (cyan/blue themed).
/// Format: [████░░] 45% 4500/10000 | status | peers | queue | inflight | retry | speed | eta
pub fn create_sync_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
    let style = ProgressStyle::with_template(
        "{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {msg}",
    )
    .expect("progress style")
    .progress_chars("█▉░");
    bar.set_style(style);
    bar
}

/// Create the compacting progress bar (magenta themed).
/// Format: [████░░] 80% 8/10 | Compacting: 2 shards left
pub fn create_compacting_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total.max(1)));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    let style = ProgressStyle::with_template(
        "{bar:40.magenta/black} {percent:>3}% {pos}/{len} | {msg}",
    )
    .expect("progress style")
    .progress_chars("█▓░");
    bar.set_style(style);
    bar
}

/// Create the sealing progress bar (bright green themed).
/// Format: [████░░] 80% 8/10 | Sealing: 2 shards left
pub fn create_sealing_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total.max(1)));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    let style = ProgressStyle::with_template(
        "{bar:40.bright.green/black} {percent:>3}% {pos}/{len} | {msg}",
    )
    .expect("progress style")
    .progress_chars("█▓░");
    bar.set_style(style);
    bar
}

/// Create the follow mode status bar (green themed, custom colored segment).
/// Format: [ 12345678 ] Synced | head N | peers N/M | ...
pub fn create_follow_bar(multi: &MultiProgress) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(100));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    let style = ProgressStyle::with_template("{msg}").expect("progress style");
    bar.set_style(style);
    bar
}

/// Create the failed/recovery progress bar (red themed).
/// Format: [████░░] 40% 4/10 | Recovering failed blocks
pub fn create_failed_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total.max(1)));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    let style = ProgressStyle::with_template("{bar:40.red/black} {percent:>3}% {pos}/{len} | {msg}")
        .expect("progress style")
        .progress_chars("▓▒░");
    bar.set_style(style);
    bar
}

/// Format a colored status segment using ANSI escape codes.
/// Returns a string like: `\x1b[...m[ block_number ]\x1b[0m`
pub fn format_colored_segment(content: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) -> String {
    let padding = BAR_WIDTH.saturating_sub(content.len());
    let left_pad = padding / 2;
    let right_pad = padding - left_pad;

    format!(
        "\x1b[38;2;{};{};{};48;2;{};{};{}m{:>width_l$}{}{:<width_r$}\x1b[0m",
        fg.0,
        fg.1,
        fg.2,
        bg.0,
        bg.1,
        bg.2,
        "",
        content,
        "",
        width_l = left_pad,
        width_r = right_pad,
    )
}

/// Format the green follow bar segment with block number.
pub fn format_follow_segment(block_number: u64) -> String {
    let content = format!("[ {block_number} ]");
    // White text on green background
    format_colored_segment(&content, (255, 255, 255), colors::GREEN)
}

/// Format the yellow startup bar segment.
pub fn format_startup_segment(message: &str) -> String {
    // Black text on yellow background
    format_colored_segment(message, (0, 0, 0), colors::YELLOW)
}
