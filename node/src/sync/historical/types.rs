//! Shared types for historical sync.

use alloy_primitives::B256;
use reth_ethereum_primitives::Receipt;
use reth_eth_wire::EthVersion;
use reth_network_api::PeerId;
use reth_primitives_traits::Header;
use std::time::Duration;

/// Mode of the historical pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingMode {
    /// Harness-like probe mode (headers + receipts only, no DB).
    Probe,
    /// Full ingest mode (future).
    Ingest,
}

/// Fetch scheduling mode for a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMode {
    /// Normal batch selection.
    Normal,
    /// Escalation batch selection (retry across peers).
    Escalation,
}

/// A batch of blocks assigned to a peer.
#[derive(Debug, Clone)]
pub struct FetchBatch {
    pub blocks: Vec<u64>,
    pub mode: FetchMode,
}

/// Configuration for benchmark/probe runs.
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub blocks_per_assignment: usize,
    pub receipts_per_request: usize,
    pub report_interval: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            blocks_per_assignment: 32,
            receipts_per_request: 16,
            report_interval: Duration::from_millis(100),
        }
    }
}

/// Fetch timing metadata for a batch or block.
#[derive(Debug, Clone, Copy, Default)]
pub struct FetchTiming {
    pub headers_ms: u64,
    pub receipts_ms: u64,
    pub total_ms: u64,
}

/// Raw fetched data before processing.
#[derive(Debug, Clone)]
pub struct FetchedBlock {
    pub number: u64,
    pub peer_id: PeerId,
    pub eth_version: EthVersion,
    pub header: Header,
    pub header_hash: B256,
    pub receipts: Vec<Receipt>,
    pub timing: FetchTiming,
}

/// Minimal processed record for probe mode.
#[derive(Debug, Clone)]
pub struct ProbeRecord {
    pub number: u64,
    pub peer_id: PeerId,
    pub receipts: u64,
    pub timing: FetchTiming,
}
