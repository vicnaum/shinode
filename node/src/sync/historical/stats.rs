//! Probe/benchmark statistics aggregation.

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug)]
pub struct ProbeStats {
    started_at: Instant,
    blocks_total: u64,
    blocks_succeeded: AtomicU64,
    blocks_failed: AtomicU64,
    receipts_total: AtomicU64,
    peer_failures: AtomicU64,
    headers_ms: Mutex<Vec<u64>>,
    receipts_ms: Mutex<Vec<u64>>,
    batch_ms: Mutex<Vec<u64>>,
}

impl ProbeStats {
    pub fn new(blocks_total: u64) -> Self {
        Self {
            started_at: Instant::now(),
            blocks_total,
            blocks_succeeded: AtomicU64::new(0),
            blocks_failed: AtomicU64::new(0),
            receipts_total: AtomicU64::new(0),
            peer_failures: AtomicU64::new(0),
            headers_ms: Mutex::new(Vec::new()),
            receipts_ms: Mutex::new(Vec::new()),
            batch_ms: Mutex::new(Vec::new()),
        }
    }

    pub fn record_block(&self, receipts: u64) {
        self.blocks_succeeded.fetch_add(1, Ordering::SeqCst);
        self.receipts_total.fetch_add(receipts, Ordering::SeqCst);
    }

    pub fn record_block_failure(&self) {
        self.blocks_failed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_block_recovered(&self, count: u64) {
        if count == 0 {
            return;
        }
        self.blocks_failed.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            Some(current.saturating_sub(count))
        }).ok();
    }

    pub fn record_peer_failure(&self) {
        self.peer_failures.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_timing(&self, headers_ms: u64, receipts_ms: u64, batch_ms: u64) {
        if let Ok(mut values) = self.headers_ms.lock() {
            values.push(headers_ms);
        }
        if let Ok(mut values) = self.receipts_ms.lock() {
            values.push(receipts_ms);
        }
        if let Ok(mut values) = self.batch_ms.lock() {
            values.push(batch_ms);
        }
    }

    pub fn blocks_succeeded(&self) -> u64 {
        self.blocks_succeeded.load(Ordering::SeqCst)
    }

    pub fn blocks_failed(&self) -> u64 {
        self.blocks_failed.load(Ordering::SeqCst)
    }

    pub fn receipts_total(&self) -> u64 {
        self.receipts_total.load(Ordering::SeqCst)
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }

    pub fn summary(
        &self,
        range_start: u64,
        range_end: u64,
        head_at_startup: u64,
        rollback_window_applied: bool,
        peers_used: u64,
    ) -> ProbeSummary {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u64;
        let blocks_succeeded = self.blocks_succeeded.load(Ordering::SeqCst);
        let blocks_failed = self.blocks_failed.load(Ordering::SeqCst);
        let receipts_total = self.receipts_total.load(Ordering::SeqCst);
        let peer_failures = self.peer_failures.load(Ordering::SeqCst);
        let blocks_per_sec = rate_per_sec(blocks_succeeded, elapsed_ms);
        let receipts_per_sec = rate_per_sec(receipts_total, elapsed_ms);

        let (headers_p50, headers_p95) = percentile_pair(&self.headers_ms);
        let (receipts_p50, receipts_p95) = percentile_pair(&self.receipts_ms);
        let (batch_p50, batch_p95) = percentile_pair(&self.batch_ms);

        ProbeSummary {
            mode: "probe",
            range: RangeSummary {
                start_block: range_start,
                end_block: range_end,
                head_at_startup,
                rollback_window_applied,
            },
            totals: TotalsSummary {
                blocks_total: self.blocks_total,
                blocks_succeeded,
                blocks_failed,
                receipts_total,
            },
            performance: PerformanceSummary {
                elapsed_ms,
                blocks_per_sec_avg: blocks_per_sec,
                receipts_per_sec_avg: receipts_per_sec,
            },
            peers: PeerSummary {
                peers_used,
                peer_failures_total: peer_failures,
            },
            latency: LatencySummary {
                headers_ms_p50: headers_p50,
                headers_ms_p95: headers_p95,
                receipts_ms_p50: receipts_p50,
                receipts_ms_p95: receipts_p95,
                batch_total_ms_p50: batch_p50,
                batch_total_ms_p95: batch_p95,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ProbeSummary {
    pub mode: &'static str,
    pub range: RangeSummary,
    pub totals: TotalsSummary,
    pub performance: PerformanceSummary,
    pub peers: PeerSummary,
    pub latency: LatencySummary,
}

#[derive(Debug, Serialize)]
pub struct RangeSummary {
    pub start_block: u64,
    pub end_block: u64,
    pub head_at_startup: u64,
    pub rollback_window_applied: bool,
}

#[derive(Debug, Serialize)]
pub struct TotalsSummary {
    pub blocks_total: u64,
    pub blocks_succeeded: u64,
    pub blocks_failed: u64,
    pub receipts_total: u64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceSummary {
    pub elapsed_ms: u64,
    pub blocks_per_sec_avg: f64,
    pub receipts_per_sec_avg: f64,
}

#[derive(Debug, Serialize)]
pub struct PeerSummary {
    pub peers_used: u64,
    pub peer_failures_total: u64,
}

#[derive(Debug, Serialize)]
pub struct LatencySummary {
    pub headers_ms_p50: Option<u64>,
    pub headers_ms_p95: Option<u64>,
    pub receipts_ms_p50: Option<u64>,
    pub receipts_ms_p95: Option<u64>,
    pub batch_total_ms_p50: Option<u64>,
    pub batch_total_ms_p95: Option<u64>,
}

fn rate_per_sec(value: u64, elapsed_ms: u64) -> f64 {
    if elapsed_ms == 0 {
        return 0.0;
    }
    value as f64 / (elapsed_ms as f64 / 1000.0)
}

fn percentile_pair(values: &Mutex<Vec<u64>>) -> (Option<u64>, Option<u64>) {
    let mut data = match values.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => Vec::new(),
    };
    if data.is_empty() {
        return (None, None);
    }
    data.sort_unstable();
    let p50 = percentile(&data, 0.50);
    let p95 = percentile(&data, 0.95);
    (p50, p95)
}

fn percentile(sorted: &[u64], p: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let clamped = p.clamp(0.0, 1.0);
    let idx = ((sorted.len() - 1) as f64 * clamped).round() as usize;
    sorted.get(idx).copied()
}
