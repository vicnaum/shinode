//! Live follow loop for continuous syncing.

use crate::cli::NodeConfig;
use crate::p2p::{discover_head_p2p, PeerPool};
use crate::storage::Storage;
use crate::sync::{ProgressReporter, SyncProgressStats, SyncStatus};
use eyre::{eyre, Result};
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};

use super::reorg::{find_common_ancestor, preflight_reorg, ReorgCheck};
use super::{run_ingest_pipeline, IngestPipelineOutcome, PeerHealthTracker};

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

const FOLLOW_POLL_MS: u64 = 1000;
const FOLLOW_NEAR_TIP_BACKOFF_MS: u64 = 500;
const FOLLOW_NEAR_TIP_BLOCKS: u64 = 2;
const HEAD_PROBE_PEERS: usize = 3;
const HEAD_PROBE_LIMIT: usize = 1024;
const REORG_PROBE_PEERS: usize = 3;

async fn dump_follow_debug(
    storage: &Storage,
    pool: &PeerPool,
    stats: Option<&SyncProgressStats>,
    peer_health: &super::scheduler::PeerHealthTracker,
) {
    let last_indexed = storage.last_indexed_block().ok().flatten();
    let head_seen = storage.head_seen().ok().flatten();
    let peers = pool.snapshot();
    let peers_len = peers.len();
    let snapshot = stats.map(|s| s.snapshot());
    let health = peer_health.snapshot().await;

    tracing::info!(
        last_indexed = ?last_indexed,
        head_seen = ?head_seen,
        peers = peers_len,
        progress = ?snapshot,
        "debug dump (SIGUSR1)"
    );

    tracing::info!("peers:");
    for peer in &peers {
        tracing::info!(peer_id = ?peer.peer_id, head_number = peer.head_number, "peer");
    }

    tracing::info!("peer health (best quality first):");
    for entry in health.into_iter().take(50) {
        tracing::info!(
            peer_id = ?entry.peer_id,
            banned = entry.is_banned,
            ban_remaining_ms = entry.ban_remaining_ms,
            inflight_blocks = entry.inflight_blocks,
            last_assigned_age_ms = entry.last_assigned_age_ms,
            quality = entry.quality_score,
            samples = entry.quality_samples,
            succ = entry.successes,
            fail = entry.failures,
            partial = entry.partials,
            cons_fail = entry.consecutive_failures,
            cons_partial = entry.consecutive_partials,
            last_err = entry.last_error,
            last_err_age_ms = entry.last_error_age_ms,
            last_err_count = entry.last_error_count,
            "health"
        );
    }
}

pub async fn run_follow_loop(
    storage: Arc<Storage>,
    pool: Arc<PeerPool>,
    config: &NodeConfig,
    progress: Option<Arc<dyn ProgressReporter>>,
    stats: Option<Arc<SyncProgressStats>>,
    peer_health: Arc<PeerHealthTracker>,
) -> Result<()> {
    let poll = Duration::from_millis(FOLLOW_POLL_MS);

    #[cfg(unix)]
    {
        let storage = Arc::clone(&storage);
        let pool = Arc::clone(&pool);
        let stats = stats.clone();
        let peer_health = Arc::clone(&peer_health);
        tokio::spawn(async move {
            let mut sigusr1 = match signal(SignalKind::user_defined1()) {
                Ok(sig) => sig,
                Err(err) => {
                    tracing::debug!(error = %err, "failed to install SIGUSR1 handler");
                    return;
                }
            };
            while sigusr1.recv().await.is_some() {
                dump_follow_debug(
                    storage.as_ref(),
                    pool.as_ref(),
                    stats.as_deref(),
                    peer_health.as_ref(),
                )
                .await;
            }
        });
    }

    loop {
        let last_indexed = storage.last_indexed_block()?;
        let baseline = last_indexed
            .unwrap_or(config.start_block)
            .max(config.start_block);

        let observed_head =
            match discover_head_p2p(&pool, baseline, HEAD_PROBE_PEERS, HEAD_PROBE_LIMIT).await? {
                Some(head) => head,
                None => {
                    if let Some(stats) = stats.as_ref() {
                        stats.set_status(SyncStatus::LookingForPeers);
                    }
                    sleep(poll).await;
                    continue;
                }
            };

        storage.set_head_seen(observed_head)?;
        if let Some(stats) = stats.as_ref() {
            stats.set_head_seen(observed_head);
        }

        let start = last_indexed
            .map(|block| block.saturating_add(1))
            .unwrap_or(config.start_block)
            .max(config.start_block);
        let mut target_head = observed_head;
        if let Some(end) = config.end_block {
            target_head = target_head.min(end);
        }

        if start > target_head {
            if let Some(stats) = stats.as_ref() {
                stats.set_status(SyncStatus::Following);
                stats.set_queue(0);
                stats.set_inflight(0);
                if let Some(last) = last_indexed {
                    stats.set_head_block(last);
                }
                stats.set_head_seen(observed_head);
            }
            sleep(poll).await;
            continue;
        }

        if target_head.saturating_sub(start) <= FOLLOW_NEAR_TIP_BLOCKS {
            sleep(Duration::from_millis(FOLLOW_NEAR_TIP_BACKOFF_MS)).await;
        }

        if let Some(last_indexed) = last_indexed {
            match preflight_reorg(
                storage.as_ref(),
                pool.as_ref(),
                last_indexed,
                start,
                REORG_PROBE_PEERS,
            )
            .await?
            {
                ReorgCheck::NoReorg => {}
                ReorgCheck::Inconclusive => {
                    tracing::debug!(last_indexed, start, "reorg check inconclusive; retrying");
                    sleep(poll).await;
                    continue;
                }
                ReorgCheck::ReorgDetected { anchor } => {
                    let low = last_indexed
                        .saturating_sub(config.rollback_window)
                        .max(config.start_block);
                    tracing::warn!(
                        last_indexed,
                        start,
                        low,
                        peer_id = ?anchor.peer_id,
                        "reorg detected; searching for common ancestor"
                    );
                    let ancestor =
                        find_common_ancestor(storage.as_ref(), &anchor, low, last_indexed).await?;
                    if let Some(ancestor) = ancestor {
                        tracing::warn!(last_indexed, ancestor, "reorg rollback to ancestor");
                        storage.rollback_to(ancestor)?;
                        continue;
                    }
                    tracing::error!(
                        last_indexed,
                        low,
                        "reorg exceeds rollback_window; stopping follow"
                    );
                    return Err(eyre!(
                        "reorg exceeds rollback_window: last_indexed={}, low={}",
                        last_indexed,
                        low
                    ));
                }
            }
        }

        let progress_ref = progress
            .as_ref()
            .map(|tracker| Arc::clone(tracker) as Arc<dyn ProgressReporter>);
        let ingest_started = Instant::now();
        let range = start..=target_head;
        let blocks = storage.missing_blocks_in_range(range.clone())?;
        let outcome = run_ingest_pipeline(
            Arc::clone(&storage),
            Arc::clone(&pool),
            config,
            range,
            blocks,
            observed_head,
            progress_ref,
            stats.clone(),
            None,
            Some(target_head),
            Arc::clone(&peer_health),
            None,
            None,
        )
        .await?;
        let elapsed = ingest_started.elapsed();

        match outcome {
            IngestPipelineOutcome::UpToDate { head } => {
                if let Some(s) = stats.as_ref() {
                    s.set_head_block(head);
                }
                tracing::info!(head, elapsed_ms = elapsed.as_millis(), "ingest up to date");
            }
            IngestPipelineOutcome::RangeApplied { range, logs } => {
                if let Some(s) = stats.as_ref() {
                    s.set_head_block(*range.end());
                }
                tracing::debug!(
                    range_start = *range.start(),
                    range_end = *range.end(),
                    logs,
                    elapsed_ms = elapsed.as_millis(),
                    "ingested range"
                );
            }
        }
    }
}
