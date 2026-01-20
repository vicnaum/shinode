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

#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessTiming {
    pub total_us: u64,
    pub header_hash_us: u64,
    pub tx_hashes_us: u64,
    pub transactions_us: u64,
    pub withdrawals_us: u64,
    pub block_size_us: u64,
    pub logs_build_us: u64,
}

#[derive(Debug)]
pub struct IngestBenchStats {
    started_at: Instant,
    blocks_total: u64,
    fetch_blocks: AtomicU64,
    fetch_failed_blocks: AtomicU64,
    fetch_batches: AtomicU64,
    fetch_failures: AtomicU64,
    fetch_total_us: AtomicU64,
    fetch_bytes_headers: AtomicU64,
    fetch_bytes_bodies: AtomicU64,
    fetch_bytes_receipts: AtomicU64,
    fetch_bytes_logs: AtomicU64,
    peer_failures: AtomicU64,
    process_blocks: AtomicU64,
    process_failures: AtomicU64,
    process_total_us: AtomicU64,
    process_header_hash_us: AtomicU64,
    process_tx_hashes_us: AtomicU64,
    process_transactions_us: AtomicU64,
    process_withdrawals_us: AtomicU64,
    process_block_size_us: AtomicU64,
    process_logs_build_us: AtomicU64,
    db_write_blocks: AtomicU64,
    db_write_batches: AtomicU64,
    db_write_total_us: AtomicU64,
    db_bytes_headers: AtomicU64,
    db_bytes_tx_hashes: AtomicU64,
    db_bytes_transactions: AtomicU64,
    db_bytes_withdrawals: AtomicU64,
    db_bytes_sizes: AtomicU64,
    db_bytes_receipts: AtomicU64,
    db_bytes_logs: AtomicU64,
}

impl IngestBenchStats {
    pub fn new(blocks_total: u64) -> Self {
        Self {
            started_at: Instant::now(),
            blocks_total,
            fetch_blocks: AtomicU64::new(0),
            fetch_failed_blocks: AtomicU64::new(0),
            fetch_batches: AtomicU64::new(0),
            fetch_failures: AtomicU64::new(0),
            fetch_total_us: AtomicU64::new(0),
            fetch_bytes_headers: AtomicU64::new(0),
            fetch_bytes_bodies: AtomicU64::new(0),
            fetch_bytes_receipts: AtomicU64::new(0),
            fetch_bytes_logs: AtomicU64::new(0),
            peer_failures: AtomicU64::new(0),
            process_blocks: AtomicU64::new(0),
            process_failures: AtomicU64::new(0),
            process_total_us: AtomicU64::new(0),
            process_header_hash_us: AtomicU64::new(0),
            process_tx_hashes_us: AtomicU64::new(0),
            process_transactions_us: AtomicU64::new(0),
            process_withdrawals_us: AtomicU64::new(0),
            process_block_size_us: AtomicU64::new(0),
            process_logs_build_us: AtomicU64::new(0),
            db_write_blocks: AtomicU64::new(0),
            db_write_batches: AtomicU64::new(0),
            db_write_total_us: AtomicU64::new(0),
            db_bytes_headers: AtomicU64::new(0),
            db_bytes_tx_hashes: AtomicU64::new(0),
            db_bytes_transactions: AtomicU64::new(0),
            db_bytes_withdrawals: AtomicU64::new(0),
            db_bytes_sizes: AtomicU64::new(0),
            db_bytes_receipts: AtomicU64::new(0),
            db_bytes_logs: AtomicU64::new(0),
        }
    }

    pub fn record_fetch_success(&self, blocks: u64, elapsed: std::time::Duration) {
        self.fetch_blocks.fetch_add(blocks, Ordering::SeqCst);
        self.fetch_batches.fetch_add(1, Ordering::SeqCst);
        self.fetch_total_us
            .fetch_add(elapsed.as_micros() as u64, Ordering::SeqCst);
    }

    pub fn record_fetch_failure(&self, blocks: u64, elapsed: std::time::Duration) {
        self.fetch_failed_blocks.fetch_add(blocks, Ordering::SeqCst);
        self.fetch_failures.fetch_add(1, Ordering::SeqCst);
        self.fetch_total_us
            .fetch_add(elapsed.as_micros() as u64, Ordering::SeqCst);
    }

    pub fn record_fetch_bytes(&self, bytes: FetchByteTotals) {
        self.fetch_bytes_headers
            .fetch_add(bytes.headers, Ordering::SeqCst);
        self.fetch_bytes_bodies
            .fetch_add(bytes.bodies, Ordering::SeqCst);
        self.fetch_bytes_receipts
            .fetch_add(bytes.receipts, Ordering::SeqCst);
        self.fetch_bytes_logs
            .fetch_add(bytes.logs, Ordering::SeqCst);
    }

    pub fn record_process(&self, timing: ProcessTiming) {
        self.process_blocks.fetch_add(1, Ordering::SeqCst);
        self.process_total_us
            .fetch_add(timing.total_us, Ordering::SeqCst);
        self.process_header_hash_us
            .fetch_add(timing.header_hash_us, Ordering::SeqCst);
        self.process_tx_hashes_us
            .fetch_add(timing.tx_hashes_us, Ordering::SeqCst);
        self.process_transactions_us
            .fetch_add(timing.transactions_us, Ordering::SeqCst);
        self.process_withdrawals_us
            .fetch_add(timing.withdrawals_us, Ordering::SeqCst);
        self.process_block_size_us
            .fetch_add(timing.block_size_us, Ordering::SeqCst);
        self.process_logs_build_us
            .fetch_add(timing.logs_build_us, Ordering::SeqCst);
    }

    pub fn record_process_failure(&self) {
        self.process_failures.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_peer_failure(&self) {
        self.peer_failures.fetch_add(1, Ordering::SeqCst);
    }

    pub fn record_db_write(&self, blocks: u64, elapsed: std::time::Duration) {
        self.db_write_blocks.fetch_add(blocks, Ordering::SeqCst);
        self.db_write_batches.fetch_add(1, Ordering::SeqCst);
        self.db_write_total_us
            .fetch_add(elapsed.as_micros() as u64, Ordering::SeqCst);
    }

    pub fn record_db_write_bytes(&self, bytes: DbWriteByteTotals) {
        self.db_bytes_headers
            .fetch_add(bytes.headers, Ordering::SeqCst);
        self.db_bytes_tx_hashes
            .fetch_add(bytes.tx_hashes, Ordering::SeqCst);
        self.db_bytes_transactions
            .fetch_add(bytes.transactions, Ordering::SeqCst);
        self.db_bytes_withdrawals
            .fetch_add(bytes.withdrawals, Ordering::SeqCst);
        self.db_bytes_sizes
            .fetch_add(bytes.sizes, Ordering::SeqCst);
        self.db_bytes_receipts
            .fetch_add(bytes.receipts, Ordering::SeqCst);
        self.db_bytes_logs
            .fetch_add(bytes.logs, Ordering::SeqCst);
    }

    pub fn summary(
        &self,
        range_start: u64,
        range_end: u64,
        head_at_startup: u64,
        rollback_window_applied: bool,
        peers_used: u64,
        logs_total: u64,
    ) -> IngestBenchSummary {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u64;
        let fetch_blocks = self.fetch_blocks.load(Ordering::SeqCst);
        let fetch_failed_blocks = self.fetch_failed_blocks.load(Ordering::SeqCst);
        let fetch_batches = self.fetch_batches.load(Ordering::SeqCst);
        let fetch_failures = self.fetch_failures.load(Ordering::SeqCst);
        let fetch_total_us = self.fetch_total_us.load(Ordering::SeqCst);
        let fetch_bytes_headers = self.fetch_bytes_headers.load(Ordering::SeqCst);
        let fetch_bytes_bodies = self.fetch_bytes_bodies.load(Ordering::SeqCst);
        let fetch_bytes_receipts = self.fetch_bytes_receipts.load(Ordering::SeqCst);
        let fetch_bytes_logs = self.fetch_bytes_logs.load(Ordering::SeqCst);
        let peer_failures = self.peer_failures.load(Ordering::SeqCst);
        let process_blocks = self.process_blocks.load(Ordering::SeqCst);
        let process_failures = self.process_failures.load(Ordering::SeqCst);
        let process_total_us = self.process_total_us.load(Ordering::SeqCst);
        let db_write_blocks = self.db_write_blocks.load(Ordering::SeqCst);
        let db_write_batches = self.db_write_batches.load(Ordering::SeqCst);
        let db_write_total_us = self.db_write_total_us.load(Ordering::SeqCst);
        let db_bytes_headers = self.db_bytes_headers.load(Ordering::SeqCst);
        let db_bytes_tx_hashes = self.db_bytes_tx_hashes.load(Ordering::SeqCst);
        let db_bytes_transactions = self.db_bytes_transactions.load(Ordering::SeqCst);
        let db_bytes_withdrawals = self.db_bytes_withdrawals.load(Ordering::SeqCst);
        let db_bytes_sizes = self.db_bytes_sizes.load(Ordering::SeqCst);
        let db_bytes_receipts = self.db_bytes_receipts.load(Ordering::SeqCst);
        let db_bytes_logs = self.db_bytes_logs.load(Ordering::SeqCst);

        let fetch_bytes_total = fetch_bytes_headers
            .saturating_add(fetch_bytes_bodies)
            .saturating_add(fetch_bytes_receipts);
        let db_bytes_total = db_bytes_headers
            .saturating_add(db_bytes_tx_hashes)
            .saturating_add(db_bytes_transactions)
            .saturating_add(db_bytes_withdrawals)
            .saturating_add(db_bytes_sizes)
            .saturating_add(db_bytes_receipts)
            .saturating_add(db_bytes_logs);

        let blocks_completed = if db_write_blocks > 0 {
            db_write_blocks
        } else {
            process_blocks
        };
        let blocks_per_sec = rate_per_sec(blocks_completed, elapsed_ms);
        let logs_per_sec = rate_per_sec(logs_total, elapsed_ms);

        let process_breakdown = ProcessBreakdownSummary {
            header_hash_us: self.process_header_hash_us.load(Ordering::SeqCst),
            tx_hashes_us: self.process_tx_hashes_us.load(Ordering::SeqCst),
            transactions_us: self.process_transactions_us.load(Ordering::SeqCst),
            withdrawals_us: self.process_withdrawals_us.load(Ordering::SeqCst),
            block_size_us: self.process_block_size_us.load(Ordering::SeqCst),
            logs_build_us: self.process_logs_build_us.load(Ordering::SeqCst),
        };
        let process_breakdown_avg = ProcessBreakdownSummary {
            header_hash_us: avg_us(process_breakdown.header_hash_us, process_blocks),
            tx_hashes_us: avg_us(process_breakdown.tx_hashes_us, process_blocks),
            transactions_us: avg_us(process_breakdown.transactions_us, process_blocks),
            withdrawals_us: avg_us(process_breakdown.withdrawals_us, process_blocks),
            block_size_us: avg_us(process_breakdown.block_size_us, process_blocks),
            logs_build_us: avg_us(process_breakdown.logs_build_us, process_blocks),
        };

        IngestBenchSummary {
            mode: "ingest-bench",
            range: RangeSummary {
                start_block: range_start,
                end_block: range_end,
                head_at_startup,
                rollback_window_applied,
            },
            totals: IngestTotalsSummary {
                blocks_total: self.blocks_total,
                blocks_fetched: fetch_blocks,
                blocks_processed: process_blocks,
                blocks_written: db_write_blocks,
                logs_total,
                fetch_failed_blocks,
                fetch_failures,
                process_failures,
            },
            performance: IngestPerformanceSummary {
                elapsed_ms,
                blocks_per_sec_avg: blocks_per_sec,
                logs_per_sec_avg: logs_per_sec,
            },
            fetch: IngestFetchSummary {
                total_us: fetch_total_us,
                batches: fetch_batches,
                blocks: fetch_blocks,
                failed_blocks: fetch_failed_blocks,
                failures: fetch_failures,
                avg_us_per_block: avg_us(fetch_total_us, fetch_blocks),
                avg_us_per_batch: avg_us(fetch_total_us, fetch_batches),
                bytes_total: fetch_bytes_total,
                bytes_headers: fetch_bytes_headers,
                bytes_bodies: fetch_bytes_bodies,
                bytes_receipts: fetch_bytes_receipts,
                bytes_logs: fetch_bytes_logs,
                avg_bytes_per_block: avg_u64(fetch_bytes_total, fetch_blocks),
                avg_bytes_per_batch: avg_u64(fetch_bytes_total, fetch_batches),
            },
            process: IngestProcessSummary {
                total_us: process_total_us,
                blocks: process_blocks,
                failures: process_failures,
                avg_us_per_block: avg_us(process_total_us, process_blocks),
                breakdown_us: process_breakdown,
                breakdown_avg_us: process_breakdown_avg,
            },
            db_write: IngestDbWriteSummary {
                total_us: db_write_total_us,
                batches: db_write_batches,
                blocks: db_write_blocks,
                avg_us_per_block: avg_us(db_write_total_us, db_write_blocks),
                avg_us_per_batch: avg_us(db_write_total_us, db_write_batches),
                bytes_total: db_bytes_total,
                bytes_headers: db_bytes_headers,
                bytes_tx_hashes: db_bytes_tx_hashes,
                bytes_transactions: db_bytes_transactions,
                bytes_withdrawals: db_bytes_withdrawals,
                bytes_sizes: db_bytes_sizes,
                bytes_receipts: db_bytes_receipts,
                bytes_logs: db_bytes_logs,
                avg_bytes_per_block: avg_u64(db_bytes_total, db_write_blocks),
                avg_bytes_per_batch: avg_u64(db_bytes_total, db_write_batches),
            },
            peers: PeerSummary {
                peers_used,
                peer_failures_total: peer_failures,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IngestBenchSummary {
    pub mode: &'static str,
    pub range: RangeSummary,
    pub totals: IngestTotalsSummary,
    pub performance: IngestPerformanceSummary,
    pub fetch: IngestFetchSummary,
    pub process: IngestProcessSummary,
    pub db_write: IngestDbWriteSummary,
    pub peers: PeerSummary,
}

#[derive(Debug, Serialize)]
pub struct IngestTotalsSummary {
    pub blocks_total: u64,
    pub blocks_fetched: u64,
    pub blocks_processed: u64,
    pub blocks_written: u64,
    pub logs_total: u64,
    pub fetch_failed_blocks: u64,
    pub fetch_failures: u64,
    pub process_failures: u64,
}

#[derive(Debug, Serialize)]
pub struct IngestPerformanceSummary {
    pub elapsed_ms: u64,
    pub blocks_per_sec_avg: f64,
    pub logs_per_sec_avg: f64,
}

#[derive(Debug, Serialize)]
pub struct IngestFetchSummary {
    pub total_us: u64,
    pub batches: u64,
    pub blocks: u64,
    pub failed_blocks: u64,
    pub failures: u64,
    pub avg_us_per_block: u64,
    pub avg_us_per_batch: u64,
    pub bytes_total: u64,
    pub bytes_headers: u64,
    pub bytes_bodies: u64,
    pub bytes_receipts: u64,
    pub bytes_logs: u64,
    pub avg_bytes_per_block: u64,
    pub avg_bytes_per_batch: u64,
}

#[derive(Debug, Serialize)]
pub struct IngestProcessSummary {
    pub total_us: u64,
    pub blocks: u64,
    pub failures: u64,
    pub avg_us_per_block: u64,
    pub breakdown_us: ProcessBreakdownSummary,
    pub breakdown_avg_us: ProcessBreakdownSummary,
}

#[derive(Debug, Serialize)]
pub struct ProcessBreakdownSummary {
    pub header_hash_us: u64,
    pub tx_hashes_us: u64,
    pub transactions_us: u64,
    pub withdrawals_us: u64,
    pub block_size_us: u64,
    pub logs_build_us: u64,
}

#[derive(Debug, Serialize)]
pub struct IngestDbWriteSummary {
    pub total_us: u64,
    pub batches: u64,
    pub blocks: u64,
    pub avg_us_per_block: u64,
    pub avg_us_per_batch: u64,
    pub bytes_total: u64,
    pub bytes_headers: u64,
    pub bytes_tx_hashes: u64,
    pub bytes_transactions: u64,
    pub bytes_withdrawals: u64,
    pub bytes_sizes: u64,
    pub bytes_receipts: u64,
    pub bytes_logs: u64,
    pub avg_bytes_per_block: u64,
    pub avg_bytes_per_batch: u64,
}

fn avg_us(total_us: u64, count: u64) -> u64 {
    if count == 0 {
        0
    } else {
        total_us / count
    }
}

fn avg_u64(total: u64, count: u64) -> u64 {
    if count == 0 {
        0
    } else {
        total / count
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FetchByteTotals {
    pub headers: u64,
    pub bodies: u64,
    pub receipts: u64,
    pub logs: u64,
}

impl FetchByteTotals {
    pub fn add(&mut self, other: FetchByteTotals) {
        self.headers = self.headers.saturating_add(other.headers);
        self.bodies = self.bodies.saturating_add(other.bodies);
        self.receipts = self.receipts.saturating_add(other.receipts);
        self.logs = self.logs.saturating_add(other.logs);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DbWriteByteTotals {
    pub headers: u64,
    pub tx_hashes: u64,
    pub transactions: u64,
    pub withdrawals: u64,
    pub sizes: u64,
    pub receipts: u64,
    pub logs: u64,
}

impl DbWriteByteTotals {
    pub fn add(&mut self, other: DbWriteByteTotals) {
        self.headers = self.headers.saturating_add(other.headers);
        self.tx_hashes = self.tx_hashes.saturating_add(other.tx_hashes);
        self.transactions = self.transactions.saturating_add(other.transactions);
        self.withdrawals = self.withdrawals.saturating_add(other.withdrawals);
        self.sizes = self.sizes.saturating_add(other.sizes);
        self.receipts = self.receipts.saturating_add(other.receipts);
        self.logs = self.logs.saturating_add(other.logs);
    }
}
