//! Session types for managing sync state.

use indicatif::ProgressBar;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::sync::ProgressReporter;

/// Progress tracker that wraps a progress bar with atomic length tracking.
pub struct IngestProgress {
    bar: ProgressBar,
    total_len: AtomicU64,
}

impl IngestProgress {
    pub fn new(bar: ProgressBar, total_len: u64) -> Self {
        bar.set_length(total_len);
        Self {
            bar,
            total_len: AtomicU64::new(total_len),
        }
    }

    pub fn finish(&self) {
        self.bar.finish_and_clear();
    }
}

impl ProgressReporter for IngestProgress {
    fn set_length(&self, len: u64) {
        let current = self.total_len.load(Ordering::SeqCst);
        if len > current {
            self.total_len.store(len, Ordering::SeqCst);
            self.bar.set_length(len);
        }
    }

    fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }
}

/// Resources specific to follow mode that need explicit cleanup.
#[expect(dead_code, reason = "fields accessed via destructuring and method calls")]
pub struct FollowModeResources {
    pub head_tracker: Option<JoinHandle<()>>,
    pub tail_feeder: Option<JoinHandle<()>>,
    pub head_stop_tx: Option<watch::Sender<bool>>,
    pub tail_stop_tx: Option<watch::Sender<bool>>,
    pub head_seen_rx: Option<watch::Receiver<u64>>,
}

impl FollowModeResources {
    /// Create empty resources (for non-follow mode).
    pub fn empty() -> Self {
        Self {
            head_tracker: None,
            tail_feeder: None,
            head_stop_tx: None,
            tail_stop_tx: None,
            head_seen_rx: None,
        }
    }

    /// Signal stop to both trackers and await their handles.
    pub async fn shutdown(&mut self) {
        // Stop tail feeder first
        if let Some(stop_tx) = self.tail_stop_tx.take() {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = self.tail_feeder.take() {
            let _ = handle.await;
        }

        // Stop head tracker
        if let Some(stop_tx) = self.head_stop_tx.take() {
            let _ = stop_tx.send(true);
        }
        if let Some(handle) = self.head_tracker.take() {
            let _ = handle.await;
        }
    }

    /// Check if follow mode is enabled.
    #[expect(dead_code, reason = "API for checking follow mode state")]
    pub fn is_enabled(&self) -> bool {
        self.head_tracker.is_some()
    }
}
