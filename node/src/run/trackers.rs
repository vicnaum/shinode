//! Head tracking and tail feeding for follow mode.

use crate::p2p::PeerPool;
use crate::storage::Storage;
use crate::sync::historical::TailIngestConfig;
use crate::sync::SyncProgressStats;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

/// Handles returned from spawning the head tracker.
pub struct HeadTrackerHandles {
    pub handle: JoinHandle<()>,
    pub stop_tx: watch::Sender<bool>,
    pub head_seen_rx: watch::Receiver<u64>,
}

/// Handles returned from spawning the tail feeder.
#[expect(dead_code, reason = "fields accessed via destructuring")]
pub struct TailFeederHandles {
    pub handle: JoinHandle<()>,
    pub stop_tx: watch::Sender<bool>,
    pub ranges_rx: mpsc::UnboundedReceiver<RangeInclusive<u64>>,
}

/// Spawn the head tracker task that monitors peer heads and updates storage/stats.
pub fn spawn_head_tracker(
    pool: Arc<PeerPool>,
    storage: Arc<Storage>,
    stats: Option<Arc<SyncProgressStats>>,
    initial_head: u64,
    rollback_window: u64,
) -> HeadTrackerHandles {
    let (stop_tx, stop_rx) = watch::channel(false);
    let (head_seen_tx, head_seen_rx) = watch::channel(initial_head);

    let mut stop_rx_head = stop_rx.clone();
    let handle = tokio::spawn(async move {
        let mut last_head = initial_head;
        loop {
            if *stop_rx_head.borrow() {
                break;
            }
            let snapshot = pool.snapshot();
            let best_head = snapshot
                .iter()
                .map(|peer| peer.head_number)
                .max()
                .unwrap_or(last_head);
            if best_head > last_head {
                last_head = best_head;
                let _ = head_seen_tx.send(best_head);
                if let Err(err) = storage.set_head_seen(best_head) {
                    tracing::debug!(error = %err, "head tracker: failed to persist head_seen");
                }
                if let Some(stats) = stats.as_ref() {
                    stats.set_head_seen(best_head);
                }
            }
            tokio::select! {
                _ = stop_rx_head.changed() => {
                    if *stop_rx_head.borrow() {
                        break;
                    }
                }
                () = tokio::time::sleep(Duration::from_secs(1)) => {}
            }
        }
    });

    // Create a separate receiver for TailIngestConfig
    let _ = rollback_window; // Used by caller for tail feeder

    HeadTrackerHandles {
        handle,
        stop_tx,
        head_seen_rx,
    }
}

/// Spawn the tail feeder task that emits new block ranges as the head advances.
pub fn spawn_tail_feeder(
    initial_end: u64,
    rollback_window: u64,
    head_seen_rx: &watch::Receiver<u64>,
) -> TailFeederHandles {
    let (stop_tx, stop_rx) = watch::channel(false);
    let (tail_tx, ranges_rx) = mpsc::unbounded_channel();

    let mut stop_rx_tail = stop_rx.clone();
    let mut head_seen_rx_tail = head_seen_rx.clone();
    let mut next_to_schedule = initial_end.saturating_add(1);

    let handle = tokio::spawn(async move {
        loop {
            if *stop_rx_tail.borrow() {
                break;
            }
            let head_seen = *head_seen_rx_tail.borrow();
            let safe_head = head_seen.saturating_sub(rollback_window);
            if safe_head >= next_to_schedule {
                let start = next_to_schedule;
                let end = safe_head;
                let _ = tail_tx.send(start..=end);
                next_to_schedule = end.saturating_add(1);
            }
            tokio::select! {
                _ = head_seen_rx_tail.changed() => {}
                _ = stop_rx_tail.changed() => {
                    if *stop_rx_tail.borrow() {
                        break;
                    }
                }
                () = tokio::time::sleep(Duration::from_millis(500)) => {}
            }
        }
    });

    TailFeederHandles {
        handle,
        stop_tx,
        ranges_rx,
    }
}

/// Build TailIngestConfig from tail feeder handles.
pub fn build_tail_config(
    tail_handles: TailFeederHandles,
    head_seen_rx: watch::Receiver<u64>,
    rollback_window: u64,
) -> TailIngestConfig {
    TailIngestConfig {
        ranges_rx: tail_handles.ranges_rx,
        head_seen_rx,
        stop_tx: tail_handles.stop_tx,
        stop_when_caught_up: true,
        head_offset: rollback_window,
    }
}
