//! Ratatui-based fullscreen TUI dashboard.
//!
//! This module provides an alternative to the indicatif progress bars,
//! rendering a fullscreen dashboard with detailed sync statistics.

// TUI rendering code has many format calls and casts that are intentional
#![expect(
    clippy::needless_borrows_for_generic_args,
    clippy::uninlined_format_args,
    clippy::cast_lossless,
    clippy::or_fun_call,
    clippy::comparison_chain,
    clippy::unused_self,
    reason = "TUI rendering code has many format/display operations"
)]


use std::collections::VecDeque;
use std::io::{self, Stdout};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Max time since last block before considering status stale in follow mode (30 seconds).
/// If we haven't received a block in this time, we're likely not truly synced.
const FOLLOW_STALENESS_THRESHOLD_MS: u64 = 30_000;

// ============================================================================
// Splash screen constants
// ============================================================================

const SPLASH_ART: &str = r#" ██████\\████████  ██████\\████████ ████████ ██     | ████████  ██████\  ██████\
██___\██  | ██  | ██__| ██  | ██  | ██__   | ██     | ██__   | ██___\██ ██___\██
██    \   | ██  | ██    ██  | ██  | ██  \  | ██     | ██  \   \██    \ \██    \
\██████\  | ██  | ████████  | ██  | █████  | ██     | █████   _\██████\_\██████\
 \__| ██  | ██  | ██  | ██  | ██  | ██_____| ██_____| ██_____|  \__| ██  \__| ██
██    ██  | ██  | ██  | ██  | ██  | ██     \ ██     \ ██     \\██    ██\██    ██
\██████    \██   \██   \██   \██   \████████\████████\████████ \██████  \██████
        |  \  |  \      \/      \|        \/      \|       \|  \    /  \
        | ██  | ██\██████  ██████\\████████  ██████\ ███████\\██\  /  ██
        | ██__| ██ | ██ | ██___\██  | ██  | ██  | ██ ██__| ██ \██\/  ██
        | ██    ██ | ██  \██    \   | ██  | ██  | ██ ██    ██  \██  ██
        | ████████ | ██  _\██████\  | ██  | ██  | ██ ███████\   \████
        | ██  | ██_| ██_|  \__| ██  | ██  | ██__/ ██ ██  | ██   | ██
        | ██  | ██   ██ \\██    ██  | ██   \██    ██ ██  | ██   | ██
         \██   \██\██████ \██████    \██    \██████ \██   \██    \██
                      |  \  |  \/      \|       \|        \
                     | ██\ | ██  ██████\ ███████\ ████████
                     | ███\| ██ ██  | ██ ██  | ██ ██__
                     | ████\ ██ ██  | ██ ██  | ██ ██  \
                     | ██\██ ██ ██  | ██ ██  | ██ █████
                     | ██ \████ ██__/ ██ ██__/ ██ ██_____
                     | ██  \███\██    ██ ██    ██ ██     \
                      \██   \██ \██████ \███████ \████████"#;

const CREDIT: &str = "2026 by @vicnaum";
const ART_WIDTH: u16 = 80;
const ART_HEIGHT: u16 = 23;

use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear},
};

use crate::sync::{FinalizePhase, SyncProgressSnapshot, SyncStatus};

// ============================================================================
// Phase enum for TUI display
// ============================================================================

/// Phase enum ordering is significant: lower values are earlier phases.
/// This ordering is used to prevent backwards phase transitions.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Phase {
    Startup = 0,
    Sync = 1,
    Retry = 2,
    Compact = 3,
    Seal = 4,
    Follow = 5,
}

impl Phase {
    pub const fn color(self) -> Color {
        match self {
            Self::Startup => Color::Yellow,
            Self::Sync => Color::Cyan,
            Self::Retry => Color::Red,
            Self::Compact => Color::Magenta,
            Self::Seal => Color::Green,
            Self::Follow => Color::LightGreen,
        }
    }

    pub const fn status_text(self) -> &'static str {
        match self {
            Self::Startup => "Starting",
            Self::Sync => "Fetching",
            Self::Retry => "Retrying",
            Self::Compact => "Compacting",
            Self::Seal => "Sealing",
            Self::Follow => "Following",
        }
    }
}

// ============================================================================
// TuiState - holds all display data
// ============================================================================

/// State for the TUI dashboard, derived from `SyncProgressSnapshot`.
pub struct TuiState {
    // From SyncProgressSnapshot (available now)
    pub phase: Phase,
    pub progress: f64,
    pub start_block: u64,
    pub end_block: u64,
    pub current_block: u64,
    pub current_speed: u64,
    pub avg_speed: u64,
    pub peak_speed: u64,
    pub peers_connected: u64,
    pub peers_max: u64,
    pub peers_stale: u64,
    pub pending: u64,
    pub inflight: u64,
    pub retry: u64,
    pub chain_head: u64,
    pub our_head: u64,
    pub total_shards: u64,
    pub compacted_shards: u64,
    pub current_shard: u64,
    /// Separate sealing counters (distinct from compaction).
    pub sealed_shards: u64,
    pub total_to_seal: u64,
    pub last_block_secs: u64,
    /// Unix timestamp in milliseconds of when we last received a block.
    pub last_block_received_ms: u64,

    // Extended tracking
    pub speed_history: VecDeque<u64>,

    // 30-second windowed average tracking: (timestamp, processed_blocks)
    avg_speed_window: VecDeque<(Instant, u64)>,

    // Animation frame counter (incremented on each draw)
    pub animation_frame: u64,

    // Startup phase state
    pub startup_status: String,
    pub best_head_seen: u64,

    // Config parameters (set once at startup)
    pub config_data_dir: String,
    pub config_shard_size: u64,
    pub config_rpc_bind: String,
    pub config_rollback_window: u64,

    // Quitting state
    pub quitting: bool,

    /// Total blocks to sync (missing blocks, NOT full range).
    /// This is updated from `queue` on first sync update to capture the actual work.
    /// Once set, it doesn't change (represents the total work for this session).
    pub blocks_to_sync: u64,

    // Placeholders (show "--" for now)
    pub storage_total: Option<f64>,
    pub write_rate: Option<f64>,
    pub db_blocks: Option<u64>,
    pub db_transactions: Option<u64>,
    pub db_receipts: Option<u64>,
    /// Coverage per bucket (0-100%) for blocks map visualization.
    /// Each bucket represents a range of blocks; value is percentage synced.
    pub coverage_buckets: Vec<u8>,
    #[expect(dead_code, reason = "placeholder for future feature")]
    pub shards_status: Vec<ShardStatus>,
    pub logs: VecDeque<LogEntry>,

    // RPC stats (for follow mode)
    pub rpc_active: bool,
    pub rpc_total_requests: u64,
    pub rpc_requests_per_sec: f64,
    pub rpc_get_logs: u64,
    pub rpc_get_block: u64,
    pub rpc_errors: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "placeholder for future feature")]
pub enum ShardStatus {
    Pending,
    Compacting,
    Done,
}

#[derive(Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    /// Milliseconds since logging started.
    pub timestamp_ms: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl TuiState {
    /// Create a new TuiState with initial values.
    pub fn new(start_block: u64, end_block: u64) -> Self {
        Self {
            phase: Phase::Startup,
            progress: 0.0,
            start_block,
            end_block,
            current_block: start_block,
            current_speed: 0,
            avg_speed: 0,
            peak_speed: 0,
            peers_connected: 0,
            peers_max: 0,
            peers_stale: 0,
            pending: 0,
            inflight: 0,
            retry: 0,
            chain_head: 0,
            our_head: start_block,
            total_shards: 0,
            compacted_shards: 0,
            current_shard: 0,
            sealed_shards: 0,
            total_to_seal: 0,
            last_block_secs: 0,
            last_block_received_ms: 0,
            speed_history: VecDeque::with_capacity(60),
            avg_speed_window: VecDeque::with_capacity(300), // 30 seconds at 100ms intervals
            animation_frame: 0,
            startup_status: "Connecting to Ethereum P2P network...".to_string(),
            best_head_seen: 0,
            config_data_dir: String::new(),
            config_shard_size: 0,
            config_rpc_bind: String::new(),
            config_rollback_window: 0,
            quitting: false,
            blocks_to_sync: 0,  // Will be set on first sync update
            storage_total: None,
            write_rate: None,
            db_blocks: None,
            db_transactions: None,
            db_receipts: None,
            coverage_buckets: Vec::new(),
            shards_status: Vec::new(),
            logs: VecDeque::new(),
            rpc_active: false,
            rpc_total_requests: 0,
            rpc_requests_per_sec: 0.0,
            rpc_get_logs: 0,
            rpc_get_block: 0,
            rpc_errors: 0,
        }
    }

    /// Update startup status message.
    #[expect(dead_code, reason = "helper for future startup status updates")]
    pub fn set_startup_status(&mut self, status: &str) {
        self.startup_status = status.to_string();
    }

    /// Mark startup as complete, transitioning to Sync phase.
    #[expect(dead_code, reason = "helper for explicit phase transition")]
    pub fn complete_startup(&mut self) {
        if self.phase == Phase::Startup {
            self.phase = Phase::Sync;
        }
    }

    /// Set configuration parameters for display.
    pub fn set_config(&mut self, data_dir: &str, shard_size: u64, rpc_bind: &str, rollback_window: u64) {
        self.config_data_dir = data_dir.to_string();
        self.config_shard_size = shard_size;
        self.config_rpc_bind = rpc_bind.to_string();
        self.config_rollback_window = rollback_window;
    }

    /// Drain logs from a TuiLogBuffer and add them to state.
    pub fn drain_log_buffer(&mut self, buffer: &crate::logging::TuiLogBuffer) {
        for entry in buffer.drain() {
            self.add_log(entry.level, entry.message, entry.timestamp_ms);
        }
    }

    /// Add a log entry from a tracing Level.
    pub fn add_log(&mut self, level: tracing::Level, message: String, timestamp_ms: u64) {
        let log_level = match level {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG | tracing::Level::TRACE => LogLevel::Debug,
        };
        // Keep only the last 100 logs
        if self.logs.len() >= 100 {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            message,
            level: log_level,
            timestamp_ms,
        });
    }

    /// Update state from a `SyncProgressSnapshot`.
    #[expect(clippy::cognitive_complexity, reason = "sync state update is inherently complex")]
    pub fn update_from_snapshot(&mut self, snapshot: &SyncProgressSnapshot, current_speed: u64) {
        // Determine phase from snapshot
        let new_phase = match snapshot.status {
            SyncStatus::LookingForPeers | SyncStatus::Fetching => {
                if snapshot.queue == 0 && snapshot.escalation > 0 && !snapshot.fetch_complete {
                    Phase::Retry
                } else {
                    // If we were at SEAL or later and now fetching, we're in follow mode catching up
                    // Show FOLLOW (catching up) instead of SYNC to avoid confusing backwards transition
                    if self.phase >= Phase::Seal {
                        Phase::Follow
                    } else {
                        Phase::Sync
                    }
                }
            }
            SyncStatus::Finalizing => match snapshot.finalize_phase {
                FinalizePhase::Compacting => Phase::Compact,
                FinalizePhase::Sealing => Phase::Seal,
            },
            SyncStatus::UpToDate | SyncStatus::Following => Phase::Follow,
        };

        // Prevent backwards phase transitions to avoid race conditions
        // Only allow forward transitions (higher ordinal)
        // Exceptions:
        // - Startup can transition to any phase
        // - Transition to Follow is always allowed (follow mode catching up)
        if self.phase != Phase::Startup
            && new_phase != Phase::Follow
            && (new_phase as u8) < (self.phase as u8)
        {
            // Don't go backwards - keep current phase
        } else {
            self.phase = new_phase;
        }

        // Update block tracking
        if snapshot.start_block > 0 {
            self.start_block = snapshot.start_block;
        }
        // snapshot.processed is a COUNT of blocks processed, not an absolute block number
        // Convert to absolute block number by adding start_block
        self.current_block = self.start_block.saturating_add(snapshot.processed);
        self.our_head = snapshot.head_block;
        self.chain_head = snapshot.head_seen;

        // Capture blocks_to_sync on first meaningful update
        // This represents the ACTUAL work to do (missing blocks), not the full range
        // Formula: total_work = processed + queue + inflight + escalation (retry)
        // We capture the max seen to handle race conditions during startup
        let current_total = snapshot.processed
            .saturating_add(snapshot.queue)
            .saturating_add(snapshot.inflight)
            .saturating_add(snapshot.escalation);
        if current_total > self.blocks_to_sync {
            self.blocks_to_sync = current_total;
        }

        // Calculate progress based on current phase
        // - Sync/Retry: based on blocks processed vs blocks to sync
        // - Compact: based on compactions_done vs compactions_total
        // - Seal: based on sealings_done vs sealings_total
        match self.phase {
            Phase::Sync | Phase::Retry | Phase::Startup => {
                if self.blocks_to_sync > 0 {
                    self.progress = (snapshot.processed as f64 / self.blocks_to_sync as f64).clamp(0.0, 1.0);
                }
            }
            Phase::Compact => {
                if snapshot.compactions_total > 0 {
                    self.progress = (snapshot.compactions_done as f64 / snapshot.compactions_total as f64).clamp(0.0, 1.0);
                } else {
                    self.progress = 1.0; // No compactions needed = 100%
                }
            }
            Phase::Seal => {
                if snapshot.sealings_total > 0 {
                    self.progress = (snapshot.sealings_done as f64 / snapshot.sealings_total as f64).clamp(0.0, 1.0);
                } else {
                    self.progress = 1.0; // No sealings needed = 100%
                }
            }
            Phase::Follow => {
                self.progress = 1.0; // Following = 100%
            }
        }

        // RPC active flag is set from snapshot (signaled when RPC server starts)
        self.rpc_active = snapshot.rpc_active;

        // Speed tracking
        self.current_speed = current_speed;
        self.peak_speed = snapshot.peak_speed.max(self.peak_speed);

        // Update speed history (keep last 60 samples for chart)
        self.speed_history.push_back(current_speed);
        while self.speed_history.len() > 60 {
            self.speed_history.pop_front();
        }

        // Calculate 30-second windowed average speed
        let now = Instant::now();
        let processed = snapshot.processed;
        self.avg_speed_window.push_back((now, processed));

        // Remove entries older than 30 seconds
        let window_duration = Duration::from_secs(30);
        while let Some((t, _)) = self.avg_speed_window.front() {
            if now.duration_since(*t) > window_duration {
                self.avg_speed_window.pop_front();
            } else {
                break;
            }
        }

        // Calculate average from window
        if let (Some((t_start, p_start)), Some((t_end, p_end))) =
            (self.avg_speed_window.front(), self.avg_speed_window.back())
        {
            let elapsed_secs = t_end.duration_since(*t_start).as_secs_f64();
            if elapsed_secs > 0.5 {
                // Need at least 0.5s of data
                let blocks_processed = p_end.saturating_sub(*p_start);
                self.avg_speed = (blocks_processed as f64 / elapsed_secs) as u64;
            }
        }

        // Peer tracking
        self.peers_connected = snapshot.peers_active;
        self.peers_max = snapshot.peers_total;
        self.peers_stale = snapshot.peers_stale;

        // Queue tracking
        self.pending = snapshot.queue;
        self.inflight = snapshot.inflight;
        self.retry = snapshot.escalation;
        // failed stays as placeholder for now

        // Compaction tracking (separate from sealing)
        self.total_shards = snapshot.compactions_total;
        self.compacted_shards = snapshot.compactions_done;
        self.current_shard = snapshot.compactions_done;

        // Sealing tracking (separate counters)
        self.sealed_shards = snapshot.sealings_done;
        self.total_to_seal = snapshot.sealings_total;

        // Last block timing
        if snapshot.last_block_received_ms > 0 {
            self.last_block_received_ms = snapshot.last_block_received_ms;
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            self.last_block_secs = now_ms.saturating_sub(snapshot.last_block_received_ms) / 1000;
        }
    }

    /// Returns blocks processed in current sync session.
    /// This is the count of blocks fetched, starting from 0.
    pub const fn synced_blocks(&self) -> u64 {
        self.current_block.saturating_sub(self.start_block)
    }

    /// Returns total blocks to sync (missing blocks, NOT full range).
    /// This is the actual work to be done in this session.
    /// Progress = synced_blocks() / total_blocks() = 100% when complete.
    pub const fn total_blocks(&self) -> u64 {
        self.blocks_to_sync
    }

    /// Returns the full requested range size (for display labels).
    /// This is end_block - start_block, regardless of how many blocks already exist.
    #[expect(dead_code, reason = "available for future use in blocks map labels")]
    pub const fn full_range(&self) -> u64 {
        self.end_block.saturating_sub(self.start_block)
    }

    /// Returns true if we're fully synced (our head >= chain head AND data is fresh).
    ///
    /// In follow mode, we also check that we've received a block recently.
    /// If `last_block_received_ms` is too old (>30 seconds), we're not truly synced.
    pub fn is_synced(&self) -> bool {
        // Basic check: our head must be at or past chain head
        if self.our_head < self.chain_head {
            return false;
        }

        // In follow mode, also check if data is fresh
        if self.phase == Phase::Follow {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            // If last block was too long ago, we're not really synced
            if self.last_block_received_ms > 0
                && now_ms.saturating_sub(self.last_block_received_ms) > FOLLOW_STALENESS_THRESHOLD_MS
            {
                return false;
            }
        }

        true
    }

    /// Returns blocks behind (0 if synced).
    pub const fn blocks_behind(&self) -> u64 {
        self.chain_head.saturating_sub(self.our_head)
    }

    /// Format ETA string.
    pub fn eta_string(&self) -> String {
        if self.current_speed == 0 {
            return "--".into();
        }
        // Remaining = total_blocks (work to do) - synced_blocks (work done)
        let remaining = self.total_blocks().saturating_sub(self.synced_blocks());
        let secs = remaining as f64 / self.current_speed as f64;
        crate::sync::format_eta_seconds(secs)
    }

}

// ============================================================================
// TuiController - manages the terminal and rendering
// ============================================================================

/// Main TUI controller that manages the terminal and rendering.
pub struct TuiController {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    pub state: TuiState,
    pub should_quit: bool,
}

impl TuiController {
    /// Create a new TuiController and enter alternate screen mode.
    pub fn new(start_block: u64, end_block: u64) -> io::Result<Self> {
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        Ok(Self {
            terminal,
            state: TuiState::new(start_block, end_block),
            should_quit: false,
        })
    }

    /// Poll for keyboard events (non-blocking).
    /// Returns true if 'q' was pressed.
    pub fn poll_quit(&mut self) -> io::Result<bool> {
        if event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.should_quit = true;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Draw the current state to the terminal.
    pub fn draw(&mut self) -> io::Result<()> {
        // Increment animation frame for sparkle effects
        self.state.animation_frame = self.state.animation_frame.wrapping_add(1);
        self.terminal.draw(|frame| render_ui(frame, &self.state))?;
        Ok(())
    }

    /// Restore terminal to normal state.
    pub fn restore(&self) -> io::Result<()> {
        disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;
        io::stdout().execute(Show)?; // Show cursor
        Ok(())
    }
}

impl Drop for TuiController {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

// ============================================================================
// Rendering functions (adapted from ui_mock.rs)
// ============================================================================

fn render_ui(frame: &mut Frame, data: &TuiState) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    if data.phase == Phase::Startup && data.peers_connected == 0 {
        render_splash(frame, data);
    } else {
        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(data.phase.color()))
            .style(Style::default().bg(Color::Rgb(20, 20, 28)));
        let inner = main_block.inner(area);
        frame.render_widget(main_block, area);

        match data.phase {
            Phase::Startup => render_startup_ui(frame, inner, data),
            Phase::Sync | Phase::Retry => render_sync_ui(frame, inner, data),
            Phase::Compact | Phase::Seal => render_compact_ui(frame, inner, data),
            Phase::Follow => render_follow_ui(frame, inner, data),
        }
    }

    // Render quit overlay if quitting
    if data.quitting {
        render_quit_overlay(frame, area);
    }

    // Help text at very bottom
    let help = " Press 'q' to quit ";
    frame.buffer_mut().set_string(
        area.x + 2,
        area.y + area.height - 1,
        help,
        Style::default().fg(Color::DarkGray),
    );
}

fn render_sync_ui(frame: &mut Frame, inner: Rect, data: &TuiState) {
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

    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);
    render_separator(chunks[3], frame.buffer_mut());
    render_progress_section(chunks[4], frame.buffer_mut(), data);
    render_blocks_map(chunks[6], frame.buffer_mut(), data);
    render_separator(chunks[7], frame.buffer_mut());
    render_speed_chart(chunks[8], frame.buffer_mut(), data);
    render_separator(chunks[9], frame.buffer_mut());

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

    render_separator(chunks[11], frame.buffer_mut());
    render_logs_panel(chunks[12], frame.buffer_mut(), data);
}

fn render_compact_ui(frame: &mut Frame, inner: Rect, data: &TuiState) {
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
            Constraint::Min(4),     // Logs
        ])
        .split(inner);

    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);
    render_separator(chunks[3], frame.buffer_mut());
    render_compact_progress_section(chunks[4], frame.buffer_mut(), data);
    render_shards_map(chunks[6], frame.buffer_mut(), data);
    render_separator(chunks[7], frame.buffer_mut());

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

    render_separator(chunks[9], frame.buffer_mut());
    render_logs_panel(chunks[10], frame.buffer_mut(), data);
}

fn render_follow_ui(frame: &mut Frame, inner: Rect, data: &TuiState) {
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
            Constraint::Min(4),     // Logs
        ])
        .split(inner);

    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);
    render_separator(chunks[3], frame.buffer_mut());
    render_synced_status(chunks[4], frame.buffer_mut(), data);
    render_separator(chunks[5], frame.buffer_mut());

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(chunks[6]);

    render_network_panel(panels[0], frame.buffer_mut(), data);
    render_storage_panel(panels[1], frame.buffer_mut(), data);
    render_db_panel(panels[2], frame.buffer_mut(), data);
    render_rpc_panel(panels[3], frame.buffer_mut(), data);

    render_separator(chunks[7], frame.buffer_mut());
    render_logs_panel(chunks[8], frame.buffer_mut(), data);
}

fn render_splash(frame: &mut Frame, data: &TuiState) {
    let area = frame.area();

    let dos_blue = Color::Rgb(0, 0, 170);
    let dos_gold = Color::Rgb(255, 255, 85);

    // Fill entire screen with DOS blue
    let bg = Block::default().style(Style::default().bg(dos_blue));
    frame.render_widget(bg, area);

    let buf = frame.buffer_mut();
    let art_style = Style::default().fg(dos_gold).bg(dos_blue);

    let lines: Vec<&str> = SPLASH_ART.lines().collect();

    // Center the logo (art + 1 credit line = 24 lines total)
    let total_height = ART_HEIGHT + 1;
    let start_y = area.y + area.height.saturating_sub(total_height) / 2;
    let start_x = area.x + area.width.saturating_sub(ART_WIDTH) / 2;

    // Build status message with animated dots
    let base_status = data.startup_status.trim_end_matches('.');
    let max_dots: usize = 3;
    let dot_count = (data.animation_frame / 5) % (max_dots as u64 + 1);
    let dots = ".".repeat(dot_count as usize);
    let padding = " ".repeat(max_dots - dot_count as usize);
    let status_msg = format!("{}{}{}", base_status, dots, padding);
    let status_style = Style::default().fg(Color::LightCyan).bg(dos_blue);

    // Two modes: framed (big terminal) vs unframed (small terminal)
    let framed = area.width >= ART_WIDTH + 4 && area.height >= total_height + 4;
    let framed_with_status = framed && area.height >= total_height + 4 + 5;

    let (art_x, art_y, frame_bottom_y) = if framed {
        let frame_w = ART_WIDTH + 4;
        let frame_h = total_height + 4;
        let frame_x = area.x + area.width.saturating_sub(frame_w) / 2;
        let frame_y = area.y + area.height.saturating_sub(frame_h) / 2;

        let frame_style = Style::default().fg(dos_gold).bg(dos_blue);

        // Draw double-line border
        let horiz: String = "═".repeat(frame_w as usize - 2);
        buf.set_string(frame_x, frame_y, format!("╔{}╗", horiz), frame_style);
        buf.set_string(frame_x, frame_y + frame_h - 1, format!("╚{}╝", horiz), frame_style);
        for row in 1..frame_h - 1 {
            buf.set_string(frame_x, frame_y + row, "║", frame_style);
            buf.set_string(frame_x + frame_w - 1, frame_y + row, "║", frame_style);
        }

        (frame_x + 2, frame_y + 2, frame_y + frame_h - 1)
    } else {
        (start_x, start_y, 0)
    };

    // Draw art lines
    for (i, line) in lines.iter().enumerate() {
        let y = art_y + i as u16;
        if y < area.y + area.height {
            buf.set_string(art_x, y, *line, art_style);
        }
    }

    // Version — gold, right-aligned on the last art line
    let last_art_y = art_y + ART_HEIGHT - 1;
    if last_art_y < area.y + area.height {
        let version = "v0.3";
        let version_x = art_x + ART_WIDTH - version.len() as u16;
        buf.set_string(version_x, last_art_y, version, art_style);
    }

    // Credit line — white, right-aligned on the line after the art
    let credit_y = art_y + ART_HEIGHT;
    if credit_y < area.y + area.height {
        let credit_x = art_x + ART_WIDTH - CREDIT.len() as u16;
        buf.set_string(
            credit_x,
            credit_y,
            CREDIT,
            Style::default().fg(Color::White).bg(dos_blue),
        );
    }

    // Status message placement depends on mode
    if framed_with_status {
        // Big screen: centered below the frame, with 1 line padding
        let status_x = area.x + area.width.saturating_sub(status_msg.len() as u16) / 2;
        let status_y = frame_bottom_y + 2;
        if status_y < area.y + area.height {
            buf.set_string(status_x, status_y, &status_msg, status_style);
        }
    } else {
        // Small screen: left-aligned on the credit line
        let status_y = credit_y;
        if status_y < area.y + area.height {
            buf.set_string(art_x, status_y, &status_msg, status_style);
        }
    }
}

fn render_startup_ui(frame: &mut Frame, inner: Rect, data: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Spacing
            Constraint::Length(1),  // Phase indicator
            Constraint::Length(1),  // Separator
            Constraint::Length(3),  // Startup status
            Constraint::Length(1),  // Separator
            Constraint::Length(8),  // Network + Config panels
            Constraint::Length(1),  // Separator
            Constraint::Min(4),     // Logs
        ])
        .split(inner);

    render_header(chunks[0], frame.buffer_mut(), data.phase);
    render_phase_indicator(chunks[2], frame.buffer_mut(), data.phase);
    render_separator(chunks[3], frame.buffer_mut());
    render_startup_status(chunks[4], frame.buffer_mut(), data);
    render_separator(chunks[5], frame.buffer_mut());

    // Two side-by-side panels: NETWORK and CONFIG
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(chunks[6]);

    render_network_panel(panels[0], frame.buffer_mut(), data);
    render_config_panel(panels[1], frame.buffer_mut(), data);

    render_separator(chunks[7], frame.buffer_mut());
    render_logs_panel(chunks[8], frame.buffer_mut(), data);
}

fn render_startup_status(area: Rect, buf: &mut Buffer, data: &TuiState) {
    // Center the status message
    let status = &data.startup_status;
    let x = area.x + (area.width.saturating_sub(status.len() as u16)) / 2;

    buf.set_string(
        x,
        area.y,
        status,
        Style::default().fg(Color::Yellow).bold(),
    );

    // Show best head seen (or waiting message)
    let head_text = if data.best_head_seen > 0 {
        format!("Best chain head seen: {}", format_number(data.best_head_seen))
    } else {
        "Waiting for peer heads...".to_string()
    };
    let head_x = area.x + (area.width.saturating_sub(head_text.len() as u16)) / 2;
    buf.set_string(
        head_x,
        area.y + 1,
        &head_text,
        Style::default().fg(if data.best_head_seen > 0 { Color::White } else { Color::DarkGray }),
    );
}

#[expect(clippy::cognitive_complexity, reason = "ASCII overlay rendering is verbose but straightforward")]
fn render_quit_overlay(frame: &mut Frame, area: Rect) {
    use ratatui::widgets::Clear;

    let overlay_width: u16 = 30;
    let overlay_height: u16 = 3;
    let overlay_x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let overlay_y = area.y + (area.height.saturating_sub(overlay_height)) / 2;

    let overlay_area = Rect {
        x: overlay_x,
        y: overlay_y,
        width: overlay_width,
        height: overlay_height,
    };

    // Clear the overlay area
    frame.render_widget(Clear, overlay_area);

    // Draw directly to buffer with simple ASCII border
    let buf = frame.buffer_mut();
    let bg = Color::Rgb(40, 40, 40);
    let border_color = Color::Yellow;
    let text_color = Color::Yellow;

    // Fill entire area with background
    for y in overlay_area.y..overlay_area.y + overlay_area.height {
        for x in overlay_area.x..overlay_area.x + overlay_area.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(bg);
                cell.set_fg(text_color);
            }
        }
    }

    // Draw simple ASCII border: +--+ | | +--+
    let top = overlay_area.y;
    let bottom = overlay_area.y + overlay_area.height - 1;
    let left = overlay_area.x;
    let right = overlay_area.x + overlay_area.width - 1;

    // Corners
    if let Some(cell) = buf.cell_mut((left, top)) {
        cell.set_char('+');
        cell.set_fg(border_color);
    }
    if let Some(cell) = buf.cell_mut((right, top)) {
        cell.set_char('+');
        cell.set_fg(border_color);
    }
    if let Some(cell) = buf.cell_mut((left, bottom)) {
        cell.set_char('+');
        cell.set_fg(border_color);
    }
    if let Some(cell) = buf.cell_mut((right, bottom)) {
        cell.set_char('+');
        cell.set_fg(border_color);
    }

    // Horizontal lines
    for x in (left + 1)..right {
        if let Some(cell) = buf.cell_mut((x, top)) {
            cell.set_char('-');
            cell.set_fg(border_color);
        }
        if let Some(cell) = buf.cell_mut((x, bottom)) {
            cell.set_char('-');
            cell.set_fg(border_color);
        }
    }

    // Vertical lines
    for y in (top + 1)..bottom {
        if let Some(cell) = buf.cell_mut((left, y)) {
            cell.set_char('|');
            cell.set_fg(border_color);
        }
        if let Some(cell) = buf.cell_mut((right, y)) {
            cell.set_char('|');
            cell.set_fg(border_color);
        }
    }

    // Center text
    let text = "Quitting... please wait";
    let text_x = overlay_area.x + (overlay_area.width.saturating_sub(text.len() as u16)) / 2;
    let text_y = overlay_area.y + overlay_area.height / 2;
    for (i, ch) in text.chars().enumerate() {
        if let Some(cell) = buf.cell_mut((text_x + i as u16, text_y)) {
            cell.set_char(ch);
            cell.set_fg(text_color);
            cell.set_bg(bg);
        }
    }
}

fn render_separator(area: Rect, buf: &mut Buffer) {
    let sep = "\u{2501}".repeat(area.width as usize);
    buf.set_string(area.x, area.y, &sep, Style::default().fg(Color::DarkGray));
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

fn render_phase_indicator(area: Rect, buf: &mut Buffer, current_phase: Phase) {
    let phases = [
        ("STARTUP", Phase::Startup),
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
            ("\u{25A0}", Style::default().fg(phase.color()).bold())
        } else if (phase as u8) < (current_phase as u8) {
            ("\u{2713}", Style::default().fg(Color::DarkGray))
        } else {
            ("\u{25A1}", Style::default().fg(Color::DarkGray))
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

fn render_progress_section(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let label_width: u16 = 12;
    let right_margin: u16 = 6;

    let status_text = data.phase.status_text();
    let pct_text = format!("{:>3}%", (data.progress * 100.0) as u32);

    buf.set_string(
        area.x + 2,
        area.y,
        status_text,
        Style::default().fg(data.phase.color()).bold(),
    );

    buf.set_string(
        area.x + area.width - right_margin,
        area.y,
        &pct_text,
        Style::default().fg(Color::White).bold(),
    );

    let bar_start = area.x + 2 + label_width;
    let bar_end = area.x + area.width - right_margin - 1;
    let bar_width = bar_end.saturating_sub(bar_start) as usize;

    if bar_width > 4 {
        let filled = (bar_width as f64 * data.progress) as usize;
        let empty = bar_width - filled;

        let filled_str: String = "\u{2588}".repeat(filled);
        buf.set_string(
            bar_start,
            area.y,
            &filled_str,
            Style::default().fg(data.phase.color()),
        );

        let empty_str: String = "\u{2591}".repeat(empty);
        buf.set_string(
            bar_start + filled as u16,
            area.y,
            &empty_str,
            Style::default().fg(Color::DarkGray),
        );
    }

    let blocks_text = format!(
        "synced {} of {} blocks",
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

fn render_compact_progress_section(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let label_width: u16 = 12;
    let right_margin: u16 = 6;

    let status_text = data.phase.status_text();
    let pct_text = format!("{:>3}%", (data.progress * 100.0) as u32);

    buf.set_string(
        area.x + 2,
        area.y,
        status_text,
        Style::default().fg(data.phase.color()).bold(),
    );

    buf.set_string(
        area.x + area.width - right_margin,
        area.y,
        &pct_text,
        Style::default().fg(Color::White).bold(),
    );

    let bar_start = area.x + 2 + label_width;
    let bar_end = area.x + area.width - right_margin - 1;
    let bar_width = bar_end.saturating_sub(bar_start) as usize;

    if bar_width > 4 {
        let filled = (bar_width as f64 * data.progress) as usize;
        let empty = bar_width - filled;

        let filled_str: String = "\u{2588}".repeat(filled);
        buf.set_string(
            bar_start,
            area.y,
            &filled_str,
            Style::default().fg(data.phase.color()),
        );

        let empty_str: String = "\u{2591}".repeat(empty);
        buf.set_string(
            bar_start + filled as u16,
            area.y,
            &empty_str,
            Style::default().fg(Color::DarkGray),
        );
    }

    // Show different counters and text based on phase
    let shards_text = match data.phase {
        Phase::Compact => format!(
            "{} / {} shards compacted",
            data.compacted_shards,
            data.total_shards
        ),
        Phase::Seal => format!(
            "{} / {} shards sealed",
            data.sealed_shards,
            data.total_to_seal
        ),
        _ => format!(
            "{} / {} shards",
            data.compacted_shards,
            data.total_shards
        ),
    };
    let shards_x = area.x + (area.width.saturating_sub(shards_text.len() as u16)) / 2;
    buf.set_string(
        shards_x,
        area.y + 1,
        &shards_text,
        Style::default().fg(Color::White),
    );
}

fn render_blocks_map(area: Rect, buf: &mut Buffer, data: &TuiState) {
    // Two rows of braille dots showing block coverage
    // Braille patterns: ⠀ (empty), ⣿ (full)

    let label_width: u16 = 12;
    let right_margin: u16 = 6;

    buf.set_string(area.x + 2, area.y, "Blocks", Style::default().fg(Color::Gray));
    buf.set_string(area.x + 2, area.y + 1, "Map", Style::default().fg(Color::Gray));

    let start_label = format_number(data.start_block);
    let end_label = format_number(data.end_block);

    let map_start = area.x + 2 + label_width;
    let map_end = area.x + area.width - right_margin - 1;
    let map_width = map_end.saturating_sub(map_start) as usize;

    if map_width == 0 {
        return;
    }

    // 2 rows x map_width = total visual cells
    // Layout is column-major (top-bottom, left-right):
    //   Col 0: buckets 0,1  Col 1: buckets 2,3  Col 2: buckets 4,5 ...
    //   Row 0: bucket 0     Row 0: bucket 2     Row 0: bucket 4
    //   Row 1: bucket 1     Row 1: bucket 3     Row 1: bucket 5
    let total_cells = 2 * map_width;
    let has_coverage = !data.coverage_buckets.is_empty();
    let num_buckets = data.coverage_buckets.len();

    for col in 0..map_width {
        for row in 0..2 {
            // Column-major: cell_idx = col * 2 + row
            let cell_idx = col * 2 + row;

            // Scale cell position to bucket index (buckets may differ from cells)
            let bucket_idx = if num_buckets > 0 {
                (cell_idx * num_buckets / total_cells).min(num_buckets - 1)
            } else {
                0
            };

            let coverage_pct = if has_coverage && bucket_idx < num_buckets {
                data.coverage_buckets[bucket_idx]
            } else if has_coverage {
                // Out of bounds, treat as not synced
                0
            } else {
                // Fallback: use simple progress (synced / total)
                let total = data.total_blocks();
                let synced = data.synced_blocks();
                if total > 0 {
                    let cell_progress = cell_idx as f64 / total_cells as f64;
                    let sync_progress = synced as f64 / total as f64;
                    if cell_progress < sync_progress { 100 } else { 0 }
                } else {
                    0
                }
            };

            // Braille characters for different fill levels:
            // ⠀ (empty), ⠄ (1 dot), ⠆ (2 dots), ⠇ (3 dots), ⣿ (full)
            // Note: macOS Terminal ignores RGB colors on braille chars,
            // so we use ANSI named colors only. No explicit bg — inherits
            // from the app background fill.
            let (ch, fg) = match coverage_pct {
                0 => ('⣿', Color::DarkGray),           // Dim grid — empty region
                1..=20 => ('⠄', Color::Yellow),        // 1 dot - starting
                21..=40 => ('⠆', Color::Yellow),       // 2 dots - partial
                41..=60 => ('⠇', Color::LightGreen),   // 3 dots - half done
                61..=80 => ('⣤', Color::LightGreen),   // 4 dots - mostly done
                81..=99 => ('⣶', Color::LightGreen),   // 6 dots - almost done
                _ => ('⣿', Color::Green),              // Full - complete
            };

            let x = map_start + col as u16;
            let y = area.y + row as u16;
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(ch);
                cell.set_fg(fg);
            }
        }
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

fn render_shards_map(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let label_width: u16 = 12;
    let right_margin: u16 = 6;

    let label = "Shards";
    buf.set_string(area.x + 2, area.y, label, Style::default().fg(Color::Gray));

    // Use phase-appropriate counters and colors
    let (total, done, done_color) = match data.phase {
        Phase::Seal => (data.total_to_seal, data.sealed_shards, Color::Green),
        _ => (data.total_shards, data.compacted_shards, Color::Magenta),
    };

    let start_label = "Shard 0".to_string();
    let end_label = format!("Shard {}", total.saturating_sub(1));

    let map_start = area.x + 2 + label_width;
    let map_end = area.x + area.width - right_margin - 1;
    let map_width = map_end.saturating_sub(map_start) as usize;

    if map_width == 0 || total == 0 {
        return;
    }

    // Scale shards to fit width
    let scale = total as f64 / map_width as f64;

    // Row 1 & 2
    for row in 0..2 {
        for i in 0..map_width {
            let shard_idx = (i as f64 * scale) as u64;
            let (ch, color) = if shard_idx < done {
                ('\u{2588}', done_color)
            } else if shard_idx == done {
                ('\u{2593}', Color::Yellow)
            } else {
                ('\u{2591}', Color::DarkGray)
            };
            buf.set_string(
                map_start + i as u16,
                area.y + row,
                ch.to_string(),
                Style::default().fg(color),
            );
        }
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

#[expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "chart rendering with gradient colors is verbose but straightforward"
)]
fn render_speed_chart(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" Speed (blocks/s) ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let y_axis_width: u16 = 7;
    let chart_height = inner.height.saturating_sub(2);
    let chart_width = inner.width.saturating_sub(y_axis_width + 1);

    if chart_height == 0 || chart_width == 0 {
        return;
    }

    // Find max for scaling - scale to actual peak with nice rounding
    let max_val = data.speed_history.iter().copied().max().unwrap_or(1).max(1);
    let scale_max = if max_val <= 50 {
        ((max_val / 10) + 1) * 10
    } else if max_val <= 200 {
        ((max_val / 25) + 1) * 25
    } else if max_val <= 500 {
        ((max_val / 50) + 1) * 50
    } else if max_val <= 2000 {
        ((max_val / 100) + 1) * 100
    } else {
        ((max_val / 500) + 1) * 500
    };

    // Draw Y-axis labels
    for row in 0..chart_height {
        if row == 0 {
            let label = format!("{:>6}", format_number(scale_max));
            buf.set_string(inner.x, inner.y + row, &label, Style::default().fg(Color::DarkGray));
        } else if row == chart_height / 2 {
            let label = format!("{:>6}", format_number(scale_max / 2));
            buf.set_string(inner.x, inner.y + row, &label, Style::default().fg(Color::DarkGray));
        } else if row == chart_height - 1 {
            buf.set_string(inner.x, inner.y + row, "     0", Style::default().fg(Color::DarkGray));
        }
    }

    // Braille dots for smooth line graph (each cell is 2x4 dots)
    // We'll use the bottom row of dots for the line
    let chart_start_x = inner.x + y_axis_width;
    let history_len = data.speed_history.len();

    if history_len == 0 {
        return;
    }

    // Calculate values for each column (with sub-cell precision using braille)
    // Braille cell has 4 vertical dot positions per character
    let total_dots_height = chart_height as f64 * 4.0;

    // Gradient colors based on relative height (red at bottom, green at top)
    let color_for_height = |ratio: f64| -> Color {
        if ratio < 0.25 {
            Color::Rgb(255, 80, 80)   // Red (low)
        } else if ratio < 0.5 {
            Color::Rgb(255, 180, 50)  // Orange
        } else if ratio < 0.75 {
            Color::Rgb(200, 220, 50)  // Yellow-green
        } else {
            Color::Rgb(80, 220, 120)  // Green (high)
        }
    };

    // First pass: draw filled area with gradient
    let samples_per_col = history_len as f64 / chart_width as f64;

    for col in 0..chart_width {
        let sample_idx = (col as f64 * samples_per_col) as usize;
        let value = data.speed_history.get(sample_idx).copied().unwrap_or(0);
        let ratio = value as f64 / scale_max as f64;
        let dot_height = (ratio * total_dots_height).round() as u16;

        // Draw each row of this column
        for row in 0..chart_height {
            let y = inner.y + chart_height - 1 - row;
            let row_dot_start = row * 4;
            let row_dot_end = row_dot_start + 4;

            // How many dots in this cell should be filled?
            let dots_in_cell = if dot_height <= row_dot_start {
                0
            } else if dot_height >= row_dot_end {
                4
            } else {
                (dot_height - row_dot_start) as usize
            };

            // Braille patterns for 0-4 dots filled from bottom
            // Using left column only for thin line look: ⠀⢀⢠⢰⢸
            let ch = match dots_in_cell {
                0 => '⠀',  // Empty
                1 => '⢀',  // Dot 7
                2 => '⢠',  // Dots 7,4
                3 => '⢰',  // Dots 7,4,2
                _ => '⢸',  // Dots 7,4,2,1 (full left column)
            };

            let cell_ratio = (row as f64 + 0.5) / chart_height as f64;
            let color = if dots_in_cell > 0 {
                color_for_height(cell_ratio.min(ratio))
            } else {
                Color::Rgb(30, 30, 40)  // Dark background
            };

            if let Some(cell) = buf.cell_mut((chart_start_x + col, y)) {
                cell.set_char(ch);
                cell.set_fg(color);
            }
        }
    }

    // Draw sparkline on top (current value marker)
    if !data.speed_history.is_empty() {
        let last_val = *data.speed_history.back().unwrap_or(&0);
        let last_ratio = last_val as f64 / scale_max as f64;
        let last_row = ((1.0 - last_ratio) * (chart_height as f64 - 1.0)).round() as u16;
        let marker_y = inner.y + last_row.min(chart_height - 1);
        let marker_x = chart_start_x + chart_width - 1;

        if let Some(cell) = buf.cell_mut((marker_x, marker_y)) {
            cell.set_char('●');
            cell.set_fg(Color::White);
        }
    }

    // Time axis labels
    let axis_y = inner.y + chart_height;
    buf.set_string(chart_start_x, axis_y, "-1m", Style::default().fg(Color::DarkGray));
    buf.set_string(
        chart_start_x + chart_width - 3,
        axis_y,
        "now",
        Style::default().fg(Color::DarkGray),
    );

    // Speed stats line
    let stats_y = inner.y + inner.height - 1;
    let mut x = inner.x + 1;

    buf.set_string(x, stats_y, "● Cur: ", Style::default().fg(Color::Yellow));
    x += 7;
    buf.set_string(x, stats_y, &format!("{}/s", format_number(data.current_speed)), Style::default().fg(Color::Yellow));
    x += 10;
    buf.set_string(x, stats_y, "◆ Avg: ", Style::default().fg(Color::White));
    x += 7;
    buf.set_string(x, stats_y, &format!("{}/s", format_number(data.avg_speed)), Style::default().fg(Color::White));
    x += 10;
    buf.set_string(x, stats_y, "★ Peak: ", Style::default().fg(Color::LightCyan));
    x += 8;
    buf.set_string(x, stats_y, &format!("{}/s", format_number(data.peak_speed)), Style::default().fg(Color::LightCyan));

    let eta_text = format!("ETA: {}", data.eta_string());
    buf.set_string(
        inner.x + inner.width - eta_text.len() as u16 - 1,
        stats_y,
        &eta_text,
        Style::default().fg(Color::Magenta),
    );
}

fn render_network_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" NETWORK ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 12;

    // Peers visual: 1 peer = 1 character
    // ● = active (green, fetching), ○ = healthy idle (green), ⊘ = stale-head banned (dark gray)
    // If peers exceed available width, show … at the end
    let available_width = inner.width.saturating_sub(12) as u64;
    let active = data.peers_connected;
    let stale = data.peers_stale;
    let connected_total = active.saturating_add(data.peers_max).saturating_add(stale);
    let healthy_idle = data.peers_max.saturating_sub(active);

    let peers_visual = if connected_total == 0 {
        "—".to_string() // No peers
    } else if connected_total <= available_width {
        let active_chars: String = (0..active).map(|_| '\u{25CF}').collect(); // ●
        let idle_chars: String = (0..healthy_idle).map(|_| '\u{25CB}').collect(); // ○
        let stale_chars: String = (0..stale).map(|_| '\u{2298}').collect(); // ⊘
        format!("{active_chars}{idle_chars}{stale_chars}")
    } else {
        let display_slots = available_width.saturating_sub(1);
        let active_to_show = active.min(display_slots);
        let remaining = display_slots.saturating_sub(active_to_show);
        let idle_to_show = healthy_idle.min(remaining);
        let remaining = remaining.saturating_sub(idle_to_show);
        let stale_to_show = stale.min(remaining);

        let active_chars: String = (0..active_to_show).map(|_| '\u{25CF}').collect();
        let idle_chars: String = (0..idle_to_show).map(|_| '\u{25CB}').collect();
        let stale_chars: String = (0..stale_to_show).map(|_| '\u{2298}').collect();
        format!("{active_chars}{idle_chars}{stale_chars}\u{2026}")
    };

    buf.set_string(inner.x + 1, inner.y, "Peers", Style::default().fg(Color::Gray));
    // Render active+idle chars in green, then stale chars in dark gray
    let green_len = active.saturating_add(healthy_idle).min(available_width) as u16;
    let green_part: String = peers_visual.chars().take(green_len as usize).collect();
    let rest_part: String = peers_visual.chars().skip(green_len as usize).collect();
    buf.set_string(value_col, inner.y, &green_part, Style::default().fg(Color::Green));
    if !rest_part.is_empty() {
        buf.set_string(
            value_col + green_len,
            inner.y,
            &rest_part,
            Style::default().fg(Color::DarkGray),
        );
    }

    // Active peers (currently fetching blocks)
    buf.set_string(inner.x + 1, inner.y + 1, "Active", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 1,
        &data.peers_connected.to_string(),
        Style::default().fg(Color::Yellow),
    );

    // Healthy peers (connected & capable, not active)
    buf.set_string(inner.x + 1, inner.y + 2, "Healthy", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 2,
        &healthy_idle.to_string(),
        Style::default().fg(Color::Green),
    );

    // Stale peers (banned for stale head)
    buf.set_string(inner.x + 1, inner.y + 3, "Stale", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 3,
        &data.peers_stale.to_string(),
        Style::default().fg(Color::Rgb(139, 0, 0)),
    );

    // Connected peers (total in pool)
    buf.set_string(inner.x + 1, inner.y + 4, "Connected", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 4,
        &connected_total.to_string(),
        Style::default().fg(Color::White),
    );

    // Chain Head - use best_head_seen during startup, chain_head during sync
    let head_value = if data.phase == Phase::Startup && data.chain_head == 0 {
        data.best_head_seen
    } else {
        data.chain_head
    };
    buf.set_string(inner.x + 1, inner.y + 5, "Chain Head", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y + 5,
        &format_number(head_value),
        Style::default().fg(Color::White),
    );
}

fn render_config_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" CONFIG ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 12;

    // Start block
    buf.set_string(inner.x + 1, inner.y, "Start", Style::default().fg(Color::Gray));
    buf.set_string(
        value_col,
        inner.y,
        &format_number(data.start_block),
        Style::default().fg(Color::White),
    );

    // End block
    buf.set_string(inner.x + 1, inner.y + 1, "End", Style::default().fg(Color::Gray));
    let end_str = if data.end_block > 0 {
        format_number(data.end_block)
    } else {
        "head".to_string()
    };
    buf.set_string(value_col, inner.y + 1, &end_str, Style::default().fg(Color::White));

    // Shard size
    buf.set_string(inner.x + 1, inner.y + 2, "Shard Size", Style::default().fg(Color::Gray));
    let shard_str = if data.config_shard_size > 0 {
        format_number(data.config_shard_size)
    } else {
        "--".to_string()
    };
    buf.set_string(value_col, inner.y + 2, &shard_str, Style::default().fg(Color::White));

    // Rollback window
    buf.set_string(inner.x + 1, inner.y + 3, "Rollback", Style::default().fg(Color::Gray));
    let rollback_str = if data.config_rollback_window > 0 {
        format_number(data.config_rollback_window)
    } else {
        "--".to_string()
    };
    buf.set_string(value_col, inner.y + 3, &rollback_str, Style::default().fg(Color::White));

    // RPC bind address
    buf.set_string(inner.x + 1, inner.y + 4, "RPC", Style::default().fg(Color::Gray));
    let rpc_str = if data.config_rpc_bind.is_empty() {
        "--".to_string()
    } else {
        data.config_rpc_bind.clone()
    };
    buf.set_string(value_col, inner.y + 4, &rpc_str, Style::default().fg(Color::White));

    // Data directory (truncated if needed)
    buf.set_string(inner.x + 1, inner.y + 5, "Data Dir", Style::default().fg(Color::Gray));
    let dir_str = if data.config_data_dir.is_empty() {
        "--".to_string()
    } else {
        // Truncate to fit in panel, show last part of path
        let max_len = (inner.width as usize).saturating_sub(13);
        if data.config_data_dir.len() > max_len {
            format!("...{}", &data.config_data_dir[data.config_data_dir.len() - max_len + 3..])
        } else {
            data.config_data_dir.clone()
        }
    };
    buf.set_string(value_col, inner.y + 5, &dir_str, Style::default().fg(Color::DarkGray));
}

fn render_queue_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" QUEUE ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    // Show remaining blocks (total - synced) instead of internal queue
    let remaining = data.total_blocks().saturating_sub(data.synced_blocks());
    let items = [
        ("Remaining", remaining, Color::White),
        ("Inflight", data.inflight, Color::Yellow),
        ("Retry", data.retry, Color::Rgb(255, 165, 0)),
    ];

    for (i, (label, value, color)) in items.iter().enumerate() {
        buf.set_string(inner.x + 1, inner.y + i as u16, *label, Style::default().fg(Color::Gray));
        buf.set_string(
            inner.x + 11,
            inner.y + i as u16,
            &format!("{:>6}", value),
            Style::default().fg(*color),
        );
    }
}

fn render_storage_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" STORAGE ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 14;

    let items = [("Headers", "--"), ("Transactions", "--"), ("Receipts", "--")];

    for (i, (label, value)) in items.iter().enumerate() {
        buf.set_string(inner.x + 1, inner.y + i as u16, *label, Style::default().fg(Color::Gray));
        buf.set_string(value_col, inner.y + i as u16, *value, Style::default().fg(Color::DarkGray));
    }

    buf.set_string(
        inner.x + 1,
        inner.y + 3,
        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        Style::default().fg(Color::DarkGray),
    );

    buf.set_string(inner.x + 1, inner.y + 4, "Total", Style::default().fg(Color::Gray));
    let total_str = data
        .storage_total
        .map_or("--".into(), |v| format!("{v:.1} GiB"));
    buf.set_string(value_col, inner.y + 4, &total_str, Style::default().fg(Color::White).bold());

    buf.set_string(inner.x + 1, inner.y + 5, "Write Rate", Style::default().fg(Color::Gray));
    let rate_str = data
        .write_rate
        .map_or("--".into(), |v| format!("{v:.1} MB/s"));
    buf.set_string(value_col, inner.y + 5, &rate_str, Style::default().fg(Color::Green));
}

fn render_db_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" DB ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 10;

    buf.set_string(inner.x + 1, inner.y, "Blocks", Style::default().fg(Color::Gray));
    let blocks_str = data.db_blocks.map_or("--".into(), format_number);
    buf.set_string(value_col, inner.y, &blocks_str, Style::default().fg(Color::White));

    buf.set_string(inner.x + 1, inner.y + 1, "Txns", Style::default().fg(Color::Gray));
    let txns_str = data.db_transactions.map_or("--".into(), format_number);
    buf.set_string(value_col, inner.y + 1, &txns_str, Style::default().fg(Color::White));

    buf.set_string(inner.x + 1, inner.y + 2, "Receipts", Style::default().fg(Color::Gray));
    let receipts_str = data.db_receipts.map_or("--".into(), format_number);
    buf.set_string(value_col, inner.y + 2, &receipts_str, Style::default().fg(Color::White));
}

fn render_rpc_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let border_color = if data.rpc_active { Color::Green } else { Color::DarkGray };
    let status_color = if data.rpc_active { Color::LightGreen } else { Color::DarkGray };

    let block = Block::default()
        .title(" RPC ")
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    block.render(area, buf);

    let value_col = inner.x + 10;

    // Status indicator
    let status = if data.rpc_active { "● Active" } else { "○ Inactive" };
    buf.set_string(inner.x + 1, inner.y, status, Style::default().fg(status_color));

    // Request rate
    buf.set_string(inner.x + 1, inner.y + 1, "Req/s", Style::default().fg(Color::Gray));
    let rate_str = if data.rpc_active {
        format!("{:.1}", data.rpc_requests_per_sec)
    } else {
        "--".into()
    };
    buf.set_string(value_col, inner.y + 1, &rate_str, Style::default().fg(Color::White));

    // Total requests
    buf.set_string(inner.x + 1, inner.y + 2, "Total", Style::default().fg(Color::Gray));
    let total_str = if data.rpc_active {
        format_number(data.rpc_total_requests)
    } else {
        "--".into()
    };
    buf.set_string(value_col, inner.y + 2, &total_str, Style::default().fg(Color::White));

    // getLogs calls
    buf.set_string(inner.x + 1, inner.y + 3, "getLogs", Style::default().fg(Color::Gray));
    let logs_str = if data.rpc_active {
        format_number(data.rpc_get_logs)
    } else {
        "--".into()
    };
    buf.set_string(value_col, inner.y + 3, &logs_str, Style::default().fg(Color::White));

    // getBlock calls
    buf.set_string(inner.x + 1, inner.y + 4, "getBlock", Style::default().fg(Color::Gray));
    let block_str = if data.rpc_active {
        format_number(data.rpc_get_block)
    } else {
        "--".into()
    };
    buf.set_string(value_col, inner.y + 4, &block_str, Style::default().fg(Color::White));

    // Errors
    buf.set_string(inner.x + 1, inner.y + 5, "Errors", Style::default().fg(Color::Gray));
    let err_color = if data.rpc_errors > 0 { Color::Red } else { Color::White };
    let err_str = if data.rpc_active {
        format_number(data.rpc_errors)
    } else {
        "--".into()
    };
    buf.set_string(value_col, inner.y + 5, &err_str, Style::default().fg(err_color));
}

fn render_compaction_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    // Use phase-appropriate title, counters, and colors
    let (title, done, total, done_color) = match data.phase {
        Phase::Seal => (" SEALING ", data.sealed_shards, data.total_to_seal, Color::Green),
        _ => (" COMPACTION ", data.compacted_shards, data.total_shards, Color::Magenta),
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::White).bold())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    buf.set_string(inner.x + 1, inner.y, "Done", Style::default().fg(Color::Gray));
    buf.set_string(
        inner.x + 11,
        inner.y,
        &format!("{:>6}", done),
        Style::default().fg(done_color),
    );

    buf.set_string(inner.x + 1, inner.y + 1, "Current", Style::default().fg(Color::Gray));
    if done < total {
        buf.set_string(
            inner.x + 11,
            inner.y + 1,
            &format!("{:>6}", done),
            Style::default().fg(Color::Yellow),
        );
    } else {
        buf.set_string(inner.x + 11, inner.y + 1, "     -", Style::default().fg(Color::DarkGray));
    }

    let pending = total.saturating_sub(done).saturating_sub(1);
    buf.set_string(inner.x + 1, inner.y + 2, "Pending", Style::default().fg(Color::Gray));
    buf.set_string(
        inner.x + 11,
        inner.y + 2,
        &format!("{:>6}", pending),
        Style::default().fg(Color::White),
    );
}

fn render_logs_panel(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let block = Block::default()
        .title(" Logs ")
        .title_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    block.render(area, buf);

    if data.logs.is_empty() {
        buf.set_string(
            inner.x + 1,
            inner.y,
            "(no logs captured)",
            Style::default().fg(Color::DarkGray),
        );
        return;
    }

    // Show most recent logs first (reverse order), limited to visible height
    for (i, log) in data.logs.iter().rev().take(inner.height as usize).enumerate() {
        // Padded level names (ERROR is 5 chars, so pad others)
        let (level_str, level_style) = match log.level {
            LogLevel::Error => ("ERROR", Style::default().fg(Color::Red)),
            LogLevel::Warn => ("WARN ", Style::default().fg(Color::Yellow)),
            LogLevel::Info => ("INFO ", Style::default().fg(Color::Cyan)),
            LogLevel::Debug => ("DEBUG", Style::default().fg(Color::DarkGray)),
        };

        // Format timestamp as "2026-Jan-27 10:46:36.123"
        let timestamp = format_unix_timestamp_ms(log.timestamp_ms);

        // Calculate available width for message
        // Format: "2026-Jan-27 10:46:36.123 INFO  message..."
        let prefix_len = timestamp.len() + 1 + 5 + 1; // timestamp + space + level + space
        let max_msg_len = (inner.width as usize).saturating_sub(prefix_len + 1);
        let truncated_msg: String = log.message.chars().take(max_msg_len).collect();

        // Render timestamp in gray
        let x = inner.x + 1;
        let y = inner.y + i as u16;
        buf.set_string(x, y, &timestamp, Style::default().fg(Color::DarkGray));

        // Render level with its color
        let level_x = x + timestamp.len() as u16 + 1;
        buf.set_string(level_x, y, level_str, level_style);

        // Render message: base in Gray, structured fields after │ in DarkGray
        let msg_x = level_x + 6; // 5 chars for level + 1 space
        if let Some(sep_pos) = truncated_msg.find('\u{2502}') {
            let base = &truncated_msg[..sep_pos];
            buf.set_string(msg_x, y, base, Style::default().fg(Color::Gray));
            let fields_part = &truncated_msg[sep_pos..];
            let fields_x = msg_x + base.len() as u16;
            buf.set_string(fields_x, y, fields_part, Style::default().fg(Color::DarkGray));
        } else {
            buf.set_string(msg_x, y, &truncated_msg, Style::default().fg(Color::Gray));
        }
    }
}

/// Format a Unix timestamp in milliseconds to "2026-Jan-27 10:46:36.123" format.
fn format_unix_timestamp_ms(timestamp_ms: u64) -> String {
    // Convert to time components manually (UTC)
    // For simplicity, use chrono-free approach with rough calculation
    let secs = timestamp_ms / 1000;
    let millis = timestamp_ms % 1000;

    // Get UTC time components (simplified - doesn't account for leap seconds)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year/month/day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days_since_epoch);

    let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                       "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let month_str = month_names.get(month as usize).unwrap_or(&"???");

    format!("{}-{}-{:02} {:02}:{:02}:{:02}.{:03}",
            year, month_str, day, hours, minutes, seconds, millis)
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Simplified algorithm - good enough for display purposes
    let mut remaining_days = days as i64;
    let mut year = 1970i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0i64;
    for (i, &days_in_month) in days_in_months.iter().enumerate() {
        if remaining_days < days_in_month {
            month = i as i64;
            break;
        }
        remaining_days -= days_in_month;
    }

    let day = remaining_days + 1; // Days are 1-indexed
    (year as u64, month as u64, day as u64)
}

const fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn render_synced_status(area: Rect, buf: &mut Buffer, data: &TuiState) {
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(area);

    let left = halves[0];
    let right = halves[1];

    let number_color = if data.is_synced() {
        Color::LightGreen
    } else {
        Color::Yellow
    };

    // Network name with sparkle animation
    let sparkle_chars = ['·', '✧', '✦', '★', '✦', '✧'];
    let sparkle_colors = [
        Color::Rgb(100, 100, 120),  // dim
        Color::Rgb(150, 150, 180),  // medium
        Color::Rgb(200, 200, 255),  // bright
        Color::Rgb(255, 255, 255),  // white
        Color::Rgb(200, 200, 255),  // bright
        Color::Rgb(150, 150, 180),  // medium
    ];

    let frame = (data.animation_frame / 2) as usize; // Slow down animation (2x)
    let x = left.x + 2;
    let y = left.y + 1;

    // Left sparkles (offset phases)
    let s1 = sparkle_chars[frame % 6];
    let c1 = sparkle_colors[frame % 6];
    let s2 = sparkle_chars[(frame + 2) % 6];
    let c2 = sparkle_colors[(frame + 2) % 6];
    let s3 = sparkle_chars[(frame + 4) % 6];
    let c3 = sparkle_colors[(frame + 4) % 6];

    buf.set_string(x, y, &format!("{}", s1), Style::default().fg(c1));
    buf.set_string(x + 1, y, " ", Style::default());
    buf.set_string(x + 2, y, &format!("{}", s2), Style::default().fg(c2));
    buf.set_string(x + 3, y, " ", Style::default());

    // Main title
    buf.set_string(x + 4, y, "Ethereum Mainnet", Style::default().fg(Color::White).bold());

    // Right sparkles (different offset phases)
    let rx = x + 4 + 16; // after "Ethereum Mainnet"
    buf.set_string(rx, y, " ", Style::default());
    buf.set_string(rx + 1, y, &format!("{}", s3), Style::default().fg(c3));
    buf.set_string(rx + 2, y, " ", Style::default());
    buf.set_string(rx + 3, y, &format!("{}", s2), Style::default().fg(c2));

    if data.is_synced() {
        let synced_text = "\u{2713} SYNCED";
        buf.set_string(left.x + 4, left.y + 3, synced_text, Style::default().fg(Color::LightGreen).bold());

        buf.set_string(left.x + 4, left.y + 5, "Last block", Style::default().fg(Color::Gray));
        buf.set_string(
            left.x + 15,
            left.y + 5,
            &format!("{}s ago", data.last_block_secs),
            Style::default().fg(Color::White),
        );
    } else {
        let catching_text = "CATCHING UP";
        buf.set_string(left.x + 4, left.y + 3, catching_text, Style::default().fg(Color::Yellow).bold());

        buf.set_string(
            left.x + 4,
            left.y + 5,
            &format!("{} blocks behind", data.blocks_behind()),
            Style::default().fg(Color::Yellow),
        );
    }

    render_big_number(right, buf, data.our_head, number_color);
}

/// ASCII art digits (3 chars wide, 5 rows tall)
const DIGITS: [[&str; 5]; 10] = [
    ["\u{2588}\u{2580}\u{2588}", "\u{2588} \u{2588}", "\u{2588} \u{2588}", "\u{2588} \u{2588}", "\u{2580}\u{2580}\u{2580}"], // 0
    [" \u{2580}\u{2588}", "  \u{2588}", "  \u{2588}", "  \u{2588}", "  \u{2580}"], // 1
    ["\u{2580}\u{2580}\u{2588}", "  \u{2588}", "\u{2588}\u{2580}\u{2580}", "\u{2588}  ", "\u{2580}\u{2580}\u{2580}"], // 2
    ["\u{2580}\u{2580}\u{2588}", "  \u{2588}", "\u{2580}\u{2580}\u{2588}", "  \u{2588}", "\u{2580}\u{2580}\u{2580}"], // 3
    ["\u{2588} \u{2588}", "\u{2588} \u{2588}", "\u{2580}\u{2580}\u{2588}", "  \u{2588}", "  \u{2580}"], // 4
    ["\u{2588}\u{2580}\u{2580}", "\u{2588}  ", "\u{2580}\u{2580}\u{2588}", "  \u{2588}", "\u{2580}\u{2580}\u{2580}"], // 5
    ["\u{2588}\u{2580}\u{2580}", "\u{2588}  ", "\u{2588}\u{2580}\u{2588}", "\u{2588} \u{2588}", "\u{2580}\u{2580}\u{2580}"], // 6
    ["\u{2580}\u{2580}\u{2588}", "  \u{2588}", "  \u{2588}", "  \u{2588}", "  \u{2580}"], // 7
    ["\u{2588}\u{2580}\u{2588}", "\u{2588} \u{2588}", "\u{2588}\u{2580}\u{2588}", "\u{2588} \u{2588}", "\u{2580}\u{2580}\u{2580}"], // 8
    ["\u{2588}\u{2580}\u{2588}", "\u{2588} \u{2588}", "\u{2580}\u{2580}\u{2588}", "  \u{2588}", "\u{2580}\u{2580}\u{2580}"], // 9
];

const COMMA: [&str; 5] = ["   ", "   ", "   ", " \u{2584} ", " \u{2580} "];

fn render_big_number(area: Rect, buf: &mut Buffer, number: u64, color: Color) {
    let formatted = format_number(number);
    let chars: Vec<char> = formatted.chars().collect();

    let total_width: u16 = chars.len() as u16 * 4;
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
// Helper functions
// ============================================================================

fn chrono_time() -> String {
    let now = SystemTime::now();
    let secs = now.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
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
