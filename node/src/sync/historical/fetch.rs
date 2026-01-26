//! Fetch logic for historical sync.

use crate::p2p::{fetch_payloads_for_peer, NetworkPeer, PayloadFetchOutcome};
use crate::sync::BlockPayload;
use eyre::{eyre, Result};

#[derive(Debug)]
pub struct FetchIngestOutcome {
    pub payloads: Vec<BlockPayload>,
    pub missing_blocks: Vec<u64>,
    pub fetch_stats: crate::p2p::FetchStageStats,
}

/// Fetch full block payloads for ingest mode.
pub async fn fetch_ingest_batch(peer: &NetworkPeer, blocks: &[u64]) -> Result<FetchIngestOutcome> {
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
}
