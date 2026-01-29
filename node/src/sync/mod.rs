//! Sync and ingest orchestration.

use reth_ethereum_primitives::{BlockBody, Receipt};
use reth_primitives_traits::Header;

pub mod historical;

/// Full payload for a block: header, body, receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPayload {
    pub header: Header,
    pub body: BlockBody,
    pub receipts: Vec<Receipt>,
}

pub trait ProgressReporter: Send + Sync {
    fn set_length(&self, len: u64);
    fn inc(&self, delta: u64);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    LookingForPeers,
    Fetching,
    Finalizing,
    UpToDate,
    Following,
}

/// Sub-phase during finalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FinalizePhase {
    #[default]
    Compacting,
    Sealing,
}

impl SyncStatus {
    /// Machine-readable status string (for logging/metrics).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LookingForPeers => "looking_for_peers",
            Self::Fetching => "fetching",
            Self::Finalizing => "finalizing",
            Self::UpToDate => "up_to_date",
            Self::Following => "following",
        }
    }

    /// Human-readable display name (for UI).
    #[expect(dead_code, reason = "API for UI display")]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::LookingForPeers => "Waiting for peers",
            Self::Fetching => "Syncing",
            Self::Finalizing => "Finalizing",
            Self::UpToDate | Self::Following => "Synced",
        }
    }
}

pub fn format_eta_seconds(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return "--".to_string();
    }
    let mut remaining = seconds.round().max(0.0) as u64;
    let days = remaining / 86_400;
    remaining %= 86_400;
    let hours = remaining / 3_600;
    remaining %= 3_600;
    let mins = remaining / 60;
    let secs = remaining % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else if mins > 0 {
        format!("{mins}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

/// Coverage tracking for the blocks map visualization.
/// Stores (start_block, end_block, buckets) where each bucket is a count of synced blocks.
#[derive(Debug, Default)]
pub struct CoverageTracker {
    /// Start block of the range being tracked.
    start_block: u64,
    /// End block of the range being tracked.
    end_block: u64,
    /// Number of synced blocks per bucket.
    buckets: Vec<u16>,
}

impl CoverageTracker {
    /// Initialize coverage tracking for a block range.
    pub fn init(&mut self, start: u64, end: u64, num_buckets: usize) {
        self.start_block = start;
        self.end_block = end;
        self.buckets = vec![0; num_buckets];
    }

    /// Record a completed block, incrementing the appropriate bucket.
    pub fn record_completed(&mut self, block: u64) {
        if block < self.start_block || block >= self.end_block || self.buckets.is_empty() {
            return;
        }
        let range = self.end_block - self.start_block;
        let offset = block - self.start_block;
        let bucket_idx = ((offset as f64 / range as f64) * self.buckets.len() as f64) as usize;
        if bucket_idx < self.buckets.len() {
            self.buckets[bucket_idx] = self.buckets[bucket_idx].saturating_add(1);
        }
    }

    /// Record multiple completed blocks.
    pub fn record_completed_batch(&mut self, blocks: &[u64]) {
        for &block in blocks {
            self.record_completed(block);
        }
    }

}

#[derive(Debug, Default)]
pub struct SyncProgressStats {
    processed: std::sync::atomic::AtomicU64,
    queue: std::sync::atomic::AtomicU64,
    inflight: std::sync::atomic::AtomicU64,
    compactions_done: std::sync::atomic::AtomicU64,
    compactions_total: std::sync::atomic::AtomicU64,
    /// Separate counters for sealing phase (distinct from compaction).
    sealings_done: std::sync::atomic::AtomicU64,
    sealings_total: std::sync::atomic::AtomicU64,
    peers_active: std::sync::atomic::AtomicU64,
    peers_total: std::sync::atomic::AtomicU64,
    peers_stale: std::sync::atomic::AtomicU64,
    status: std::sync::atomic::AtomicU8,
    head_block: std::sync::atomic::AtomicU64,
    head_seen: std::sync::atomic::AtomicU64,
    /// True when all fetch tasks have completed (blocks downloaded from peers).
    fetch_complete: std::sync::atomic::AtomicBool,
    /// Blocks in escalation queue (priority retry for difficult blocks).
    escalation: std::sync::atomic::AtomicU64,
    /// Current finalize sub-phase (compacting or sealing).
    finalize_phase: std::sync::atomic::AtomicU8,
    /// First block of the sync range (from CLI).
    start_block: std::sync::atomic::AtomicU64,
    /// Peak speed observed during this session (blocks/s).
    peak_speed: std::sync::atomic::AtomicU64,
    /// Timestamp (ms since epoch) when last block was received. Used for "Xs ago" display.
    last_block_received_ms: std::sync::atomic::AtomicU64,
    /// True when RPC server has been started and is serving requests.
    rpc_active: std::sync::atomic::AtomicBool,
    /// RPC counters for TUI display.
    rpc_total_requests: std::sync::atomic::AtomicU64,
    rpc_get_logs: std::sync::atomic::AtomicU64,
    rpc_get_block: std::sync::atomic::AtomicU64,
    rpc_errors: std::sync::atomic::AtomicU64,
    /// Coverage tracking for blocks map visualization.
    coverage: parking_lot::Mutex<CoverageTracker>,
}

#[derive(Debug, Clone, Copy)]
pub struct SyncProgressSnapshot {
    pub processed: u64,
    pub queue: u64,
    pub inflight: u64,
    pub compactions_done: u64,
    pub compactions_total: u64,
    /// Separate counters for sealing phase (distinct from compaction).
    pub sealings_done: u64,
    pub sealings_total: u64,
    pub peers_active: u64,
    pub peers_total: u64,
    pub peers_stale: u64,
    pub status: SyncStatus,
    pub head_block: u64,
    pub head_seen: u64,
    /// True when all fetch tasks have completed (blocks downloaded from peers).
    pub fetch_complete: bool,
    /// Blocks in escalation queue (priority retry for difficult blocks).
    pub escalation: u64,
    /// Current finalize sub-phase.
    pub finalize_phase: FinalizePhase,
    /// First block of the sync range (from CLI).
    pub start_block: u64,
    /// Peak speed observed during this session (blocks/s).
    pub peak_speed: u64,
    /// Timestamp (ms since epoch) when last block was received.
    pub last_block_received_ms: u64,
    /// True when RPC server has been started and is serving requests.
    pub rpc_active: bool,
    /// RPC counters for TUI display.
    pub rpc_total_requests: u64,
    pub rpc_get_logs: u64,
    pub rpc_get_block: u64,
    pub rpc_errors: u64,
}

impl SyncProgressStats {
    pub fn snapshot(&self) -> SyncProgressSnapshot {
        SyncProgressSnapshot {
            processed: self.processed.load(std::sync::atomic::Ordering::SeqCst),
            queue: self.queue.load(std::sync::atomic::Ordering::SeqCst),
            inflight: self.inflight.load(std::sync::atomic::Ordering::SeqCst),
            compactions_done: self
                .compactions_done
                .load(std::sync::atomic::Ordering::SeqCst),
            compactions_total: self
                .compactions_total
                .load(std::sync::atomic::Ordering::SeqCst),
            sealings_done: self
                .sealings_done
                .load(std::sync::atomic::Ordering::SeqCst),
            sealings_total: self
                .sealings_total
                .load(std::sync::atomic::Ordering::SeqCst),
            peers_active: self.peers_active.load(std::sync::atomic::Ordering::SeqCst),
            peers_total: self.peers_total.load(std::sync::atomic::Ordering::SeqCst),
            peers_stale: self.peers_stale.load(std::sync::atomic::Ordering::SeqCst),
            status: match self.status.load(std::sync::atomic::Ordering::SeqCst) {
                1 => SyncStatus::Fetching,
                2 => SyncStatus::Finalizing,
                3 => SyncStatus::UpToDate,
                4 => SyncStatus::Following,
                _ => SyncStatus::LookingForPeers,
            },
            head_block: self.head_block.load(std::sync::atomic::Ordering::SeqCst),
            head_seen: self.head_seen.load(std::sync::atomic::Ordering::SeqCst),
            fetch_complete: self
                .fetch_complete
                .load(std::sync::atomic::Ordering::SeqCst),
            escalation: self.escalation.load(std::sync::atomic::Ordering::SeqCst),
            finalize_phase: match self.finalize_phase.load(std::sync::atomic::Ordering::SeqCst) {
                1 => FinalizePhase::Sealing,
                _ => FinalizePhase::Compacting,
            },
            start_block: self.start_block.load(std::sync::atomic::Ordering::SeqCst),
            peak_speed: self.peak_speed.load(std::sync::atomic::Ordering::SeqCst),
            last_block_received_ms: self
                .last_block_received_ms
                .load(std::sync::atomic::Ordering::SeqCst),
            rpc_active: self
                .rpc_active
                .load(std::sync::atomic::Ordering::SeqCst),
            rpc_total_requests: self
                .rpc_total_requests
                .load(std::sync::atomic::Ordering::SeqCst),
            rpc_get_logs: self
                .rpc_get_logs
                .load(std::sync::atomic::Ordering::SeqCst),
            rpc_get_block: self
                .rpc_get_block
                .load(std::sync::atomic::Ordering::SeqCst),
            rpc_errors: self
                .rpc_errors
                .load(std::sync::atomic::Ordering::SeqCst),
        }
    }

    pub fn set_status(&self, status: SyncStatus) {
        let value = match status {
            SyncStatus::LookingForPeers => 0,
            SyncStatus::Fetching => 1,
            SyncStatus::Finalizing => 2,
            SyncStatus::UpToDate => 3,
            SyncStatus::Following => 4,
        };
        self.status
            .store(value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn inc_processed(&self, delta: u64) {
        self.processed
            .fetch_add(delta, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_queue(&self, queue: u64) {
        self.queue.store(queue, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_inflight(&self, inflight: u64) {
        self.inflight
            .store(inflight, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_compactions_total(&self, total: u64) {
        self.compactions_total
            .store(total, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn inc_compactions_done(&self, delta: u64) {
        if delta == 0 {
            return;
        }
        self.compactions_done
            .fetch_add(delta, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_compactions_done(&self, done: u64) {
        self.compactions_done
            .store(done, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_sealings_total(&self, total: u64) {
        self.sealings_total
            .store(total, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn inc_sealings_done(&self, delta: u64) {
        if delta == 0 {
            return;
        }
        self.sealings_done
            .fetch_add(delta, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_sealings_done(&self, done: u64) {
        self.sealings_done
            .store(done, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_peers_active(&self, active: u64) {
        self.peers_active
            .store(active, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_peers_total(&self, total: u64) {
        self.peers_total
            .store(total, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_peers_stale(&self, stale: u64) {
        self.peers_stale
            .store(stale, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_head_block(&self, block: u64) {
        self.head_block
            .store(block, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn update_head_block_max(&self, block: u64) {
        let _ = self.head_block.fetch_update(
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
            |current| Some(current.max(block)),
        );
    }

    pub fn set_head_seen(&self, block: u64) {
        self.head_seen
            .store(block, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_fetch_complete(&self, complete: bool) {
        self.fetch_complete
            .store(complete, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_escalation(&self, count: u64) {
        self.escalation
            .store(count, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_finalize_phase(&self, phase: FinalizePhase) {
        let value = match phase {
            FinalizePhase::Compacting => 0,
            FinalizePhase::Sealing => 1,
        };
        self.finalize_phase
            .store(value, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_start_block(&self, block: u64) {
        self.start_block
            .store(block, std::sync::atomic::Ordering::SeqCst);
    }

    #[expect(dead_code, reason = "API for direct setting")]
    pub fn set_peak_speed(&self, speed: u64) {
        self.peak_speed
            .store(speed, std::sync::atomic::Ordering::SeqCst);
    }

    /// Update peak speed if the new value is higher.
    pub fn update_peak_speed_max(&self, speed: u64) {
        let _ = self.peak_speed.fetch_update(
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
            |current| Some(current.max(speed)),
        );
    }

    /// Set the timestamp of the last block received (for "Xs ago" display).
    pub fn set_last_block_received_ms(&self, ms: u64) {
        self.last_block_received_ms
            .store(ms, std::sync::atomic::Ordering::SeqCst);
    }

    /// Set whether the RPC server is active (started and serving requests).
    pub fn set_rpc_active(&self, active: bool) {
        self.rpc_active
            .store(active, std::sync::atomic::Ordering::SeqCst);
    }

    /// Increment total RPC request counter.
    pub fn inc_rpc_total(&self) {
        self.rpc_total_requests
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Increment RPC `eth_getLogs` counter.
    pub fn inc_rpc_get_logs(&self) {
        self.rpc_get_logs
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Increment RPC `eth_getBlockByNumber` counter.
    pub fn inc_rpc_get_block(&self) {
        self.rpc_get_block
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Increment RPC error counter.
    pub fn inc_rpc_errors(&self) {
        self.rpc_errors
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Initialize coverage tracking for a block range.
    pub fn init_coverage(&self, start: u64, end: u64, num_buckets: usize) {
        let mut coverage = self.coverage.lock();
        coverage.init(start, end, num_buckets);
    }

    /// Record completed blocks for coverage tracking.
    pub fn record_coverage_completed(&self, blocks: &[u64]) {
        let mut coverage = self.coverage.lock();
        coverage.record_completed_batch(blocks);
    }

    /// Get coverage buckets as percentages (0-100).
    pub fn get_coverage_percentages(&self) -> Vec<u8> {
        let coverage = self.coverage.lock();
        let range = coverage.end_block.saturating_sub(coverage.start_block);
        let n = coverage.buckets.len() as u64;
        if n == 0 {
            return Vec::new();
        }
        coverage
            .buckets
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let lo = i as u64 * range / n;
                let hi = (i as u64 + 1) * range / n;
                let bucket_size = hi - lo;
                if bucket_size == 0 {
                    100
                } else {
                    let pct = (u64::from(count) * 100 / bucket_size).min(100);
                    pct as u8
                }
            })
            .collect()
    }
}
