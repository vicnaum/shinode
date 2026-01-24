//! Probe/benchmark statistics aggregation.

use crate::storage::StorageDiskStats;
use serde::Serialize;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread::JoinHandle;
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
        self.blocks_failed
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                Some(current.saturating_sub(count))
            })
            .ok();
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

    #[allow(dead_code)]
    pub fn receipts_total(&self) -> u64 {
        self.receipts_total.load(Ordering::SeqCst)
    }

    #[allow(dead_code)]
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

fn percentile_triplet(values: &Mutex<Vec<u64>>) -> (Option<u64>, Option<u64>, Option<u64>) {
    let mut data = match values.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => Vec::new(),
    };
    if data.is_empty() {
        return (None, None, None);
    }
    data.sort_unstable();
    let p50 = percentile(&data, 0.50);
    let p95 = percentile(&data, 0.95);
    let p99 = percentile(&data, 0.99);
    (p50, p95, p99)
}

fn percentile(sorted: &[u64], p: f64) -> Option<u64> {
    if sorted.is_empty() {
        return None;
    }
    let clamped = p.clamp(0.0, 1.0);
    let idx = ((sorted.len() - 1) as f64 * clamped).round() as usize;
    sorted.get(idx).copied()
}

const SAMPLE_LIMIT: usize = 100_000;

fn push_sample(samples: &Mutex<Vec<u64>>, value: u64) {
    if let Ok(mut guard) = samples.lock() {
        if guard.len() < SAMPLE_LIMIT {
            guard.push(value);
        }
    }
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
    fetch_headers_requests: AtomicU64,
    fetch_bodies_requests: AtomicU64,
    fetch_receipts_requests: AtomicU64,
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
    process_total_samples_us: Mutex<Vec<u64>>,
    db_write_blocks: AtomicU64,
    db_write_batches: AtomicU64,
    db_write_total_us: AtomicU64,
    db_write_ms_samples: Mutex<Vec<u64>>,
    db_bytes_headers: AtomicU64,
    db_bytes_tx_hashes: AtomicU64,
    db_bytes_transactions: AtomicU64,
    db_bytes_withdrawals: AtomicU64,
    db_bytes_sizes: AtomicU64,
    db_bytes_receipts: AtomicU64,
    db_bytes_logs: AtomicU64,
    logs_total: AtomicU64,
    fetch_headers_ms: Mutex<Vec<u64>>,
    fetch_bodies_ms: Mutex<Vec<u64>>,
    fetch_receipts_ms: Mutex<Vec<u64>>,
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
            fetch_headers_requests: AtomicU64::new(0),
            fetch_bodies_requests: AtomicU64::new(0),
            fetch_receipts_requests: AtomicU64::new(0),
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
            process_total_samples_us: Mutex::new(Vec::new()),
            db_write_blocks: AtomicU64::new(0),
            db_write_batches: AtomicU64::new(0),
            db_write_total_us: AtomicU64::new(0),
            db_write_ms_samples: Mutex::new(Vec::new()),
            db_bytes_headers: AtomicU64::new(0),
            db_bytes_tx_hashes: AtomicU64::new(0),
            db_bytes_transactions: AtomicU64::new(0),
            db_bytes_withdrawals: AtomicU64::new(0),
            db_bytes_sizes: AtomicU64::new(0),
            db_bytes_receipts: AtomicU64::new(0),
            db_bytes_logs: AtomicU64::new(0),
            logs_total: AtomicU64::new(0),
            fetch_headers_ms: Mutex::new(Vec::new()),
            fetch_bodies_ms: Mutex::new(Vec::new()),
            fetch_receipts_ms: Mutex::new(Vec::new()),
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

    pub fn record_fetch_stage_stats(
        &self,
        headers_ms: u64,
        bodies_ms: u64,
        receipts_ms: u64,
        headers_requests: u64,
        bodies_requests: u64,
        receipts_requests: u64,
    ) {
        if headers_ms > 0 {
            push_sample(&self.fetch_headers_ms, headers_ms);
        }
        if bodies_ms > 0 {
            push_sample(&self.fetch_bodies_ms, bodies_ms);
        }
        if receipts_ms > 0 {
            push_sample(&self.fetch_receipts_ms, receipts_ms);
        }
        self.fetch_headers_requests
            .fetch_add(headers_requests, Ordering::SeqCst);
        self.fetch_bodies_requests
            .fetch_add(bodies_requests, Ordering::SeqCst);
        self.fetch_receipts_requests
            .fetch_add(receipts_requests, Ordering::SeqCst);
    }

    pub fn record_process(&self, timing: ProcessTiming) {
        self.process_blocks.fetch_add(1, Ordering::SeqCst);
        self.process_total_us
            .fetch_add(timing.total_us, Ordering::SeqCst);
        push_sample(&self.process_total_samples_us, timing.total_us);
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
        push_sample(&self.db_write_ms_samples, elapsed.as_millis() as u64);
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
        self.db_bytes_sizes.fetch_add(bytes.sizes, Ordering::SeqCst);
        self.db_bytes_receipts
            .fetch_add(bytes.receipts, Ordering::SeqCst);
        self.db_bytes_logs.fetch_add(bytes.logs, Ordering::SeqCst);
    }

    pub fn record_logs(&self, count: u64) {
        self.logs_total.fetch_add(count, Ordering::SeqCst);
    }

    #[allow(dead_code)]
    pub fn logs_total(&self) -> u64 {
        self.logs_total.load(Ordering::SeqCst)
    }

    pub fn summary(
        &self,
        range_start: u64,
        range_end: u64,
        head_at_startup: u64,
        rollback_window_applied: bool,
        peers_used: u64,
        logs_total: u64,
        storage_stats: Option<StorageDiskStats>,
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
        let fetch_headers_requests = self.fetch_headers_requests.load(Ordering::SeqCst);
        let fetch_bodies_requests = self.fetch_bodies_requests.load(Ordering::SeqCst);
        let fetch_receipts_requests = self.fetch_receipts_requests.load(Ordering::SeqCst);
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

        let (headers_ms_p50, headers_ms_p95, headers_ms_p99) =
            percentile_triplet(&self.fetch_headers_ms);
        let (bodies_ms_p50, bodies_ms_p95, bodies_ms_p99) =
            percentile_triplet(&self.fetch_bodies_ms);
        let (receipts_ms_p50, receipts_ms_p95, receipts_ms_p99) =
            percentile_triplet(&self.fetch_receipts_ms);
        let (process_total_us_p50, process_total_us_p95, process_total_us_p99) =
            percentile_triplet(&self.process_total_samples_us);
        let (db_flush_ms_p50, db_flush_ms_p95, db_flush_ms_p99) =
            percentile_triplet(&self.db_write_ms_samples);

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
        let fetch_seconds = fetch_total_us as f64 / 1_000_000.0;
        let fetch_bytes_per_sec = if fetch_seconds > 0.0 {
            fetch_bytes_total as f64 / fetch_seconds
        } else {
            0.0
        };
        let fetch_mib_per_sec = fetch_bytes_per_sec / (1024.0 * 1024.0);
        let db_seconds = db_write_total_us as f64 / 1_000_000.0;
        let db_bytes_per_sec = if db_seconds > 0.0 {
            db_bytes_total as f64 / db_seconds
        } else {
            0.0
        };
        let db_mib_per_sec = db_bytes_per_sec / (1024.0 * 1024.0);

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
                download_bytes_per_sec_avg: fetch_bytes_per_sec,
                download_mib_per_sec_avg: fetch_mib_per_sec,
                headers_requests: fetch_headers_requests,
                bodies_requests: fetch_bodies_requests,
                receipts_requests: fetch_receipts_requests,
                headers_ms_p50,
                headers_ms_p95,
                headers_ms_p99,
                bodies_ms_p50,
                bodies_ms_p95,
                bodies_ms_p99,
                receipts_ms_p50,
                receipts_ms_p95,
                receipts_ms_p99,
            },
            process: IngestProcessSummary {
                total_us: process_total_us,
                blocks: process_blocks,
                failures: process_failures,
                avg_us_per_block: avg_us(process_total_us, process_blocks),
                total_us_p50: process_total_us_p50,
                total_us_p95: process_total_us_p95,
                total_us_p99: process_total_us_p99,
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
                flush_ms_p50: db_flush_ms_p50,
                flush_ms_p95: db_flush_ms_p95,
                flush_ms_p99: db_flush_ms_p99,
                write_bytes_per_sec_avg: db_bytes_per_sec,
                write_mib_per_sec_avg: db_mib_per_sec,
            },
            storage: storage_stats,
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
    pub storage: Option<StorageDiskStats>,
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
    pub download_bytes_per_sec_avg: f64,
    pub download_mib_per_sec_avg: f64,
    pub headers_requests: u64,
    pub bodies_requests: u64,
    pub receipts_requests: u64,
    pub headers_ms_p50: Option<u64>,
    pub headers_ms_p95: Option<u64>,
    pub headers_ms_p99: Option<u64>,
    pub bodies_ms_p50: Option<u64>,
    pub bodies_ms_p95: Option<u64>,
    pub bodies_ms_p99: Option<u64>,
    pub receipts_ms_p50: Option<u64>,
    pub receipts_ms_p95: Option<u64>,
    pub receipts_ms_p99: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct IngestProcessSummary {
    pub total_us: u64,
    pub blocks: u64,
    pub failures: u64,
    pub avg_us_per_block: u64,
    pub total_us_p50: Option<u64>,
    pub total_us_p95: Option<u64>,
    pub total_us_p99: Option<u64>,
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
    pub flush_ms_p50: Option<u64>,
    pub flush_ms_p95: Option<u64>,
    pub flush_ms_p99: Option<u64>,
    pub write_bytes_per_sec_avg: f64,
    pub write_mib_per_sec_avg: f64,
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
    #[allow(dead_code)]
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

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BenchEvent {
    ResourcesSample {
        rss_kb: u64,
        swap_kb: u64,
        rss_anon_kb: u64,
        rss_file_kb: u64,
        rss_shmem_kb: u64,
        cpu_busy_pct: f64,
        cpu_iowait_pct: f64,
        disk_read_mib_s: f64,
        disk_write_mib_s: f64,
        status: Option<String>,
        processed: Option<u64>,
        queue: Option<u64>,
        inflight: Option<u64>,
        failed: Option<u64>,
        peers_active: Option<u64>,
        peers_total: Option<u64>,
        head_block: Option<u64>,
        head_seen: Option<u64>,
    },
    SchedulerGaugeSample {
        pending_total: u64,
        pending_main: u64,
        inflight: u64,
        failed: u64,
        completed: u64,
        attempts: u64,
        escalation_len: u64,
        escalation_attempted: u64,
    },
    DbWriterGaugeSample {
        buffer_len: u64,
        compactions_total: u64,
        compactions_inflight: u64,
    },
    BatchAssigned {
        peer_id: String,
        range_start: u64,
        range_end: u64,
        blocks: u64,
        batch_limit: u64,
        mode: &'static str,
    },
    FetchStart {
        peer_id: String,
        range_start: u64,
        range_end: u64,
        blocks: u64,
        batch_limit: u64,
    },
    FetchEnd {
        peer_id: String,
        range_start: u64,
        range_end: u64,
        blocks: u64,
        batch_limit: u64,
        duration_ms: u64,
        missing_blocks: u64,
        bytes_headers: u64,
        bytes_bodies: u64,
        bytes_receipts: u64,
        headers_ms: u64,
        bodies_ms: u64,
        receipts_ms: u64,
        headers_requests: u64,
        bodies_requests: u64,
        receipts_requests: u64,
    },
    FetchTimeout {
        peer_id: String,
        range_start: u64,
        range_end: u64,
        blocks: u64,
        batch_limit: u64,
        timeout_ms: u64,
    },
    ProcessStart {
        block: u64,
    },
    ProcessEnd {
        block: u64,
        duration_us: u64,
        logs: u64,
    },
    DbFlushStart {
        blocks: u64,
        bytes_total: u64,
    },
    DbFlushEnd {
        blocks: u64,
        bytes_total: u64,
        duration_ms: u64,
    },
    CompactionStart {
        shard_start: u64,
    },
    CompactionEnd {
        shard_start: u64,
        duration_ms: u64,
    },
    CompactAllDirtyStart,
    CompactAllDirtyEnd {
        duration_ms: u64,
    },
    SealCompletedStart,
    SealCompletedEnd {
        duration_ms: u64,
    },
    PeerConnected {
        peer_id: String,
    },
    PeerDisconnected {
        peer_id: String,
    },
    PeerBanned {
        peer_id: String,
        reason: &'static str,
    },
    PeerUnbanned {
        peer_id: String,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct BenchEventRecord {
    pub t_ms: u64,
    #[serde(flatten)]
    pub event: BenchEvent,
}

#[derive(Debug)]
enum BenchEventMessage {
    Record(BenchEventRecord),
}

#[derive(Debug)]
pub struct BenchEventLogger {
    started_at: Instant,
    sender: Mutex<Option<mpsc::Sender<BenchEventMessage>>>,
    handle: Mutex<Option<JoinHandle<eyre::Result<()>>>>,
    dropped_events: AtomicU64,
    total_events: AtomicU64,
}

impl BenchEventLogger {
    pub fn new(path: PathBuf) -> eyre::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(&path)?;
        let mut writer = BufWriter::new(file);

        let (tx, rx) = mpsc::channel::<BenchEventMessage>();
        let handle = std::thread::spawn(move || -> eyre::Result<()> {
            let mut since_flush = 0usize;
            for msg in rx {
                match msg {
                    BenchEventMessage::Record(record) => {
                        serde_json::to_writer(&mut writer, &record)?;
                        writer.write_all(b"\n")?;
                        since_flush = since_flush.saturating_add(1);
                        if since_flush >= 4096 {
                            writer.flush()?;
                            since_flush = 0;
                        }
                    }
                }
            }
            writer.flush()?;
            Ok(())
        });

        Ok(Self {
            started_at: Instant::now(),
            sender: Mutex::new(Some(tx)),
            handle: Mutex::new(Some(handle)),
            dropped_events: AtomicU64::new(0),
            total_events: AtomicU64::new(0),
        })
    }

    pub fn record(&self, event: BenchEvent) {
        let record = BenchEventRecord {
            t_ms: self.started_at.elapsed().as_millis() as u64,
            event,
        };
        let sender = self
            .sender
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().cloned());
        if let Some(sender) = sender {
            match sender.send(BenchEventMessage::Record(record)) {
                Ok(()) => {
                    self.total_events.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    self.dropped_events.fetch_add(1, Ordering::SeqCst);
                }
            }
        } else {
            self.dropped_events.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub fn finish(&self) -> eyre::Result<()> {
        let sender = match self.sender.lock() {
            Ok(mut guard) => guard.take(),
            Err(_) => None,
        };
        drop(sender);

        let handle = match self.handle.lock() {
            Ok(mut guard) => guard.take(),
            Err(_) => None,
        };
        if let Some(handle) = handle {
            match handle.join() {
                Ok(res) => res?,
                Err(_) => return Err(eyre::eyre!("event writer thread panicked")),
            }
        }
        Ok(())
    }

    pub fn dropped_events(&self) -> u64 {
        self.dropped_events.load(Ordering::SeqCst)
    }

    pub fn total_events(&self) -> u64 {
        self.total_events.load(Ordering::SeqCst)
    }
}

impl Drop for BenchEventLogger {
    fn drop(&mut self) {
        let sender = self.sender.lock().ok().and_then(|mut g| g.take());
        drop(sender);
        let handle = self.handle.lock().ok().and_then(|mut g| g.take());
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }
}
