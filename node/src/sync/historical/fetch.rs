//! Fetch logic for historical probe mode.

use crate::p2p::{
    fetch_payloads_for_peer, request_headers_batch, request_receipt_counts, NetworkPeer,
    PayloadFetchOutcome,
};
use crate::sync::{BlockPayload};
use crate::sync::historical::types::{FetchTiming, ProbeRecord};
use eyre::{eyre, Result};
use reth_primitives_traits::SealedHeader;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug)]
pub struct FetchProbeOutcome {
    pub records: Vec<ProbeRecord>,
    pub missing_blocks: Vec<u64>,
}

#[derive(Debug)]
pub struct FetchIngestOutcome {
    pub payloads: Vec<BlockPayload>,
    pub missing_blocks: Vec<u64>,
    pub fetch_stats: crate::p2p::FetchStageStats,
}

/// Fetch headers + receipts for a consecutive batch of blocks.
pub async fn fetch_probe_batch(
    peer: &NetworkPeer,
    blocks: &[u64],
    receipts_per_request: usize,
) -> Result<FetchProbeOutcome> {
    if blocks.is_empty() {
        return Ok(FetchProbeOutcome {
            records: Vec::new(),
            missing_blocks: Vec::new(),
        });
    }
    ensure_consecutive(blocks)?;

    let start_block = blocks[0];
    let count = blocks.len();

    let headers_start = Instant::now();
    let headers = request_headers_batch(peer, start_block, count).await?;
    let headers_ms = headers_start.elapsed().as_millis() as u64;

    let mut headers_by_number = HashMap::new();
    for header in headers {
        headers_by_number.insert(header.number, header);
    }

    let mut hashes = Vec::with_capacity(count);
    let mut missing_blocks = Vec::new();
    for block in blocks {
        if let Some(header) = headers_by_number.remove(block) {
            let hash = SealedHeader::seal_slow(header).hash();
            hashes.push((*block, hash));
        } else {
            missing_blocks.push(*block);
        }
    }

    let mut records = Vec::with_capacity(hashes.len());
    let chunk_size = receipts_per_request.max(1);
    for chunk in hashes.chunks(chunk_size) {
        let chunk_hashes: Vec<_> = chunk.iter().map(|(_, hash)| *hash).collect();
        let receipts_start = Instant::now();
        let counts = request_receipt_counts(peer, &chunk_hashes).await?;
        let receipts_ms = receipts_start.elapsed().as_millis() as u64;
        let total_ms = headers_start.elapsed().as_millis() as u64;

        let timing = FetchTiming {
            headers_ms,
            receipts_ms,
            total_ms,
        };

        for (idx, (block_number, _)) in chunk.iter().enumerate() {
            if let Some(count) = counts.get(idx).copied() {
                records.push(ProbeRecord {
                    number: *block_number,
                    peer_id: peer.peer_id,
                    receipts: count as u64,
                    timing,
                });
            } else {
                missing_blocks.push(*block_number);
            }
        }
    }

    Ok(FetchProbeOutcome {
        records,
        missing_blocks,
    })
}

/// Fetch full block payloads for ingest mode.
pub async fn fetch_ingest_batch(
    peer: &NetworkPeer,
    blocks: &[u64],
) -> Result<FetchIngestOutcome> {
    if blocks.is_empty() {
        return Ok(FetchIngestOutcome {
            payloads: Vec::new(),
            missing_blocks: Vec::new(),
            fetch_stats: crate::p2p::FetchStageStats::default(),
        });
    }
    ensure_consecutive(blocks)?;
    let start = blocks[0];
    let end = blocks[blocks.len() - 1];
    let PayloadFetchOutcome {
        payloads,
        missing_blocks,
        fetch_stats,
    } = fetch_payloads_for_peer(peer, start..=end).await?;
    Ok(FetchIngestOutcome {
        payloads,
        missing_blocks,
        fetch_stats,
    })
}

fn ensure_consecutive(blocks: &[u64]) -> Result<()> {
    for idx in 1..blocks.len() {
        if blocks[idx] != blocks[idx - 1].saturating_add(1) {
            return Err(eyre!("block batch is not consecutive"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_consecutive;

    #[test]
    fn ensure_consecutive_rejects_gaps() {
        assert!(ensure_consecutive(&[1, 2, 4]).is_err());
        assert!(ensure_consecutive(&[10, 11, 12]).is_ok());
    }

    // Partial receipts handling is exercised in integration tests via the probe pipeline.
}
