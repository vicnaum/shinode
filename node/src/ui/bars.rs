//! Progress bar rendering helpers and color definitions.

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

/// Color definitions for different UI states (RGB values).
pub mod colors {
    /// Yellow for startup phases (black text on yellow background).
    pub const YELLOW: (u8, u8, u8) = (255, 200, 0);
    /// Teal for finalizing phase (white text on teal background).
    pub const TEAL: (u8, u8, u8) = (0, 200, 200);
    /// Green for synced/following phase (white text on green background).
    pub const GREEN: (u8, u8, u8) = (0, 128, 0);
}

/// Bar width constant used across all bars.
pub const BAR_WIDTH: usize = 40;

/// Create a startup spinner bar (yellow themed).
pub fn create_startup_bar(multi: &MultiProgress) -> ProgressBar {
    let bar = multi.add(ProgressBar::new_spinner());
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
    let style = ProgressStyle::with_template("{spinner:.yellow} {msg}")
        .expect("progress style")
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");
    bar.set_style(style);
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    bar
}

/// Create the main sync progress bar (cyan/blue themed).
pub fn create_sync_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
    let style = ProgressStyle::with_template(
        "{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}",
    )
    .expect("progress style")
    .progress_chars("█▉░");
    bar.set_style(style);
    bar
}

/// Create the finalizing progress bar (teal themed).
pub fn create_finalizing_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    // Use bright cyan (closest to teal in indicatif's color palette)
    let style = ProgressStyle::with_template(
        "{bar:40.bright.cyan/black} {pos}/{len} | {msg}",
    )
    .expect("progress style")
    .progress_chars("█▓░");
    bar.set_style(style);
    bar
}

/// Create the follow mode status bar (green themed, custom colored segment).
pub fn create_follow_bar(multi: &MultiProgress) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(100));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    let style = ProgressStyle::with_template("{msg}").expect("progress style");
    bar.set_style(style);
    bar
}

/// Create the failed recovery progress bar (red themed).
pub fn create_failed_bar(multi: &MultiProgress, total: u64) -> ProgressBar {
    let bar = multi.add(ProgressBar::new(total));
    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(2));
    let style = ProgressStyle::with_template("{bar:40.red/black} {pos}/{len} | {msg}")
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
    let content = format!("[ {} ]", block_number);
    // White text on green background
    format_colored_segment(&content, (255, 255, 255), colors::GREEN)
}

/// Format the teal finalizing bar segment.
pub fn format_finalizing_segment(message: &str) -> String {
    // White text on teal background
    format_colored_segment(message, (255, 255, 255), colors::TEAL)
}

/// Format the yellow startup bar segment.
pub fn format_startup_segment(message: &str) -> String {
    // Black text on yellow background
    format_colored_segment(message, (0, 0, 0), colors::YELLOW)
}
