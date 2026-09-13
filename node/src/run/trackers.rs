//! Head tracking and tail feeding for follow mode.

use crate::chain::ChainTypes;
use crate::p2p::PeerPool;
use crate::storage::Storage;
use crate::sync::historical::TailIngestConfig;
use crate::sync::SyncProgressStats;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

/// Handles returned from spawning the head tracker.
pub struct HeadTrackerHandles {
    pub handle: JoinHandle<()>,
    pub stop_tx: watch::Sender<bool>,
    pub head_seen_rx: watch::Receiver<u64>,
}

/// Handles returned from spawning the tail feeder.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "fields accessed via destructuring")
)]
pub struct TailFeederHandles {
    pub handle: JoinHandle<()>,
    pub stop_tx: watch::Sender<bool>,
    pub ranges_rx: mpsc::UnboundedReceiver<RangeInclusive<u64>>,
}

/// Spawn the head tracker task that monitors peer heads and updates storage/stats.
pub fn spawn_head_tracker<C: ChainTypes>(
    pool: Arc<PeerPool<C>>,
    storage: Arc<Storage>,
    stats: Option<Arc<SyncProgressStats>>,
    initial_head: u64,
    rollback_window: u64,
) -> HeadTrackerHandles {
    let (stop_tx, stop_rx) = watch::channel(false);
    let (head_seen_tx, head_seen_rx) = watch::channel(initial_head);

    let mut stop_rx_head = stop_rx;
    let handle = tokio::spawn(async move {
        let mut last_head = initial_head;
        loop {
            if *stop_rx_head.borrow() {
                break;
            }
            let result = tokio::select! {
                _ = stop_rx_head.changed() => break,
                result = crate::p2p::discover_head_p2p(&pool, last_head, 3, 1024) => result,
            };
            let best_head = match result {
                Ok(Some(head)) => head,
                Ok(None) => last_head,
                Err(err) => {
                    tracing::debug!(error = %err, "head tracker: header probe failed");
                    last_head
                }
            };
            if best_head > last_head {
                last_head = best_head;
                let _ = head_seen_tx.send(best_head);
                if let Err(err) = storage.set_head_seen(best_head) {
                    tracing::debug!(error = %err, "head tracker: failed to persist head_seen");
                }
                if let Some(stats) = stats.as_ref() {
                    stats.set_head_seen(best_head);
                }
            }
            tokio::select! {
                _ = stop_rx_head.changed() => {
                    if *stop_rx_head.borrow() {
                        break;
                    }
                }
                () = tokio::time::sleep(Duration::from_secs(1)) => {}
            }
        }
    });

    // Create a separate receiver for TailIngestConfig
    let _ = rollback_window; // Used by caller for tail feeder

    HeadTrackerHandles {
        handle,
        stop_tx,
        head_seen_rx,
    }
}

/// Spawn the tail feeder task that emits new block ranges as the head advances.
pub fn spawn_tail_feeder(
    initial_end: u64,
    rollback_window: u64,
    head_seen_rx: &watch::Receiver<u64>,
) -> TailFeederHandles {
    let (stop_tx, stop_rx) = watch::channel(false);
    let (tail_tx, ranges_rx) = mpsc::unbounded_channel();

    let mut stop_rx_tail = stop_rx;
    let mut head_seen_rx_tail = head_seen_rx.clone();
    let mut next_to_schedule = initial_end.saturating_add(1);

    let handle = tokio::spawn(async move {
        loop {
            if *stop_rx_tail.borrow() {
                break;
            }
            let head_seen = *head_seen_rx_tail.borrow();
            let safe_head = head_seen.saturating_sub(rollback_window);
            if safe_head >= next_to_schedule {
                let start = next_to_schedule;
                let end = safe_head;
                let _ = tail_tx.send(start..=end);
                next_to_schedule = end.saturating_add(1);
            }
            tokio::select! {
                _ = head_seen_rx_tail.changed() => {}
                _ = stop_rx_tail.changed() => {
                    if *stop_rx_tail.borrow() {
                        break;
                    }
                }
                () = tokio::time::sleep(Duration::from_millis(500)) => {}
            }
        }
    });

    TailFeederHandles {
        handle,
        stop_tx,
        ranges_rx,
    }
}

/// Build TailIngestConfig from tail feeder handles.
pub fn build_tail_config(
    tail_handles: TailFeederHandles,
    head_seen_rx: watch::Receiver<u64>,
    rollback_window: u64,
) -> TailIngestConfig {
    TailIngestConfig {
        ranges_rx: tail_handles.ranges_rx,
        head_seen_rx,
        stop_tx: tail_handles.stop_tx,
        stop_when_caught_up: true,
        head_offset: rollback_window,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::BaseChain;
    use crate::p2p::{peer_pool_for_tests, NetworkPeer};
    use eyre::{ensure, Result};
    use reth_eth_wire_types::{BlockHashOrNumber, BlockHeaders};
    use reth_network_api::{PeerId, PeerRequest, PeerRequestSender};
    use reth_primitives_traits::Header;
    use tokio::time::timeout;

    #[tokio::test]
    async fn base_head_tracker_advances_by_headers_and_cancels_pending_probes() -> Result<()> {
        let root = crate::test_utils::temp_dir("base-head-tracker");
        let mut config = crate::test_utils::base_config(root.clone());
        config.chain = crate::chain::ChainKind::Base;
        config.chain_id = config.chain.chain_id();
        config.genesis_hash = config.chain.genesis_hash();
        let storage = Arc::new(Storage::open(&config)?);
        let peer_id = PeerId::random();
        let (tx, mut rx) = mpsc::channel(8);
        let pool = Arc::new(peer_pool_for_tests(vec![NetworkPeer::<BaseChain> {
            peer_id,
            eth_version: reth_eth_wire::EthVersion::Eth69,
            messages: PeerRequestSender::new(peer_id, tx),
            head_number: 100,
            earliest_block: Some(0),
            latest_block: Some(100),
        }]));
        let mut tracker =
            spawn_head_tracker(Arc::clone(&pool), Arc::clone(&storage), None, 100, 64);
        let Some(PeerRequest::GetBlockHeaders { request, response }) =
            timeout(Duration::from_secs(2), rx.recv()).await?
        else {
            return Err(eyre::eyre!("missing header probe"));
        };
        ensure!(
            request.start_block == BlockHashOrNumber::Number(101),
            "probe did not start above observed head"
        );
        let mut parent_hash = alloy_primitives::B256::ZERO;
        let headers: Vec<_> = (101..=104)
            .map(|number| {
                let header = Header {
                    number,
                    parent_hash,
                    ..Default::default()
                };
                parent_hash = header.hash_slow();
                header
            })
            .collect();
        let _ = response.send(Ok(BlockHeaders(headers)));
        timeout(Duration::from_secs(2), tracker.head_seen_rx.changed()).await??;
        ensure!(
            *tracker.head_seen_rx.borrow() == 104 && pool.get_peer_head(peer_id) == Some(104),
            "head did not follow header probes"
        );
        ensure!(
            storage.head_seen()? == Some(104),
            "observed head not persisted"
        );
        let Some(PeerRequest::GetBlockHeaders { request, response }) =
            timeout(Duration::from_secs(2), rx.recv()).await?
        else {
            return Err(eyre::eyre!("missing second probe"));
        };
        ensure!(
            request.start_block == BlockHashOrNumber::Number(105),
            "tracker repeated the old baseline"
        );
        tracker.stop_tx.send(true)?;
        timeout(Duration::from_millis(250), tracker.handle).await??;
        drop(response);
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[tokio::test]
    async fn tail_feeder_preserves_rollback_depth() -> Result<()> {
        let (head_tx, head_rx) = watch::channel(100);
        let mut feeder = spawn_tail_feeder(36, 64, &head_rx);
        ensure!(
            feeder.ranges_rx.try_recv().is_err(),
            "tail started above delayed head"
        );
        head_tx.send(105)?;
        let range = timeout(Duration::from_secs(1), feeder.ranges_rx.recv()).await?;
        ensure!(range == Some(37..=41), "rollback depth was not preserved");
        feeder.stop_tx.send(true)?;
        timeout(Duration::from_millis(250), feeder.handle).await??;
        Ok(())
    }
}
