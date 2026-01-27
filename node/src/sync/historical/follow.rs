//! Live follow loop for continuous syncing.

use crate::cli::NodeConfig;
use crate::p2p::{discover_head_p2p, PeerPool};
use crate::storage::Storage;
use crate::sync::{ProgressReporter, SyncProgressStats, SyncStatus};
use eyre::{eyre, Result};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, Duration};

use super::db_writer::DbWriteMode;
use super::reorg::{find_common_ancestor, preflight_reorg, ReorgCheck};
use super::{run_ingest_pipeline, IngestPipelineOutcome, PeerHealthTracker, TailIngestConfig};

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
    let snapshot = stats.map(SyncProgressStats::snapshot);
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
    mut synced_once_tx: Option<tokio::sync::oneshot::Sender<()>>,
) -> Result<()> {
    let poll = Duration::from_millis(FOLLOW_POLL_MS);
    let mut synced_once = false;

    // Notify the caller (main) when we become synced for the first time.
    //
    // `run_ingest_pipeline()` can run indefinitely in follow mode, so we can't rely on the
    // outer loop's "start > head" check to trigger this signal.
    let synced_sender = Arc::new(tokio::sync::Mutex::new(synced_once_tx.take()));
    if synced_sender.lock().await.is_some() {
        let synced_sender = Arc::clone(&synced_sender);
        let storage = Arc::clone(&storage);
        let stats = stats.clone();
        tokio::spawn(async move {
            loop {
                let synced = if let Some(stats) = stats.as_ref() {
                    matches!(
                        stats.snapshot().status,
                        SyncStatus::UpToDate | SyncStatus::Following
                    )
                } else {
                    match (
                        storage.last_indexed_block().ok().flatten(),
                        storage.head_seen().ok().flatten(),
                    ) {
                        (Some(last), Some(head)) => last >= head,
                        _ => false,
                    }
                };
                if synced {
                    if let Some(tx) = synced_sender.lock().await.take() {
                        let _ = tx.send(());
                    }
                    break;
                }
                sleep(Duration::from_millis(200)).await;
            }
        });
    }

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

        let Some(observed_head) =
            discover_head_p2p(&pool, baseline, HEAD_PROBE_PEERS, HEAD_PROBE_LIMIT).await?
        else {
            if let Some(stats) = stats.as_ref() {
                stats.set_status(SyncStatus::LookingForPeers);
            }
            sleep(poll).await;
            continue;
        };

        storage.set_head_seen(observed_head)?;
        if let Some(stats) = stats.as_ref() {
            stats.set_head_seen(observed_head);
        }

        let start = last_indexed
            .map_or(config.start_block, |block| block.saturating_add(1))
            .max(config.start_block);
        let mut target_head = observed_head;
        if let Some(end) = config.end_block {
            target_head = target_head.min(end);
        }

        if start > target_head {
            if let Some(stats) = stats.as_ref() {
                stats.set_status(SyncStatus::UpToDate);
                stats.set_queue(0);
                stats.set_inflight(0);
                if let Some(last) = last_indexed {
                    stats.set_head_block(last);
                }
                stats.set_head_seen(observed_head);
            }
            if !synced_once {
                synced_once = true;
                tracing::info!(head = observed_head, "follow: synced (up to date)");
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
        tracing::info!(
            range_start = *range.start(),
            range_end = *range.end(),
            missing_blocks = blocks.len(),
            last_indexed = ?last_indexed,
            observed_head,
            "follow: ingest epoch"
        );

        let (head_seen_tx, head_seen_rx) = watch::channel(observed_head);
        let (tail_tx, tail_rx) = mpsc::unbounded_channel();
        let (stop_tx, stop_rx) = watch::channel(false);
        let stop_tx_reorg = stop_tx.clone();
        let mut stop_rx_head = stop_rx.clone();
        let mut stop_rx_tail = stop_rx.clone();
        let mut stop_rx_reorg = stop_rx.clone();

        let storage_head = Arc::clone(&storage);
        let pool_head = Arc::clone(&pool);
        let stats_head = stats.clone();
        let start_block = config.start_block;
        let end_block = config.end_block;
        let head_task = tokio::spawn(async move {
            let mut last_head = observed_head;
            loop {
                if *stop_rx_head.borrow() {
                    break;
                }
                let baseline = storage_head
                    .last_indexed_block()
                    .ok()
                    .flatten()
                    .unwrap_or(start_block)
                    .max(start_block);
                let head = match discover_head_p2p(
                    &pool_head,
                    baseline,
                    HEAD_PROBE_PEERS,
                    HEAD_PROBE_LIMIT,
                )
                .await
                {
                    Ok(Some(head)) => head,
                    Ok(None) => {
                        sleep(poll).await;
                        continue;
                    }
                    Err(err) => {
                        tracing::debug!(error = %err, "follow head probe failed");
                        sleep(poll).await;
                        continue;
                    }
                };
                let capped = end_block.map_or(head, |end| head.min(end));
                if capped > last_head {
                    last_head = capped;
                    let _ = head_seen_tx.send(capped);
                    if let Err(err) = storage_head.set_head_seen(capped) {
                        tracing::debug!(error = %err, "follow head tracker: failed to persist head_seen");
                    }
                    if let Some(stats) = stats_head.as_ref() {
                        stats.set_head_seen(capped);
                    }
                }
                tokio::select! {
                    _ = stop_rx_head.changed() => {
                        if *stop_rx_head.borrow() {
                            break;
                        }
                    }
                    () = sleep(poll) => {}
                }
            }
        });

        let mut head_seen_rx_tail = head_seen_rx.clone();
        let mut next_to_schedule = target_head.saturating_add(1);
        let tail_task = tokio::spawn(async move {
            loop {
                if *stop_rx_tail.borrow() {
                    break;
                }
                let head_seen = *head_seen_rx_tail.borrow();
                if head_seen >= next_to_schedule {
                    let _ = tail_tx.send(next_to_schedule..=head_seen);
                    next_to_schedule = head_seen.saturating_add(1);
                }
                tokio::select! {
                    _ = head_seen_rx_tail.changed() => {}
                    _ = stop_rx_tail.changed() => {
                        if *stop_rx_tail.borrow() {
                            break;
                        }
                    }
                    () = sleep(Duration::from_millis(200)) => {}
                }
            }
        });

        let (reorg_tx, mut reorg_rx) = mpsc::unbounded_channel();
        let storage_reorg = Arc::clone(&storage);
        let pool_reorg = Arc::clone(&pool);
        let rollback_window = config.rollback_window;
        let start_block = config.start_block;
        let reorg_task = tokio::spawn(async move {
            loop {
                if *stop_rx_reorg.borrow() {
                    break;
                }
                let Ok(Some(last_indexed)) = storage_reorg.last_indexed_block() else {
                    sleep(poll).await;
                    continue;
                };
                let start = last_indexed.saturating_add(1).max(start_block);
                match preflight_reorg(
                    storage_reorg.as_ref(),
                    pool_reorg.as_ref(),
                    last_indexed,
                    start,
                    REORG_PROBE_PEERS,
                )
                .await
                {
                    Ok(ReorgCheck::NoReorg) => {}
                    Ok(ReorgCheck::Inconclusive) => {
                        sleep(poll).await;
                        continue;
                    }
                    Ok(ReorgCheck::ReorgDetected { anchor }) => {
                        let low = last_indexed
                            .saturating_sub(rollback_window)
                            .max(start_block);
                        tracing::warn!(
                            last_indexed,
                            low,
                            peer_id = ?anchor.peer_id,
                            "reorg detected during follow; searching for common ancestor"
                        );
                        let ancestor = find_common_ancestor(
                            storage_reorg.as_ref(),
                            &anchor,
                            low,
                            last_indexed,
                        )
                        .await
                        .ok()
                        .flatten();
                        if let Some(ancestor) = ancestor {
                            let _ = reorg_tx.send(Ok(ancestor));
                        } else {
                            let _ = reorg_tx.send(Err(eyre!(
                                "reorg exceeds rollback_window: last_indexed={}, low={}",
                                last_indexed,
                                low
                            )));
                        }
                        let _ = stop_tx_reorg.send(true);
                        break;
                    }
                    Err(err) => {
                        tracing::debug!(error = %err, "reorg check failed");
                    }
                }
                tokio::select! {
                    _ = stop_rx_reorg.changed() => {
                        if *stop_rx_reorg.borrow() {
                            break;
                        }
                    }
                    () = sleep(poll) => {}
                }
            }
        });

        let tail = TailIngestConfig {
            ranges_rx: tail_rx,
            head_seen_rx,
            stop_tx: stop_tx.clone(),
            stop_when_caught_up: false,
            head_offset: 0,
        };

        let outcome = run_ingest_pipeline(
            Arc::clone(&storage),
            Arc::clone(&pool),
            config,
            range,
            super::MissingBlocks::Precomputed(blocks),
            observed_head,
            progress_ref,
            stats.clone(),
            None,
            Some(DbWriteMode::Follow),
            None,
            Arc::clone(&peer_health),
            None,
            Some(stop_rx),
            Some(tail),
        )
        .await?;
        let _ = stop_tx.send(true);
        let _ = head_task.await;
        let _ = tail_task.await;
        let _ = reorg_task.await;
        let elapsed = ingest_started.elapsed();

        if let Ok(result) = reorg_rx.try_recv() {
            match result {
                Ok(ancestor) => {
                    tracing::warn!(ancestor, "reorg rollback during follow");
                    storage.rollback_to(ancestor)?;
                    continue;
                }
                Err(err) => return Err(err),
            }
        }

        match outcome {
            IngestPipelineOutcome::UpToDate { head } => {
                if let Some(s) = stats.as_ref() {
                    s.set_head_block(head);
                }
                tracing::info!(head, elapsed_ms = elapsed.as_millis(), "ingest up to date");
            }
            IngestPipelineOutcome::RangeApplied { range, logs, .. } => {
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
