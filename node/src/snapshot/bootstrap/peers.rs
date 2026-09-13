use crate::{
    chain::BaseChain,
    cli::NodeConfig,
    p2p::{self, NetworkPeer},
    ui::archive::Progress,
};
use alloy_primitives::B256;
use eyre::{ensure, eyre, Result};
use std::{
    future::Future,
    time::{Duration, Instant},
};

pub(super) struct Window {
    pub first: u64,
    pub end: u64,
    pub parent: B256,
}

pub(super) async fn discover(
    config: &NodeConfig,
    start: u64,
    snapshot_end: u64,
    overlap: u64,
    progress: &Progress,
) -> Result<Vec<Window>> {
    ensure!(
        snapshot_end >= start,
        "snapshot is older than the requested start"
    );
    let session = p2p::start_session::<BaseChain>(None, 0, &config.trusted_peers).await?;
    let result = tokio::time::timeout(Duration::from_secs(240), async {
        let warmup = Instant::now();
        while session.pool.len() < 8 && warmup.elapsed() < Duration::from_secs(20) {
            progress.check()?;
            progress.activity(&format!("{} compatible peers", session.pool.len()));
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        select_windows(
            session.pool.snapshot(),
            start,
            snapshot_end,
            overlap,
            progress,
        )
        .await
    })
    .await
    .map_err(|_| eyre!("peer history discovery timed out"));
    // Explicitly stop the discovery network before normal history startup binds its port.
    let shutdown = tokio::time::timeout(Duration::from_secs(5), session.handle.shutdown()).await;
    let windows = result??;
    shutdown.map_err(|_| eyre!("discovery network shutdown timed out"))??;
    Ok(windows)
}

async fn select_windows(
    mut peers: Vec<NetworkPeer<BaseChain>>,
    start: u64,
    snapshot_end: u64,
    overlap: u64,
    progress: &Progress,
) -> Result<Vec<Window>> {
    peers.sort_by_key(|p| p.earliest_block.unwrap_or(0));
    let mut windows = Vec::new();
    for peer in peers.iter().take(8) {
        progress.check()?;
        let range = peer.serving_range(snapshot_end.saturating_add(1));
        let low = start.max(*range.start());
        let high = *range.end();
        let first = first_available(low, high, |number| async move {
            Ok(probe(peer, number, progress).await?.is_some())
        })
        .await?;
        let Some(first) = first else {
            continue;
        };
        let end = first
            .saturating_add(overlap)
            .min(snapshot_end)
            .min(high.saturating_sub(1));
        if end < start || end.saturating_add(1) < first {
            continue;
        }
        if let Some(parent) = probe(peer, end + 1, progress).await? {
            windows.push(Window { first, end, parent });
        }
    }
    windows.sort_by_key(|w| w.end);
    ensure!(!windows.is_empty(), "snapshot-to-peer history gap: no probed peer serves a successor within this snapshot; supply a newer pinned manifest or a serving --trusted-peer");
    Ok(windows)
}

async fn probe(
    peer: &NetworkPeer<BaseChain>,
    number: u64,
    progress: &Progress,
) -> Result<Option<B256>> {
    progress.check()?;
    progress.activity(&format!("Probing block {number} bodies and receipts"));
    let request = p2p::fetch_payloads_for_peer(peer, number..=number);
    let response = tokio::select! {
        response = tokio::time::timeout(Duration::from_secs(6), request) => response,
        () = cancelled(progress) => return Err(eyre!("archive job cancelled")),
    };
    progress.advance(1);
    Ok(response.ok().and_then(Result::ok).and_then(|batch| {
        batch
            .payloads
            .into_iter()
            .find(|p| p.header.number == number)
            .map(|p| p.header.parent_hash)
    }))
}

async fn cancelled(progress: &Progress) {
    while progress.check().is_ok() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// Pruned ranges are usually contiguous. Every selected successor is separately
// fetched and validated; a hole or transient error can only select a later boundary.
async fn first_available<F, Fut>(mut low: u64, mut high: u64, probe: F) -> Result<Option<u64>>
where
    F: Fn(u64) -> Fut,
    Fut: Future<Output = Result<bool>>,
{
    if low > high || !probe(high).await? {
        return Ok(None);
    }
    if probe(low).await? {
        return Ok(Some(low));
    }
    while low < high {
        let middle = low + (high - low) / 2;
        if probe(middle).await? {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    Ok(Some(low))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn probes_pruned_peer_bodies_and_uses_snapshot_capped_overlap() -> Result<()> {
        let fixture = crate::snapshot::tests::fixture(false)?;
        let payloads = crate::snapshot::handoff_tests::successors(&fixture.payloads[3]);
        let (pool, server) =
            crate::snapshot::handoff_tests::serving_peer_at(payloads.clone(), None, 5, true);
        let windows = select_windows(pool.snapshot(), 0, 4, 43_200, &Progress::default()).await?;
        ensure!(
            windows.len() == 1
                && windows[0].first == 4
                && windows[0].end == 4
                && windows[0].parent == payloads[1].header.parent_hash,
            "incorrect overlap boundary"
        );
        server.abort();
        let (pool, server) =
            crate::snapshot::handoff_tests::serving_peer_at(payloads, Some(4), 5, false);
        ensure!(
            select_windows(pool.snapshot(), 0, 4, 1, &Progress::default())
                .await
                .is_err(),
            "headers-only peer accepted"
        );
        server.abort();
        Ok(())
    }
    #[tokio::test]
    async fn availability_search_handles_empty_complete_and_pruned_ranges() -> Result<()> {
        for (low, high, first, expected) in [
            (0, 100, 31, Some(31)),
            (10, 10, 10, Some(10)),
            (10, 9, 0, None),
            (0, 10, 11, None),
            (3, 10, 0, Some(3)),
        ] {
            ensure!(
                first_available(low, high, |n| async move { Ok(n >= first) }).await? == expected,
                "bad availability boundary"
            );
        }
        ensure!(
            first_available(0, 10, |_| async { Err(eyre!("cancelled")) })
                .await
                .is_err(),
            "probe error swallowed"
        );
        Ok(())
    }
}
