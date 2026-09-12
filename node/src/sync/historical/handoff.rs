//! Verify actual successor availability; advertised retention is only a hint.
use crate::{
    chain::ChainTypes,
    p2p::{fetch_payloads_for_peer, PeerPool},
    storage::Storage,
};
use eyre::{ensure, eyre, Result};

/// Reconcile followed blocks before successor validation, including after restart.
pub async fn reconcile_snapshot_tip<C: ChainTypes>(
    storage: &Storage,
    pool: &PeerPool<C>,
    rollback_window: u64,
) -> Result<bool> {
    let Some(boundary) = storage.snapshot_boundary() else {
        return Ok(false);
    };
    let head = storage
        .last_indexed_block()?
        .ok_or_else(|| eyre!("missing snapshot indexed head"))?;
    let check = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        super::reorg::preflight_reorg(storage, pool, head, head.saturating_add(1), 8),
    )
    .await??;
    if let super::reorg::ReorgCheck::ReorgDetected { anchor } = check {
        let low = head.saturating_sub(rollback_window).max(boundary.number);
        let ancestor = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            super::reorg::find_common_ancestor(storage, &anchor, low, head),
        ).await??.ok_or_else(|| eyre!(
            "snapshot/P2P reorg exceeds rollback_window or crosses trusted snapshot boundary {}; refusing to rewrite imported history", boundary.number
        ))?;
        tracing::warn!(
            head,
            ancestor,
            "snapshot continuation: rolling back followed blocks"
        );
        storage.rollback_to(ancestor)?;
        return Ok(true);
    }
    Ok(false)
}

pub async fn preflight_snapshot_handoff<C: ChainTypes>(
    storage: &Storage,
    pool: &PeerPool<C>,
    target: u64,
) -> Result<()> {
    if storage.snapshot_boundary().is_none() {
        return Ok(());
    }
    let head = storage
        .last_indexed_block()?
        .ok_or_else(|| eyre!("missing snapshot indexed head"))?;
    let next = head
        .checked_add(1)
        .ok_or_else(|| eyre!("snapshot head overflow"))?;
    if next > target {
        return Ok(());
    }
    let parent = storage
        .block_header(head)?
        .ok_or_else(|| eyre!("missing boundary header"))?
        .hash_slow();
    let peers = pool.snapshot();
    let candidates: Vec<_> = peers
        .iter()
        .filter(|peer| peer.serving_range(target).contains(&next))
        .take(8)
        .collect();
    let first_available = peers.iter().filter_map(|peer| peer.earliest_block).min();
    ensure!(!candidates.is_empty(),
        "snapshot-to-peer history gap: missing successor {next} (continuation range {next}..={target}); earliest advertised block {first_available:?}. Supply a newer verified snapshot or --trusted-peer serving the missing bodies and receipts");
    let mut mismatch = false;
    for peer in candidates {
        if let Ok(Ok(result)) = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            fetch_payloads_for_peer(peer, next..=next),
        )
        .await
        {
            if let Some(payload) = result
                .payloads
                .iter()
                .find(|payload| payload.header.number == next)
            {
                if payload.header.parent_hash == parent {
                    return Ok(());
                }
                mismatch = true;
            }
        }
    }
    ensure!(!mismatch, "snapshot/P2P boundary mismatch at {next}; peer history does not extend the indexed chain, refusing to rewrite trusted history");
    Err(eyre!("snapshot-to-peer history gap: block {next} bodies/receipts unavailable from probed peers (requested through {target}); supply a newer verified snapshot or --trusted-peer serving this range"))
}
