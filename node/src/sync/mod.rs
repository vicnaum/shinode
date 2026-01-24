//! Sync and ingest orchestration.

use async_trait::async_trait;
use eyre::Result;
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

/// Source of block payloads for ingestion.
#[async_trait]
pub trait BlockPayloadSource: Send + Sync {
    async fn head(&self) -> Result<u64>;
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

impl SyncStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SyncStatus::LookingForPeers => "looking_for_peers",
            SyncStatus::Fetching => "fetching",
            SyncStatus::Finalizing => "finalizing",
            SyncStatus::UpToDate => "up_to_date",
            SyncStatus::Following => "following",
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

#[derive(Debug, Default)]
pub struct SyncProgressStats {
    processed: std::sync::atomic::AtomicU64,
    failed: std::sync::atomic::AtomicU64,
    queue: std::sync::atomic::AtomicU64,
    inflight: std::sync::atomic::AtomicU64,
    compactions_done: std::sync::atomic::AtomicU64,
    compactions_total: std::sync::atomic::AtomicU64,
    peers_active: std::sync::atomic::AtomicU64,
    peers_total: std::sync::atomic::AtomicU64,
    status: std::sync::atomic::AtomicU8,
    head_block: std::sync::atomic::AtomicU64,
    head_seen: std::sync::atomic::AtomicU64,
}

#[derive(Debug, Clone, Copy)]
pub struct SyncProgressSnapshot {
    pub processed: u64,
    pub failed: u64,
    pub queue: u64,
    pub inflight: u64,
    pub compactions_done: u64,
    pub compactions_total: u64,
    pub peers_active: u64,
    pub peers_total: u64,
    pub status: SyncStatus,
    pub head_block: u64,
    pub head_seen: u64,
}

impl SyncProgressStats {
    pub fn snapshot(&self) -> SyncProgressSnapshot {
        SyncProgressSnapshot {
            processed: self.processed.load(std::sync::atomic::Ordering::SeqCst),
            failed: self.failed.load(std::sync::atomic::Ordering::SeqCst),
            queue: self.queue.load(std::sync::atomic::Ordering::SeqCst),
            inflight: self.inflight.load(std::sync::atomic::Ordering::SeqCst),
            compactions_done: self
                .compactions_done
                .load(std::sync::atomic::Ordering::SeqCst),
            compactions_total: self
                .compactions_total
                .load(std::sync::atomic::Ordering::SeqCst),
            peers_active: self.peers_active.load(std::sync::atomic::Ordering::SeqCst),
            peers_total: self.peers_total.load(std::sync::atomic::Ordering::SeqCst),
            status: match self.status.load(std::sync::atomic::Ordering::SeqCst) {
                1 => SyncStatus::Fetching,
                2 => SyncStatus::Finalizing,
                3 => SyncStatus::UpToDate,
                4 => SyncStatus::Following,
                _ => SyncStatus::LookingForPeers,
            },
            head_block: self.head_block.load(std::sync::atomic::Ordering::SeqCst),
            head_seen: self.head_seen.load(std::sync::atomic::Ordering::SeqCst),
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

    pub fn inc_failed(&self, delta: u64) {
        self.failed
            .fetch_add(delta, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn record_block_recovered(&self, count: u64) {
        if count == 0 {
            return;
        }
        let _ = self.failed.fetch_update(
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
            |current| Some(current.saturating_sub(count)),
        );
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

    pub fn set_peers_active(&self, active: u64) {
        self.peers_active
            .store(active, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_peers_total(&self, total: u64) {
        self.peers_total
            .store(total, std::sync::atomic::Ordering::SeqCst);
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
}
