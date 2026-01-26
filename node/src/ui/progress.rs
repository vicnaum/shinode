//! Main progress tracking and updater logic.

use crate::p2p::PeerPool;
use crate::sync::historical::PeerHealthTracker;
use crate::sync::{format_eta_seconds, SyncProgressSnapshot, SyncProgressStats, SyncStatus};
use indicatif::{MultiProgress, ProgressBar};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::bars::{
    create_failed_bar, create_finalizing_bar, create_follow_bar, create_sync_bar,
    format_follow_segment,
};
use super::state::UIState;

/// Delay before showing the failed recovery bar (to avoid flicker).
const FAILED_BAR_DELAY: Duration = Duration::from_secs(2);

/// Main UI controller that manages all progress bars.
pub struct ProgressUI {
    multi: MultiProgress,
    sync_bar: Option<ProgressBar>,
    finalizing_bar: Option<ProgressBar>,
    follow_bar: Option<ProgressBar>,
    failed_bar: Option<ProgressBar>,
    total_len: u64,
    is_tty: bool,
}

impl ProgressUI {
    /// Create a new ProgressUI.
    pub fn new(total_len: u64, is_tty: bool) -> Self {
        let multi = MultiProgress::new();
        Self {
            multi,
            sync_bar: None,
            finalizing_bar: None,
            follow_bar: None,
            failed_bar: None,
            total_len,
            is_tty,
        }
    }

    /// Get a reference to the MultiProgress for adding bars.
    pub fn multi(&self) -> &MultiProgress {
        &self.multi
    }

    /// Initialize the sync bar (cyan progress bar).
    pub fn init_sync_bar(&mut self) -> Option<ProgressBar> {
        if !self.is_tty {
            return None;
        }
        let bar = create_sync_bar(&self.multi, self.total_len);
        self.sync_bar = Some(bar.clone());
        Some(bar)
    }

    /// Get the sync bar if it exists.
    pub fn sync_bar(&self) -> Option<&ProgressBar> {
        self.sync_bar.as_ref()
    }

    /// Check if this is a TTY.
    pub fn is_tty(&self) -> bool {
        self.is_tty
    }

    /// Get total length.
    pub fn total_len(&self) -> u64 {
        self.total_len
    }
}

/// Format the progress message shown in the sync bar.
pub fn format_progress_message(
    status: SyncStatus,
    peers_active: u64,
    peers_total: u64,
    queue: u64,
    inflight: u64,
    failed: u64,
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
        "status {} | peers {}/{} | queue {} | inflight {} | failed {}{} | speed {:.1}/s | eta {}",
        status.as_str(),
        peers_active,
        peers_total,
        queue,
        inflight,
        failed,
        compact,
        speed,
        eta
    )
}

/// Spawn the background task that updates progress bars.
pub fn spawn_progress_updater(
    bar: ProgressBar,
    stats: Arc<SyncProgressStats>,
    total_len: u64,
    peer_pool: Arc<PeerPool>,
    peer_health: Option<Arc<PeerHealthTracker>>,
    multi: MultiProgress,
) {
    tokio::spawn(async move {
        let mut window: VecDeque<(Instant, u64)> = VecDeque::new();
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        let mut ui_state = UIState::Syncing;
        let mut follow_bar: Option<ProgressBar> = None;
        let mut failed_bar: Option<ProgressBar> = None;
        let mut finalizing_bar: Option<ProgressBar> = None;
        let mut failed_total = 0u64;
        let mut failed_first_seen: Option<Instant> = None;
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
            let failed = snapshot.failed;

            // Track failed blocks for the red recovery bar
            if failed > 0 {
                if failed_first_seen.is_none() {
                    failed_first_seen = Some(now);
                }
                if failed > failed_total {
                    failed_total = failed;
                }
            }

            // Show failed bar after delay (to avoid flicker for quick recoveries)
            if failed_total > 0
                && failed_bar.is_none()
                && failed_first_seen
                    .map(|t| now.duration_since(t) >= FAILED_BAR_DELAY)
                    .unwrap_or(false)
            {
                let fb = create_failed_bar(&multi, failed_total);
                failed_bar = Some(fb);
            }

            // Update failed bar progress
            if let Some(ref fb) = failed_bar {
                let recovered = failed_total.saturating_sub(failed);
                fb.set_length(failed_total.max(1));
                fb.set_position(recovered);
                fb.set_message(format!(
                    "Recovering failed blocks: {}/{}",
                    recovered, failed_total
                ));
                if failed == 0 {
                    fb.finish_and_clear();
                    failed_bar = None;
                    failed_total = 0;
                    failed_first_seen = None;
                }
            }

            // Determine current UI state
            let processed = snapshot.processed.min(total_len);
            let new_ui_state = if processed >= total_len {
                match snapshot.status {
                    SyncStatus::Finalizing => UIState::Finalizing,
                    _ => UIState::Following,
                }
            } else {
                UIState::Syncing
            };

            // Handle state transitions
            if new_ui_state != ui_state {
                match (&ui_state, &new_ui_state) {
                    // Syncing -> Finalizing: finish sync bar, show finalizing bar
                    (UIState::Syncing, UIState::Finalizing) => {
                        bar.finish_and_clear();
                        if snapshot.compactions_total > 0 {
                            let fb = create_finalizing_bar(&multi, snapshot.compactions_total);
                            fb.set_message("Finalizing: compacting shards...");
                            finalizing_bar = Some(fb);
                        }
                    }
                    // Syncing -> Following: finish sync bar, show follow bar
                    (UIState::Syncing, UIState::Following) => {
                        bar.finish_and_clear();
                        let fb = create_follow_bar(&multi);
                        follow_bar = Some(fb);
                    }
                    // Finalizing -> Following: finish finalizing bar, show follow bar
                    (UIState::Finalizing, UIState::Following) => {
                        if let Some(ref fb) = finalizing_bar {
                            fb.finish_and_clear();
                        }
                        finalizing_bar = None;
                        let fb = create_follow_bar(&multi);
                        follow_bar = Some(fb);
                    }
                    _ => {}
                }
                ui_state = new_ui_state;
            }

            // Update the appropriate bar based on current state
            match ui_state {
                UIState::Syncing => {
                    // Update sync bar
                    window.push_back((now, processed));
                    while let Some((t, _)) = window.front() {
                        if now.duration_since(*t) > Duration::from_secs(1) && window.len() > 1 {
                            window.pop_front();
                        } else {
                            break;
                        }
                    }
                    let speed = if let (Some((t0, v0)), Some((t1, v1))) =
                        (window.front(), window.back())
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
                    let remaining = total_len.saturating_sub(processed) as f64;
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
                        snapshot.failed,
                        snapshot.compactions_done,
                        snapshot.compactions_total,
                        speed,
                        &eta,
                    );
                    bar.set_message(msg);
                    bar.set_position(processed);
                }
                UIState::Finalizing => {
                    // Update finalizing bar
                    if let Some(ref fb) = finalizing_bar {
                        let done = snapshot.compactions_done.min(snapshot.compactions_total);
                        fb.set_position(done);
                        fb.set_message(format!(
                            "Finalizing: compacting shards {}/{}",
                            done, snapshot.compactions_total
                        ));

                        // Check if finalizing is done
                        if done >= snapshot.compactions_total && snapshot.compactions_total > 0 {
                            fb.finish_and_clear();
                        }
                    }
                }
                UIState::Following => {
                    // Update follow bar
                    if let Some(ref fb) = follow_bar {
                        let head_block = snapshot.head_block;
                        let head_seen = snapshot.head_seen;
                        let peers_connected = peer_pool.len() as u64;
                        let peers_available = snapshot.peers_total.min(peers_connected);
                        let active_fetch = snapshot.peers_active;

                        let (status_str, status_detail) = match snapshot.status {
                            SyncStatus::Following | SyncStatus::UpToDate => {
                                ("Synced", String::new())
                            }
                            SyncStatus::Fetching => (
                                "Catching up",
                                format!(" ({} blocks left)", head_seen.saturating_sub(head_block)),
                            ),
                            SyncStatus::Finalizing => {
                                ("Finalizing", " (flushing/compacting...)".to_string())
                            }
                            SyncStatus::LookingForPeers => ("Waiting for peers", String::new()),
                        };

                        let bar_segment = format_follow_segment(head_block);

                        let compact = if snapshot.status == SyncStatus::Finalizing
                            && snapshot.compactions_total > 0
                        {
                            let done = snapshot.compactions_done.min(snapshot.compactions_total);
                            format!(" | compact {done}/{}", snapshot.compactions_total)
                        } else {
                            String::new()
                        };

                        let msg = format!(
                            "{} {}{}{} | head {} | peers {}/{} | fetch {} | failed {}",
                            bar_segment,
                            status_str,
                            status_detail,
                            compact,
                            head_seen,
                            peers_available,
                            peers_connected,
                            active_fetch,
                            failed,
                        );
                        fb.set_message(msg);
                    }
                }
                UIState::Startup(_) => {
                    // Startup is handled separately
                }
            }
        }
    });
}
