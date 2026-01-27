//! Mock UI for testing the Ratatui dashboard design.
//!
//! Run with: cargo run --bin ui-mock --manifest-path node/Cargo.toml
//!
//! Press 'q' to quit, 'n' to cycle to next phase.

use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};

// ============================================================================
// Data Types
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Sync,
    Retry,
    Compact,
    Seal,
    Follow,
}

impl Phase {
    const fn color(self) -> Color {
        match self {
            Self::Sync => Color::Cyan,
            Self::Retry => Color::Red,
            Self::Compact => Color::Magenta,
            Self::Seal => Color::Green,
            Self::Follow => Color::LightGreen,
        }
    }

    const fn next(self) -> Self {
        match self {
            Self::Sync => Self::Retry,
            Self::Retry => Self::Compact,
            Self::Compact => Self::Seal,
            Self::Seal => Self::Follow,
            Self::Follow => Self::Sync,
        }
    }

    const fn status_text(self) -> &'static str {
        match self {
            Self::Sync => "Fetching",
            Self::Retry => "Retrying",
            Self::Compact => "Compacting",
            Self::Seal => "Sealing",
            Self::Follow => "Following",
        }
    }
}

struct MockData {
    phase: Phase,
    progress: f64,           // 0.0 to 1.0
    start_block: u64,
    end_block: u64,
    current_block: u64,
    speed_history: Vec<u64>, // Last 60 samples
    current_speed: u64,
    avg_speed: u64,
    peak_speed: u64,
    peers_connected: u8,
    peers_max: u8,
    pending: u32,
    inflight: u32,
    retry: u32,
    failed: u32,
    chain_head: u64,
    storage_total: f64, // GiB
    write_rate: f64,    // MB/s
    logs: Vec<(String, LogLevel)>,
    blocks_coverage: Vec<u8>, // 0-4 coverage level per segment

    // Compaction data
    total_shards: u32,
    compacted_shards: u32,
    current_shard: u32,
    shards_status: Vec<ShardStatus>, // Status of each shard

    // DB record counts
    db_blocks: u64,
    db_transactions: u64,
    db_receipts: u64,

    // Follow mode
    last_block_secs: u32,
    our_head: u64,  // Our synced head (may lag behind chain_head when catching up)
}

#[derive(Clone, Copy, PartialEq)]
enum ShardStatus {
    Pending,
    Compacting,
    Done,
}

#[derive(Clone, Copy)]
enum LogLevel {
    Info,
    Warn,
}

impl MockData {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let speed_history: Vec<u64> = (0..60).map(|_| rng.gen_range(800..1500)).collect();
        let blocks_coverage: Vec<u8> = (0..80)
            .map(|i| {
                if i < 50 {
                    4
                } else if i < 55 {
                    rng.gen_range(2..4)
                } else {
                    0
                }
            })
            .collect();

        Self {
            phase: Phase::Sync,
            progress: 0.0,
            start_block: 10_000_000,
            end_block: 21_000_000,
            current_block: 10_000_000,
            speed_history,
            current_speed: 1247,
            avg_speed: 1180,
            peak_speed: 1892,
            peers_connected: 6,
            peers_max: 10,
            pending: 284,
            inflight: 48,
            retry: 3,
            failed: 0,
            chain_head: 21_453_892,
            storage_total: 22.3,
            write_rate: 12.4,
            logs: vec![
                (
                    "14:32:05 INFO  Fetched batch 1847: blocks 13,245,000..13,245,128".into(),
                    LogLevel::Info,
                ),
                (
                    "14:32:04 INFO  Peer 0x7f3a..2c1b connected (eth/68)".into(),
                    LogLevel::Info,
                ),
                (
                    "14:32:03 INFO  Committed shard 1320 (13,200,000..13,209,999)".into(),
                    LogLevel::Info,
                ),
                (
                    "14:32:01 WARN  Peer 0x2d8c..9e4a timeout, retrying batch 1843".into(),
                    LogLevel::Warn,
                ),
                (
                    "14:31:58 INFO  Fetched batch 1846: blocks 13,244,872..13,245,000".into(),
                    LogLevel::Info,
                ),
                (
                    "14:31:55 INFO  Fetched batch 1845: blocks 13,244,744..13,244,872".into(),
                    LogLevel::Info,
                ),
                (
                    "14:31:52 INFO  Peer 0x9c2f..1d3e connected (eth/68)".into(),
                    LogLevel::Info,
                ),
                (
                    "14:31:50 INFO  Committed shard 1319 (13,190,000..13,199,999)".into(),
                    LogLevel::Info,
                ),
                (
                    "14:31:47 WARN  Slow peer 0x4a1b..8c2d, reducing batch size".into(),
                    LogLevel::Warn,
                ),
                (
                    "14:31:44 INFO  Fetched batch 1844: blocks 13,244,616..13,244,744".into(),
                    LogLevel::Info,
                ),
            ],
            blocks_coverage,

            // Compaction data
            total_shards: 110,
            compacted_shards: 0,
            current_shard: 0,
            shards_status: vec![ShardStatus::Pending; 110],

            // DB record counts
            db_blocks: 0,
            db_transactions: 0,
            db_receipts: 0,

            // Follow mode
            last_block_secs: 3,
            our_head: 21_453_892,  // Start synced with chain_head
        }
    }

    fn update(&mut self, dt: Duration) {
        let mut rng = rand::thread_rng();

        // Progress cycles 0->100% in ~5 seconds
        self.progress += dt.as_secs_f64() / 5.0;
        if self.progress > 1.0 {
            self.progress = 0.0;
        }

        // Update current block based on progress
        let range = self.end_block - self.start_block;
        self.current_block = self.start_block + (range as f64 * self.progress) as u64;

        // Update blocks coverage based on progress
        let coverage_pos = (self.blocks_coverage.len() as f64 * self.progress) as usize;
        for (i, coverage) in self.blocks_coverage.iter_mut().enumerate() {
            if i < coverage_pos {
                *coverage = 4;
            } else if i == coverage_pos {
                *coverage = rng.gen_range(2..4);
            } else if i == coverage_pos + 1 {
                *coverage = rng.gen_range(0..2);
            } else {
                *coverage = 0;
            }
        }

        // Vary speed
        self.current_speed = rng.gen_range(900..1600);
        if self.current_speed > self.peak_speed {
            self.peak_speed = self.current_speed;
        }

        // Update speed history (shift left, add new)
        self.speed_history.remove(0);
        self.speed_history.push(self.current_speed);

        // Vary other stats
        self.peers_connected = rng.gen_range(4..=self.peers_max);
        self.pending = rng.gen_range(200..400);
        self.inflight = rng.gen_range(30..60);
        self.retry = rng.gen_range(0..10);
        self.storage_total += dt.as_secs_f64() * 0.01;
        self.write_rate = rng.gen_range(8.0..16.0);

        // Update compaction progress based on progress
        self.compacted_shards = (self.total_shards as f64 * self.progress) as u32;
        self.current_shard = self.compacted_shards;
        for (i, status) in self.shards_status.iter_mut().enumerate() {
            let i = i as u32;
            if i < self.compacted_shards {
                *status = ShardStatus::Done;
            } else if i == self.compacted_shards {
                *status = ShardStatus::Compacting;
            } else {
                *status = ShardStatus::Pending;
            }
        }

        // Update DB record counts based on synced blocks
        self.db_blocks = self.synced_blocks();
        // Simulate ~150 transactions per block on average
        self.db_transactions = self.db_blocks * 150 + rng.gen_range(0..100);
        // Receipts = transactions (1:1)
        self.db_receipts = self.db_transactions;

        // Follow mode: simulate block arrivals every ~12 seconds
        self.last_block_secs = (self.last_block_secs + 1) % 15;
        if self.last_block_secs == 0 {
            self.chain_head += 1;
        }
        // Simulate catching up: our head lags behind, then catches up slowly
        // In follow mode, simulate being 0-5 blocks behind periodically
        if self.phase == Phase::Follow {
            // Every few seconds, fall behind a bit, then catch up
            if rng.gen_ratio(1, 50) && self.our_head >= self.chain_head {
                // Simulate falling behind by 3-8 blocks
                self.our_head = self.chain_head.saturating_sub(rng.gen_range(3..8));
            } else if self.our_head < self.chain_head {
                // Catch up one block at a time
                self.our_head += 1;
            } else {
                self.our_head = self.chain_head;
            }
        }
    }

    /// Returns true if we're fully synced (our head == chain head)
    const fn is_synced(&self) -> bool {
        self.our_head >= self.chain_head
    }

    /// Returns blocks behind (0 if synced)
    const fn blocks_behind(&self) -> u64 {
        self.chain_head.saturating_sub(self.our_head)
    }

    const fn synced_blocks(&self) -> u64 {
        self.current_block - self.start_block
    }

    const fn total_blocks(&self) -> u64 {
        self.end_block - self.start_block
    }

    fn eta_string(&self) -> String {
        if self.current_speed == 0 {
            return "∞".into();
        }
        let remaining = self.end_block - self.current_block;
        let secs = remaining / self.current_speed;
        if secs > 3600 {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        } else if secs > 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}s", secs)
        }
    }
}

// ============================================================================
// UI Rendering
// ============================================================================

fn render_phase_indicator(area: Rect, buf: &mut Buffer, current_phase: Phase) {
    let phases = [
        ("SYNC", Phase::Sync),
        ("RETRY", Phase::Retry),
        ("COMPACT", Phase::Compact),
        ("SEAL", Phase::Seal),
        ("FOLLOW", Phase::Follow),
    ];

    let mut x = area.x + 2;
    let text = "Phase:  ";
    buf.set_string(x, area.y, text, Style::default());
    x += text.len() as u16;

    for (name, phase) in phases {
        let (symbol, style) = if phase == current_phase {
            ("■", Style::default().fg(phase.color()).bold())
        } else if (phase as u8) < (current_phase as u8) {
            ("✓", Style::default().fg(Color::DarkGray))
        } else {
            ("□", Style::default().fg(Color::DarkGray))
        };

        let bracket_style = if phase == current_phase {
            Style::default().fg(phase.color())
        } else {
            Style::default().fg(Color::DarkGray)
        };

        buf.set_string(x, area.y, "[", bracket_style);
        x += 1;
        buf.set_string(x, area.y, symbol, style);
        x += 1;
        buf.set_string(x, area.y, " ", Style::default());
        x += 1;
        buf.set_string(
            x,
            area.y,
            name,
            if phase == current_phase {
                Style::default().fg(phase.color()).bold()
            } else {
                Style::default().fg(Color::DarkGray)
            },
        );
        x += name.len() as u16;
        buf.set_string(x, area.y, "]", bracket_style);
        x += 1;
        buf.set_string(x, area.y, "  ", Style::default());
        x += 2;
    }
}

fn render_progress_section(area: Rect, buf: &mut Buffer, data: &MockData) {
    // Layout: [Status    ] [████████░░░░░░░░] [XX%]
    // Row 1: progress bar with status and percentage
    // Row 2: synced / total blocks

    // Fixed label width for alignment with blocks map
    let label_width: u16 = 12;  // "Fetching" or "Blocks" padded
    let right_margin: u16 = 6;  // Space for " XXX%"

    let status_text = data.phase.status_text();
    let pct_text = format!("{:>3}%", (data.progress * 100.0) as u32);

    // Status on the left (padded to fixed width)
    buf.set_string(
        area.x + 2,
        area.y,
        status_text,
        Style::default().fg(data.phase.color()).bold(),
    );

    // Percentage on the right
    buf.set_string(
        area.x + area.width - right_margin,
        area.y,
        &pct_text,
        Style::default().fg(Color::White).bold(),
    );

    // Simple character-based progress bar in the middle
    let bar_start = area.x + 2 + label_width;
    let bar_end = area.x + area.width - right_margin - 1;
    let bar_width = bar_end.saturating_sub(bar_start) as usize;

    if bar_width > 4 {
        let filled = (bar_width as f64 * data.progress) as usize;
        let empty = bar_width - filled;

        // Draw filled portion
        let filled_str: String = "█".repeat(filled);
        buf.set_string(
            bar_start,
            area.y,
            &filled_str,
            Style::default().fg(data.phase.color()),
        );

        // Draw empty portion
        let empty_str: String = "░".repeat(empty);
        buf.set_string(
            bar_start + filled as u16,
            area.y,
            &empty_str,
            Style::default().fg(Color::DarkGray),
        );
    }

    // Row 2: synced / total blocks (centered)
    let blocks_text = format!(
        "{} / {} blocks synced",
        format_number(data.synced_blocks()),
        format_number(data.total_blocks())
    );
    let blocks_x = area.x + (area.width.saturating_sub(blocks_text.len() as u16)) / 2;
    buf.set_string(
        blocks_x,
        area.y + 1,
        &blocks_text,
        Style::default().fg(Color::White),
    );
}

fn render_blocks_map(area: Rect, buf: &mut Buffer, data: &MockData) {
    // Two rows of braille dots showing block coverage
    // Braille patterns: ⠀ (empty), ⠄, ⠆, ⠇, ⣿ (full)

    // Same alignment as progress bar
    let label_width: u16 = 12;  // Same as progress section
    let right_margin: u16 = 6;  // Same as progress section

    let label = "Blocks";
    buf.set_string(area.x + 2, area.y, label, Style::default().fg(Color::Gray));

    // Full block number labels
    let start_label = format_number(data.start_block);
    let end_label = format_number(data.end_block);

    // Align with progress bar
    let map_start = area.x + 2 + label_width;
    let map_end = area.x + area.width - right_margin - 1;
    let map_width = map_end.saturating_sub(map_start) as usize;

    // Scale coverage to fit width
    let scale = data.blocks_coverage.len() as f64 / map_width as f64;

    // Braille characters for different fill levels
    let braille_chars = ['⠀', '⠄', '⠆', '⠇', '⣿'];

    // Row 1
    for i in 0..map_width {
        let coverage_idx = (i as f64 * scale) as usize;
        let coverage = data.blocks_coverage.get(coverage_idx).copied().unwrap_or(0);
        let ch = braille_chars[coverage.min(4) as usize];
        let color = if coverage >= 4 {
            Color::Green
        } else if coverage >= 2 {
            Color::LightGreen
        } else if coverage >= 1 {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        buf.set_string(
            map_start + i as u16,
            area.y,
            ch.to_string(),
            Style::default().fg(color),
        );
    }

    // Row 2 (same pattern for visual consistency)
    for i in 0..map_width {
        let coverage_idx = (i as f64 * scale) as usize;
        let coverage = data.blocks_coverage.get(coverage_idx).copied().unwrap_or(0);
        let ch = braille_chars[coverage.min(4) as usize];
        let color = if coverage >= 4 {
            Color::Green
        } else if coverage >= 2 {
            Color::LightGreen
        } else if coverage >= 1 {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        buf.set_string(
            map_start + i as u16,
            area.y + 1,
            ch.to_string(),
            Style::default().fg(color),
        );
    }

    // Row 3: Full block number labels
    let label_y = area.y + 2;
    buf.set_string(
        map_start,
        label_y,
        &start_label,
        Style::default().fg(Color::DarkGray),
    );

    buf.set_string(
        map_start + map_width as u16 - end_label.len() as u16,
        label_y,
        &end_label,
        Style::default().fg(Color::DarkGray),
    );
}

fn render_shards_map(area: Rect, buf: &mut Buffer, data: &MockData) {
    // Two rows showing shard compaction status
    // ✓ = done, ◉ = compacting, ○ = pending

    // Same alignment as progress bar
    let label_width: u16 = 12;
    let right_margin: u16 = 6;

    let label = "Shards";
    buf.set_string(area.x + 2, area.y, label, Style::default().fg(Color::Gray));

    // Shard range labels
    let start_label = format!("Shard 0");
    let end_label = format!("Shard {}", data.total_shards - 1);

    // Align with progress bar
    let map_start = area.x + 2 + label_width;
    let map_end = area.x + area.width - right_margin - 1;
    let map_width = map_end.saturating_sub(map_start) as usize;

    // Scale shards to fit width
    let scale = data.shards_status.len() as f64 / map_width as f64;

    // Characters for different states
    // Row 1: block characters
    for i in 0..map_width {
        let shard_idx = (i as f64 * scale) as usize;
        let status = data.shards_status.get(shard_idx).copied().unwrap_or(ShardStatus::Pending);
        let (ch, color) = match status {
            ShardStatus::Done => ('█', Color::Magenta),
            ShardStatus::Compacting => ('▓', Color::Yellow),
            ShardStatus::Pending => ('░', Color::DarkGray),
        };
        buf.set_string(
            map_start + i as u16,
            area.y,
            ch.to_string(),
            Style::default().fg(color),
        );
    }

    // Row 2: same pattern
    for i in 0..map_width {
        let shard_idx = (i as f64 * scale) as usize;
        let status = data.shards_status.get(shard_idx).copied().unwrap_or(ShardStatus::Pending);
        let (ch, color) = match status {
            ShardStatus::Done => ('█', Color::Magenta),
            ShardStatus::Compacting => ('▓', Color::Yellow),
            ShardStatus::Pending => ('░', Color::DarkGray),
        };
        buf.set_string(
            map_start + i as u16,
            area.y + 1,
            ch.to_string(),
            Style::default().fg(color),
        );
    }

    // Row 3: Labels
    let label_y = area.y + 2;
    buf.set_string(
        map_start,
        label_y,
        &start_label,
        Style::default().fg(Color::DarkGray),
    );

    buf.set_string(
        map_start + map_width as u16 - end_label.len() as u16,
        label_y,
        &end_label,
        Style::default().fg(Color::DarkGray),
    );
}

fn render_compact_progress_section(area: Rect, buf: &mut Buffer, data: &MockData) {
    // Layout: [Status    ] [████████░░░░░░░░] [XX%]
    // Row 1: progress bar with status and percentage
    // Row 2: shards compacted / total

    let label_width: u16 = 12;
    let right_margin: u16 = 6;

    let status_text = data.phase.status_text();
    let pct_text = format!("{:>3}%", (data.progress * 100.0) as u32);

    // Status on the left
    buf.set_string(
        area.x + 2,
        area.y,
        status_text,
        Style::default().fg(data.phase.color()).bold(),
    );

    // Percentage on the right
    buf.set_string(
        area.x + area.width - right_margin,
        area.y,
        &pct_text,
        Style::default().fg(Color::White).bold(),
    );

    // Simple character-based progress bar
    let bar_start = area.x + 2 + label_width;
    let bar_end = area.x + area.width - right_margin - 1;
    let bar_width = bar_end.saturating_sub(bar_start) as usize;

    if bar_width > 4 {
        let filled = (bar_width as f64 * data.progress) as usize;
        let empty = bar_width - filled;

        let filled_str: String = "█".repeat(filled);
        buf.set_string(
            bar_start,
            area.y,
            &filled_str,
            Style::default().fg(data.phase.color()),
        );

        let empty_str: String = "░".repeat(empty);
        buf.set_string(
            bar_start + filled as u16,
            area.y,
            &empty_str,
            Style::default().fg(Color::DarkGray),
        );
    }

    // Row 2: shards compacted / total (centered)
    let shards_text = format!(
        "{} / {} shards compacted",
        data.compacted_shards,
        data.total_shards
    );
    let shards_x = area.x + (area.width.saturating_sub(shards_text.len() as u16)) / 2;
    buf.set_string(
        shards_x,
        area.y + 1,
        &shards_text,
        Style::default().fg(Color::White),
    );
}

fn render_compaction_panel(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" COMPACTION ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    // Done
    buf.set_string(inner.x + 1, inner.y, "Done", Style::default().fg(Color::Gray));
    buf.set_string(
        inner.x + 11,
        inner.y,
        &format!("{:>6}", data.compacted_shards),
        Style::default().fg(Color::Magenta),
    );

    // Current shard (or "Done" if finished)
    buf.set_string(inner.x + 1, inner.y + 1, "Current", Style::default().fg(Color::Gray));
    if data.compacted_shards < data.total_shards {
        buf.set_string(
            inner.x + 11,
            inner.y + 1,
            &format!("{:>6}", data.current_shard),
            Style::default().fg(Color::Yellow),
        );
    } else {
        buf.set_string(
            inner.x + 11,
            inner.y + 1,
            "     -",
            Style::default().fg(Color::DarkGray),
        );
    }

    // Pending
    let pending = data.total_shards.saturating_sub(data.compacted_shards).saturating_sub(1);
    buf.set_string(inner.x + 1, inner.y + 2, "Pending", Style::default().fg(Color::Gray));
    buf.set_string(
        inner.x + 11,
        inner.y + 2,
        &format!("{:>6}", pending),
        Style::default().fg(Color::White),
    );
}

fn render_speed_chart(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" Speed (blocks/s) ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    // Reserve space for Y-axis labels (6 chars like "2,000 ")
    let y_axis_width: u16 = 7;

    // Chart area (leave 2 rows for x-axis and stats, y_axis_width for labels)
    let chart_height = inner.height.saturating_sub(2);
    let chart_width = inner.width.saturating_sub(y_axis_width + 1);

    if chart_height == 0 || chart_width == 0 {
        return;
    }

    // Find max for scaling (always start from 0)
    let max_val = *data.speed_history.iter().max().unwrap_or(&1);
    // Round up to nice number for scale (nearest 500)
    let scale_max = ((max_val / 500) + 1) * 500;

    // Draw Y-axis labels (top, middle, bottom)
    for row in 0..chart_height {
        // Only show labels at top, middle, and bottom
        if row == 0 {
            // Top
            let label = format!("{:>6}", format_number(scale_max));
            buf.set_string(
                inner.x,
                inner.y + row,
                &label,
                Style::default().fg(Color::DarkGray),
            );
        } else if row == chart_height / 2 {
            // Middle
            let label = format!("{:>6}", format_number(scale_max / 2));
            buf.set_string(
                inner.x,
                inner.y + row,
                &label,
                Style::default().fg(Color::DarkGray),
            );
        } else if row == chart_height - 1 {
            // Bottom (0)
            buf.set_string(
                inner.x,
                inner.y + row,
                "     0",
                Style::default().fg(Color::DarkGray),
            );
        }
    }

    // Bar characters for different heights
    let bar_chars = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    // Sample data to fit width
    let chart_start_x = inner.x + y_axis_width;
    let samples_per_col = data.speed_history.len() as f64 / chart_width as f64;

    for col in 0..chart_width {
        let sample_idx = (col as f64 * samples_per_col) as usize;
        let value = data.speed_history.get(sample_idx).copied().unwrap_or(0);

        // Normalize to chart height (from 0 to scale_max)
        let normalized_f = value as f64 / scale_max as f64 * chart_height as f64;
        let full_rows = normalized_f as u16;
        let partial = ((normalized_f - full_rows as f64) * 8.0) as usize;

        // Draw column from bottom up
        for row in 0..chart_height {
            let y = inner.y + chart_height - 1 - row;
            let ch = if row < full_rows {
                '█'
            } else if row == full_rows && partial > 0 {
                bar_chars[partial]
            } else {
                ' '
            };

            buf.set_string(
                chart_start_x + col,
                y,
                ch.to_string(),
                Style::default().fg(data.phase.color()),
            );
        }
    }

    // Time axis labels
    let axis_y = inner.y + chart_height;
    buf.set_string(
        chart_start_x,
        axis_y,
        "-5m",
        Style::default().fg(Color::DarkGray),
    );
    buf.set_string(
        chart_start_x + chart_width - 3,
        axis_y,
        "now",
        Style::default().fg(Color::DarkGray),
    );

    // Speed stats line below chart
    let stats_y = inner.y + inner.height - 1;
    let mut x = inner.x + 1;

    buf.set_string(x, stats_y, "● Current: ", Style::default().fg(Color::Yellow));
    x += 11;
    buf.set_string(
        x,
        stats_y,
        &format!("{}/s", format_number(data.current_speed)),
        Style::default().fg(Color::Yellow),
    );
    x += 12;
    buf.set_string(x, stats_y, "◆ Average: ", Style::default().fg(Color::White));
    x += 11;
    buf.set_string(
        x,
        stats_y,
        &format!("{}/s", format_number(data.avg_speed)),
        Style::default().fg(Color::White),
    );
    x += 12;
    buf.set_string(
        x,
        stats_y,
        "★ Peak: ",
        Style::default().fg(Color::LightCyan),
    );
    x += 8;
    buf.set_string(
        x,
        stats_y,
        &format!("{}/s", format_number(data.peak_speed)),
        Style::default().fg(Color::LightCyan),
    );

    // ETA on the right
    let eta_text = format!("ETA: {}", data.eta_string());
    buf.set_string(
        inner.x + inner.width - eta_text.len() as u16 - 1,
        stats_y,
        &eta_text,
        Style::default().fg(Color::Magenta),
    );
}

fn render_network_panel(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" NETWORK ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    // Peers visual
    let peers_visual: String = (0..data.peers_max)
        .map(|i| if i < data.peers_connected { '●' } else { '○' })
        .collect();

    buf.set_string(
        inner.x + 1,
        inner.y,
        "Peers",
        Style::default().fg(Color::Gray),
    );
    buf.set_string(
        inner.x + 7,
        inner.y,
        &peers_visual,
        Style::default().fg(Color::Green),
    );

    buf.set_string(
        inner.x + 7,
        inner.y + 1,
        &format!("{} / {}", data.peers_connected, data.peers_max),
        Style::default().fg(Color::White),
    );

    buf.set_string(
        inner.x + 1,
        inner.y + 3,
        "↓ Rx",
        Style::default().fg(Color::Gray),
    );
    buf.set_string(
        inner.x + 7,
        inner.y + 3,
        "892 KB/s",
        Style::default().fg(Color::Green),
    );

    buf.set_string(
        inner.x + 1,
        inner.y + 4,
        "Chain Head",
        Style::default().fg(Color::Gray),
    );
    buf.set_string(
        inner.x + 12,
        inner.y + 4,
        &format_number(data.chain_head),
        Style::default().fg(Color::White),
    );
}

fn render_queue_panel(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" QUEUE ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let items = [
        ("Pending", data.pending, Color::White),
        ("Inflight", data.inflight, Color::Yellow),
        ("Retry", data.retry, Color::Rgb(255, 165, 0)), // Orange
        (
            "Failed",
            data.failed,
            if data.failed > 0 {
                Color::Red
            } else {
                Color::Green
            },
        ),
    ];

    for (i, (label, value, color)) in items.iter().enumerate() {
        buf.set_string(
            inner.x + 1,
            inner.y + i as u16,
            *label,
            Style::default().fg(Color::Gray),
        );
        buf.set_string(
            inner.x + 11,
            inner.y + i as u16,
            &format!("{:>6}", value),
            Style::default().fg(*color),
        );
    }
}

fn render_storage_panel(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" STORAGE ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 14;  // More padding from labels

    // Order: Headers, Transactions, Receipts
    let items = [("Headers", "2.4 GiB"), ("Transactions", "1.2 GiB"), ("Receipts", "18.7 GiB")];

    for (i, (label, value)) in items.iter().enumerate() {
        buf.set_string(
            inner.x + 1,
            inner.y + i as u16,
            *label,
            Style::default().fg(Color::Gray),
        );
        buf.set_string(
            value_col,
            inner.y + i as u16,
            *value,
            Style::default().fg(Color::White),
        );
    }

    // Separator
    buf.set_string(
        inner.x + 1,
        inner.y + 3,
        "─────────────────────",
        Style::default().fg(Color::DarkGray),
    );

    buf.set_string(
        inner.x + 1,
        inner.y + 4,
        "Total",
        Style::default().fg(Color::Gray),
    );
    buf.set_string(
        value_col,
        inner.y + 4,
        &format!("{:.1} GiB", data.storage_total),
        Style::default().fg(Color::White).bold(),
    );

    buf.set_string(
        inner.x + 1,
        inner.y + 5,
        "Write Rate",
        Style::default().fg(Color::Gray),
    );
    buf.set_string(
        value_col,
        inner.y + 5,
        &format!("{:.1} MB/s", data.write_rate),
        Style::default().fg(Color::Green),
    );
}

fn render_db_panel(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" DB ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 10;

    // Blocks
    buf.set_string(inner.x + 1, inner.y, "Blocks", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y,
        &format_number(data.db_blocks),
        Style::default().fg(Color::White),
    );

    // Transactions
    buf.set_string(inner.x + 1, inner.y + 1, "Txns", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 1,
        &format_number(data.db_transactions),
        Style::default().fg(Color::White),
    );

    // Receipts
    buf.set_string(inner.x + 1, inner.y + 2, "Receipts", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 2,
        &format_number(data.db_receipts),
        Style::default().fg(Color::White),
    );
}

fn render_logs_panel(area: Rect, buf: &mut Buffer, data: &MockData) {
    let block = Block::default()
        .title(" Logs ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    for (i, (log, level)) in data.logs.iter().take(inner.height as usize).enumerate() {
        let style = match level {
            LogLevel::Info => Style::default().fg(Color::Gray),
            LogLevel::Warn => Style::default().fg(Color::Yellow),
        };
        let truncated: String = log.chars().take(inner.width as usize - 1).collect();
        buf.set_string(inner.x + 1, inner.y + i as u16, &truncated, style);
    }
}

fn render_header(area: Rect, buf: &mut Buffer, _phase: Phase) {
    let now = chrono_time();
    let title = "STATELESS HISTORY NODE";
    buf.set_string(
        area.x + 2,
        area.y,
        title,
        Style::default().fg(Color::White).bold(),
    );
    buf.set_string(
        area.x + area.width - now.len() as u16 - 2,
        area.y,
        &now,
        Style::default().fg(Color::DarkGray),
    );
}

fn chrono_time() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs_only = secs % 60;
    format!("{hours:02}:{mins:02}:{secs_only:02}")
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn render_ui(frame: &mut Frame, data: &MockData) {
    let area = frame.area();

    // Main border
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
        .border_style(Style::default().fg(data.phase.color()));
    let inner = main_block.inner(area);
    frame.render_widget(main_block, area);

    match data.phase {
        Phase::Sync | Phase::Retry => render_sync_ui(frame, inner, data),
        Phase::Compact | Phase::Seal => render_compact_ui(frame, inner, data),
        Phase::Follow => render_follow_ui(frame, inner, data),
    }

    // Help text at very bottom
    let help = " Press 'q' to quit, 'n' for next phase ";
    frame.buffer_mut().set_string(
        area.x + 2,
        area.y + area.height - 1,
        help,
        Style::default().fg(Color::DarkGray),
    );
}

fn render_sync_ui(frame: &mut Frame, inner: Rect, data: &MockData) {
    // Fixed layout for SYNC phase - with speed chart
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Spacing
            Constraint::Length(1),  // Phase indicator
            Constraint::Length(1),  // Separator
            Constraint::Length(2),  // Progress bar + block count
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Blocks map (2 rows + labels)
            Constraint::Length(1),  // Separator
            Constraint::Length(7),  // Speed chart (5 rows + axis + stats)
            Constraint::Length(1),  // Separator
            Constraint::Length(8),  // Network/Queue/Storage panels
            Constraint::Length(1),  // Separator
            Constraint::Min(4),     // Logs (takes all remaining space)
        ])
        .split(inner);

    // Render sections
    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);

    // Separator line
    let sep = "━".repeat(chunks[3].width as usize);
    frame.buffer_mut().set_string(
        chunks[3].x,
        chunks[3].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_progress_section(chunks[4], frame.buffer_mut(), data);
    render_blocks_map(chunks[6], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[7].width as usize);
    frame.buffer_mut().set_string(
        chunks[7].x,
        chunks[7].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_speed_chart(chunks[8], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[9].width as usize);
    frame.buffer_mut().set_string(
        chunks[9].x,
        chunks[9].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    // Four-column layout for panels
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(chunks[10]);

    render_network_panel(panels[0], frame.buffer_mut(), data);
    render_queue_panel(panels[1], frame.buffer_mut(), data);
    render_storage_panel(panels[2], frame.buffer_mut(), data);
    render_db_panel(panels[3], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[11].width as usize);
    frame.buffer_mut().set_string(
        chunks[11].x,
        chunks[11].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_logs_panel(chunks[12], frame.buffer_mut(), data);
}

fn render_compact_ui(frame: &mut Frame, inner: Rect, data: &MockData) {
    // Fixed layout for COMPACT/SEAL phase - no speed chart, shards instead of blocks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Spacing
            Constraint::Length(1),  // Phase indicator
            Constraint::Length(1),  // Separator
            Constraint::Length(2),  // Progress bar + shard count
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Shards map (2 rows + labels)
            Constraint::Length(1),  // Separator
            Constraint::Length(8),  // Network/Compaction/Storage panels
            Constraint::Length(1),  // Separator
            Constraint::Min(4),     // Logs (takes all remaining space)
        ])
        .split(inner);

    // Render sections
    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);

    // Separator line
    let sep = "━".repeat(chunks[3].width as usize);
    frame.buffer_mut().set_string(
        chunks[3].x,
        chunks[3].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_compact_progress_section(chunks[4], frame.buffer_mut(), data);
    render_shards_map(chunks[6], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[7].width as usize);
    frame.buffer_mut().set_string(
        chunks[7].x,
        chunks[7].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    // Four-column layout for panels
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(chunks[8]);

    render_network_panel(panels[0], frame.buffer_mut(), data);
    render_compaction_panel(panels[1], frame.buffer_mut(), data);
    render_storage_panel(panels[2], frame.buffer_mut(), data);
    render_db_panel(panels[3], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[9].width as usize);
    frame.buffer_mut().set_string(
        chunks[9].x,
        chunks[9].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_logs_panel(chunks[10], frame.buffer_mut(), data);
}

fn render_follow_ui(frame: &mut Frame, inner: Rect, data: &MockData) {
    // Minimal layout for FOLLOW phase - just status, no progress/speed
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Spacing
            Constraint::Length(1),  // Phase indicator
            Constraint::Length(1),  // Separator
            Constraint::Length(7),  // Synced status section
            Constraint::Length(1),  // Separator
            Constraint::Length(8),  // Network/Storage/DB panels
            Constraint::Length(1),  // Separator
            Constraint::Min(4),     // Logs (takes all remaining space)
        ])
        .split(inner);

    // Render sections
    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);

    // Separator line
    let sep = "━".repeat(chunks[3].width as usize);
    frame.buffer_mut().set_string(
        chunks[3].x,
        chunks[3].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_synced_status(chunks[4], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[5].width as usize);
    frame.buffer_mut().set_string(
        chunks[5].x,
        chunks[5].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    // Three-column layout for panels (no Queue in follow mode)
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(chunks[6]);

    render_network_panel(panels[0], frame.buffer_mut(), data);
    render_storage_panel(panels[1], frame.buffer_mut(), data);
    render_db_panel(panels[2], frame.buffer_mut(), data);

    // Separator
    let sep = "━".repeat(chunks[7].width as usize);
    frame.buffer_mut().set_string(
        chunks[7].x,
        chunks[7].y,
        &sep,
        Style::default().fg(Color::DarkGray),
    );

    render_logs_panel(chunks[8], frame.buffer_mut(), data);
}

fn render_synced_status(area: Rect, buf: &mut Buffer, data: &MockData) {
    // Split into left (status) and right (big block number)
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    let left = halves[0];
    let right = halves[1];

    // Determine color based on sync state
    let number_color = if data.is_synced() {
        Color::LightGreen
    } else {
        Color::Yellow
    };

    // LEFT SIDE: Status info
    if data.is_synced() {
        // Big "✓ SYNCED" text
        let synced_text = "✓ SYNCED";
        buf.set_string(
            left.x + 4,
            left.y + 2,
            synced_text,
            Style::default().fg(Color::LightGreen).bold(),
        );

        // Last update
        buf.set_string(
            left.x + 4,
            left.y + 4,
            "Last block",
            Style::default().fg(Color::Gray),
        );
        buf.set_string(
            left.x + 15,
            left.y + 4,
            &format!("{}s ago", data.last_block_secs),
            Style::default().fg(Color::White),
        );
    } else {
        // "CATCHING UP" text
        let catching_text = "CATCHING UP";
        buf.set_string(
            left.x + 4,
            left.y + 2,
            catching_text,
            Style::default().fg(Color::Yellow).bold(),
        );

        // Blocks behind
        buf.set_string(
            left.x + 4,
            left.y + 4,
            &format!("{} blocks behind", data.blocks_behind()),
            Style::default().fg(Color::Yellow),
        );
    }

    // RIGHT SIDE: Big block number using ASCII art
    render_big_number(right, buf, data.our_head, number_color);
}

/// ASCII art digits (3 chars wide, 5 rows tall)
const DIGITS: [[&str; 5]; 10] = [
    ["█▀█", "█ █", "█ █", "█ █", "▀▀▀"], // 0
    [" ▀█", "  █", "  █", "  █", "  ▀"], // 1
    ["▀▀█", "  █", "█▀▀", "█  ", "▀▀▀"], // 2
    ["▀▀█", "  █", "▀▀█", "  █", "▀▀▀"], // 3
    ["█ █", "█ █", "▀▀█", "  █", "  ▀"], // 4
    ["█▀▀", "█  ", "▀▀█", "  █", "▀▀▀"], // 5
    ["█▀▀", "█  ", "█▀█", "█ █", "▀▀▀"], // 6
    ["▀▀█", "  █", "  █", "  █", "  ▀"], // 7
    ["█▀█", "█ █", "█▀█", "█ █", "▀▀▀"], // 8
    ["█▀█", "█ █", "▀▀█", "  █", "▀▀▀"], // 9
];

const COMMA: [&str; 5] = ["   ", "   ", "   ", " ▄ ", " ▀ "];

fn render_big_number(area: Rect, buf: &mut Buffer, number: u64, color: Color) {
    let formatted = format_number(number);
    let chars: Vec<char> = formatted.chars().collect();

    // Calculate total width (3 chars per digit/comma + 1 space between)
    let total_width: u16 = chars.len() as u16 * 4;

    // Start position (right-aligned with some padding)
    let start_x = area.x + area.width.saturating_sub(total_width + 2);
    let start_y = area.y + 1;

    let style = Style::default().fg(color);

    for (char_idx, ch) in chars.iter().enumerate() {
        let x = start_x + (char_idx as u16 * 4);

        let pattern: &[&str; 5] = if ch.is_ascii_digit() {
            let digit = ch.to_digit(10).unwrap_or(0) as usize;
            &DIGITS[digit]
        } else if *ch == ',' {
            &COMMA
        } else {
            continue;
        };

        for (row, line) in pattern.iter().enumerate() {
            if start_y + (row as u16) < area.y + area.height {
                buf.set_string(x, start_y + row as u16, *line, style);
            }
        }
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut data = MockData::new();
    let mut last_update = Instant::now();
    let tick_rate = Duration::from_millis(50); // 20 FPS

    loop {
        // Handle input
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('n') => {
                            data.phase = data.phase.next();
                            data.progress = 0.0;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Update data
        let now = Instant::now();
        let dt = now - last_update;
        last_update = now;
        data.update(dt);

        // Render
        terminal.draw(|frame| render_ui(frame, &data))?;
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
