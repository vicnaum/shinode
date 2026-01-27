//! Main progress tracking and UIController.

use crate::p2p::PeerPool;
use crate::sync::historical::PeerHealthTracker;
use crate::sync::{format_eta_seconds, SyncProgressSnapshot, SyncProgressStats, SyncStatus};
use indicatif::{MultiProgress, ProgressBar};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::bars::{
    create_compacting_bar, create_failed_bar, create_follow_bar, create_sealing_bar,
    create_sync_bar, format_follow_segment,
};
use super::state::UIState;

/// Main UI controller that manages all progress bars and state transitions.
pub struct UIController {
    multi: MultiProgress,
    current_bar: Option<ProgressBar>,
    recovery_bar: Option<ProgressBar>,
    state: UIState,
    is_tty: bool,
    total_blocks: u64,
    speed_window: VecDeque<(Instant, u64)>,
    /// Total failed blocks when recovery started (for progress tracking).
    recovery_total: u64,
}

impl UIController {
    /// Create a new UIController.
    pub fn new(is_tty: bool) -> Self {
        Self {
            multi: MultiProgress::new(),
            current_bar: None,
            recovery_bar: None,
            state: UIState::Startup,
            is_tty,
            total_blocks: 0,
            speed_window: VecDeque::new(),
            recovery_total: 0,
        }
    }

    /// Get the current bar (for external progress updates).
    pub fn current_bar(&self) -> Option<&ProgressBar> {
        self.current_bar.as_ref()
    }

    /// Transition to syncing state.
    pub fn show_syncing(&mut self, total_blocks: u64) {
        if !self.is_tty {
            self.state = UIState::Syncing;
            self.total_blocks = total_blocks;
            return;
        }
        // Clear any existing bar
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }
        // Clear startup line
        eprint!("\r\x1b[K");
        let _ = std::io::Write::flush(&mut std::io::stderr());

        self.total_blocks = total_blocks;
        self.speed_window.clear();
        let bar = create_sync_bar(&self.multi, total_blocks);
        self.current_bar = Some(bar);
        self.state = UIState::Syncing;
    }

    /// Transition to compacting state (magenta bar).
    pub fn show_compacting(&mut self, total_shards: u64) {
        if !self.is_tty {
            self.state = UIState::Compacting;
            return;
        }
        // Clear any existing bar
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }
        let bar = create_compacting_bar(&self.multi, total_shards);
        let left = total_shards;
        bar.set_message(format!("Compacting: {left} shards left"));
        self.current_bar = Some(bar);
        self.state = UIState::Compacting;
    }

    /// Transition to sealing state (bright green bar).
    pub fn show_sealing(&mut self, total_shards: u64) {
        if !self.is_tty {
            self.state = UIState::Sealing;
            return;
        }
        // Clear any existing bar
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }
        let bar = create_sealing_bar(&self.multi, total_shards);
        let left = total_shards;
        bar.set_message(format!("Sealing: {left} shards left"));
        self.current_bar = Some(bar);
        self.state = UIState::Sealing;
    }

    /// Transition to following state.
    pub fn show_following(&mut self) {
        if !self.is_tty {
            self.state = UIState::Following;
            return;
        }
        // Clear any existing bar
        if let Some(bar) = self.current_bar.take() {
            bar.finish_and_clear();
        }
        let bar = create_follow_bar(&self.multi);
        self.current_bar = Some(bar);
        self.state = UIState::Following;
    }

    /// Update the UI based on current snapshot.
    /// This handles state transitions and bar updates.
    pub fn update(&mut self, snapshot: &SyncProgressSnapshot, peers_connected: u64) {
        let now = Instant::now();

        // Determine expected state from snapshot
        let expected_state = UIState::from_sync_snapshot(
            snapshot.status,
            snapshot.finalize_phase,
            snapshot.fetch_complete,
            snapshot.processed,
            self.total_blocks,
        );

        // Handle state transitions
        if self.state != expected_state {
            match expected_state {
                UIState::Startup => {
                    // Shouldn't normally transition back to startup
                }
                UIState::Syncing => {
                    self.show_syncing(self.total_blocks);
                }
                UIState::Compacting => {
                    self.show_compacting(snapshot.compactions_total);
                }
                UIState::Sealing => {
                    self.show_sealing(snapshot.compactions_total);
                }
                UIState::Following => {
                    self.show_following();
                }
            }
        }

        // Update recovery bar (instant show/hide, no delay)
        self.update_recovery_bar(snapshot);

        // Update the appropriate bar based on current state
        match self.state {
            UIState::Startup => {
                // Startup doesn't use progress bars
            }
            UIState::Syncing => {
                self.update_sync_bar(snapshot, peers_connected, now);
            }
            UIState::Compacting => {
                self.update_compacting_bar(snapshot);
            }
            UIState::Sealing => {
                self.update_sealing_bar(snapshot);
            }
            UIState::Following => {
                self.update_follow_bar(snapshot, peers_connected);
            }
        }
    }

    /// Update the sync progress bar.
    fn update_sync_bar(&mut self, snapshot: &SyncProgressSnapshot, _peers_connected: u64, now: Instant) {
        let Some(bar) = self.current_bar.as_ref() else {
            return;
        };

        let processed = snapshot.processed.min(self.total_blocks);

        // Update speed window
        self.speed_window.push_back((now, processed));
        while let Some((t, _)) = self.speed_window.front() {
            if now.duration_since(*t) > Duration::from_secs(1) && self.speed_window.len() > 1 {
                self.speed_window.pop_front();
            } else {
                break;
            }
        }

        // Calculate speed
        let speed = if let (Some((t0, v0)), Some((t1, v1))) =
            (self.speed_window.front(), self.speed_window.back())
        {
            let dt = t1.duration_since(*t0).as_secs_f64();
            if dt > 0.0 && v1 >= v0 {
                (v1 - v0) as f64 / dt
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate ETA
        let remaining = self.total_blocks.saturating_sub(processed) as f64;
        let eta = if speed > 0.0 {
            format_eta_seconds(remaining / speed)
        } else {
            "--".to_string()
        };

        let peers_active = snapshot.peers_active.min(snapshot.peers_total);
        let msg = format_progress_message(
            snapshot.status,
            peers_active,
            snapshot.peers_total,
            snapshot.queue,
            snapshot.inflight,
            snapshot.escalation,
            snapshot.compactions_done,
            snapshot.compactions_total,
            speed,
            &eta,
        );
        bar.set_message(msg);
        bar.set_position(processed);
    }

    /// Update the compacting progress bar.
    fn update_compacting_bar(&mut self, snapshot: &SyncProgressSnapshot) {
        let Some(bar) = self.current_bar.as_ref() else {
            return;
        };

        let done = snapshot.compactions_done;
        let total = snapshot.compactions_total;
        let left = total.saturating_sub(done);

        bar.set_length(total.max(1));
        bar.set_position(done);
        bar.set_message(format!("Compacting: {left} shards left"));
    }

    /// Update the sealing progress bar.
    fn update_sealing_bar(&mut self, snapshot: &SyncProgressSnapshot) {
        let Some(bar) = self.current_bar.as_ref() else {
            return;
        };

        let done = snapshot.compactions_done;
        let total = snapshot.compactions_total;
        let left = total.saturating_sub(done);

        bar.set_length(total.max(1));
        bar.set_position(done);
        bar.set_message(format!("Sealing: {left} shards left"));
    }

    /// Update the follow mode bar.
    fn update_follow_bar(&mut self, snapshot: &SyncProgressSnapshot, peers_connected: u64) {
        let Some(bar) = self.current_bar.as_ref() else {
            return;
        };

        let head_block = snapshot.head_block;
        let head_seen = snapshot.head_seen;
        let peers_available = snapshot.peers_total.min(peers_connected);
        let active_fetch = snapshot.peers_active;

        let (status_str, status_detail) = match snapshot.status {
            SyncStatus::Following | SyncStatus::UpToDate => ("Synced", String::new()),
            SyncStatus::Fetching => (
                "Catching up",
                format!(" ({} blocks left)", head_seen.saturating_sub(head_block)),
            ),
            SyncStatus::Finalizing => ("Finalizing", " (flushing/compacting...)".to_string()),
            SyncStatus::LookingForPeers => ("Waiting for peers", String::new()),
        };

        let bar_segment = format_follow_segment(head_block);

        let msg = format!(
            "{} {}{} | head {} | peers {}/{} | fetch {} | retry {}",
            bar_segment,
            status_str,
            status_detail,
            head_seen,
            peers_available,
            peers_connected,
            active_fetch,
            snapshot.escalation,
        );
        bar.set_message(msg);
    }

    /// Update the recovery bar (show/hide instantly based on current state).
    fn update_recovery_bar(&mut self, snapshot: &SyncProgressSnapshot) {
        if !self.is_tty {
            return;
        }

        // Recovery-only mode: queue empty AND has escalation AND not finished
        let only_recovery_left =
            snapshot.queue == 0 && snapshot.escalation > 0 && !snapshot.fetch_complete;

        if only_recovery_left {
            // Show or update recovery bar
            if self.recovery_bar.is_none() {
                // Capture total when starting recovery
                self.recovery_total = snapshot.escalation;
                let bar = create_failed_bar(&self.multi, self.recovery_total);
                bar.set_message("Recovering failed blocks");
                self.recovery_bar = Some(bar);
            }

            if let Some(bar) = self.recovery_bar.as_ref() {
                let remaining = snapshot.escalation;
                let recovered = self.recovery_total.saturating_sub(remaining);
                bar.set_length(self.recovery_total.max(1));
                bar.set_position(recovered);
                bar.set_message(format!("Recovering failed blocks: {}/{}", recovered, self.recovery_total));
            }
        } else {
            // Hide recovery bar
            if let Some(bar) = self.recovery_bar.take() {
                bar.finish_and_clear();
            }
            self.recovery_total = 0;
        }
    }

}

/// Format the progress message shown in the sync bar.
pub fn format_progress_message(
    status: SyncStatus,
    peers_active: u64,
    peers_total: u64,
    queue: u64,
    inflight: u64,
    escalation: u64,
    compactions_done: u64,
    compactions_total: u64,
    speed: f64,
    eta: &str,
) -> String {
    let compact = if status == SyncStatus::Finalizing && compactions_total > 0 {
        format!(" | compact {compactions_done}/{compactions_total}")
    } else {
        String::new()
    };
    format!(
        "status {} | peers {}/{} | queue {} | inflight {} | retry {}{} | speed {:.1}/s | eta {}",
        status.as_str(),
        peers_active,
        peers_total,
        queue,
        inflight,
        escalation,
        compact,
        speed,
        eta
    )
}

/// Spawn the background task that updates progress bars.
pub fn spawn_progress_updater(
    ui: Arc<parking_lot::Mutex<UIController>>,
    stats: Arc<SyncProgressStats>,
    peer_pool: Arc<PeerPool>,
    peer_health: Option<Arc<PeerHealthTracker>>,
) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        let mut last_peer_update = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .unwrap_or_else(Instant::now);

        loop {
            ticker.tick().await;
            let now = Instant::now();

            // Update peer counts periodically
            if now.duration_since(last_peer_update) >= Duration::from_millis(500) {
                last_peer_update = now;
                let connected = peer_pool.len() as u64;
                let available = if let Some(peer_health) = peer_health.as_ref() {
                    let peers = peer_pool.snapshot();
                    let peer_ids: Vec<_> = peers.iter().map(|peer| peer.peer_id).collect();
                    let banned = peer_health.count_banned_peers(&peer_ids).await;
                    connected.saturating_sub(banned)
                } else {
                    connected
                };
                stats.set_peers_total(available);
            }

            let snapshot = stats.snapshot();
            let peers_connected = peer_pool.len() as u64;

            // Update UI
            ui.lock().update(&snapshot, peers_connected);
        }
    });
}

