//! Sink stage for historical sync.

use crate::sync::historical::stats::ProbeStats;
use crate::sync::historical::types::ProbeRecord;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Consume probe records and update stats (no DB writes).
pub async fn run_probe_sink(mut rx: mpsc::Receiver<ProbeRecord>, stats: Arc<ProbeStats>) {
    while let Some(record) = rx.recv().await {
        stats.record_block(record.receipts);
        stats.record_timing(
            record.timing.headers_ms,
            record.timing.receipts_ms,
            record.timing.total_ms,
        );
    }
}
