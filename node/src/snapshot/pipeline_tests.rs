//! Regression coverage for anchored backfill, recovered WAL and cancelled follow epochs.
use super::{
    handoff_tests::{serving_peer_at, successors},
    tests::{config, fixture},
    *,
};
use crate::{
    p2p::PeerPool,
    sync::{
        historical::{
            self, IngestPipelineConfig, IngestPipelineOutcome, IngestPipelineTrackers,
            MissingBlocks, TailIngestConfig,
        },
        ProgressReporter,
    },
};
use tokio::sync::{mpsc, watch};

async fn ingest(
    storage: Arc<Storage>,
    pool: Arc<PeerPool<BaseChain>>,
    options: IngestPipelineConfig<'_>,
    missing: Vec<u64>,
    progress: Option<Arc<dyn ProgressReporter>>,
) -> Result<IngestPipelineOutcome> {
    let health = historical::build_peer_health_tracker(options.config);
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        historical::run_ingest_pipeline(
            storage,
            pool,
            options,
            MissingBlocks::Precomputed(missing),
            IngestPipelineTrackers {
                progress,
                stats: None,
                bench: None,
                events: None,
            },
            health,
        ),
    )
    .await?
}

#[tokio::test]
async fn anchored_backfill_uses_historical_and_recent_peers_together() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    let mut config = config(&fixture.args)?;
    config.end_block = None;
    config.rollback_window = 195;
    run(config.clone(), fixture.args.clone()).await?;
    let storage = Arc::new(Storage::open(&config)?);
    let payloads = successors(&fixture.payloads[3]);
    let (archive, archive_server) = serving_peer_at(vec![payloads[0].clone()], Some(4), 4, true);
    let (recent, recent_server) = serving_peer_at(vec![payloads[1].clone()], Some(5), 200, true);
    let pool = Arc::new(crate::p2p::peer_pool_for_tests(
        archive
            .snapshot()
            .into_iter()
            .chain(recent.snapshot())
            .collect(),
    ));
    historical::preflight_snapshot_handoff(&storage, &pool, 5).await?;
    let (_ranges_tx, ranges_rx) = mpsc::unbounded_channel();
    let (_head_tx, head_seen_rx) = watch::channel(200);
    let (stop_tx, _stop_rx) = watch::channel(false);
    let result = ingest(
        Arc::clone(&storage),
        pool,
        IngestPipelineConfig {
            config: &config,
            range: 4..=5,
            head_at_startup: 200,
            db_mode_override: None,
            head_cap_override: None,
            stop_rx: None,
            tail: Some(TailIngestConfig {
                ranges_rx,
                head_seen_rx,
                stop_tx,
                stop_when_caught_up: true,
                head_offset: 195,
            }),
        },
        vec![4, 5],
        None,
    )
    .await;
    archive_server.abort();
    recent_server.abort();
    ensure!(
        matches!(result?, IngestPipelineOutcome::RangeApplied { .. }),
        "backfill did not complete"
    );
    ensure!(
        storage.last_indexed_block()? == Some(5),
        "historical peer was not used"
    );
    storage.validate_snapshot_boundary()?;
    Ok(())
}

#[tokio::test]
async fn anchored_startup_compacts_pending_incomplete_shards_before_scheduling() -> Result<()> {
    for defer in [false, true] {
        let mut fixture = fixture(false)?;
        fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
        let mut config = config(&fixture.args)?;
        config.defer_compaction = defer;
        run(config.clone(), fixture.args.clone()).await?;
        let storage = Storage::open(&config)?;
        let payloads = successors(&fixture.payloads[3]);
        storage.write_block_bundles_wal(&[BaseChain::process_ingest(
            payloads[0].clone(),
            None,
        )?
        .0])?;
        ensure!(
            storage.last_indexed_block()? == Some(3),
            "pending WAL unexpectedly indexed"
        );
        ensure!(
            storage.dirty_complete_shards()?.is_empty(),
            "fixture shard is complete"
        );
        drop(storage);
        let storage = crate::run::init_storage(&config, None, &None).await?;
        storage.validate_snapshot_boundary()?;
        ensure!(
            storage.last_indexed_block()? == Some(4),
            "startup left successor only in WAL"
        );
        let missing = storage.missing_blocks_in_range(4..=5)?;
        ensure!(missing == vec![5], "wrong recovered missing range");
        let (pool, server) = serving_peer_at(vec![payloads[1].clone()], Some(5), 5, true);
        let result = ingest(
            Arc::clone(&storage),
            pool,
            IngestPipelineConfig {
                config: &config,
                range: 4..=5,
                head_at_startup: 5,
                db_mode_override: None,
                head_cap_override: None,
                stop_rx: None,
                tail: None,
            },
            missing,
            None,
        )
        .await;
        server.abort();
        result?;
        ensure!(
            storage.last_indexed_block()? == Some(5),
            "restart failed to advance"
        );
        storage.validate_snapshot_boundary()?;
    }
    Ok(())
}

/// Deterministically queue the detector's result and stop signal while a newer
/// processed block is being sent to the ordered writer behind an unfilled gap.
struct ReorgAfterProcessing {
    stop: watch::Sender<bool>,
    ancestor: mpsc::UnboundedSender<u64>,
}

#[tokio::test]
async fn reorg_cancellation_skips_replacement_blocks_during_drain() -> Result<()> {
    for cancel in [false, true] {
        let mut fixture = fixture(false)?;
        fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
        let config = config(&fixture.args)?;
        run(config.clone(), fixture.args.clone()).await?;
        let storage = Arc::new(Storage::open(&config)?);
        let followed = successors(&fixture.payloads[3]);
        for payload in &followed {
            storage
                .write_block_bundle_follow(&BaseChain::process_ingest(payload.clone(), None)?.0)?;
        }
        let mut replacement = followed[1].clone();
        replacement.header.extra_data = vec![42].into();
        let next = successors(&replacement)[0].clone();
        let (pool, server) = serving_peer_at(vec![next], Some(6), 6, true);
        let (stop, stop_rx) = watch::channel(false);
        let (ancestor, mut reorg_rx) = mpsc::unbounded_channel();
        let progress = cancel.then(|| {
            Arc::new(ReorgAfterProcessing {
                stop: stop.clone(),
                ancestor,
            }) as Arc<dyn ProgressReporter>
        });
        let result = ingest(
            Arc::clone(&storage),
            pool,
            IngestPipelineConfig {
                config: &config,
                range: 6..=6,
                head_at_startup: 6,
                db_mode_override: None,
                head_cap_override: Some(6),
                stop_rx: Some(stop_rx),
                tail: None,
            },
            vec![6],
            progress,
        )
        .await;
        server.abort();
        ensure!(
            storage.last_indexed_block()? == Some(5),
            "drain changed indexed head"
        );
        ensure!(
            storage.block_header(6)?.is_none(),
            "replacement block was written before rollback"
        );
        if cancel {
            ensure!(
                matches!(result?, IngestPipelineOutcome::Cancelled { .. }),
                "cancel was lost"
            );
            storage.rollback_to(reorg_rx.try_recv()?)?;
            storage.write_block_bundle_follow(&BaseChain::process_ingest(replacement, None)?.0)?;
            storage.validate_snapshot_boundary()?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| eyre!("parent mismatch was suppressed"))?;
            ensure!(
                error.to_string().contains("parent-hash mismatch"),
                "wrong error: {error}"
            );
        }
    }
    Ok(())
}

impl ProgressReporter for ReorgAfterProcessing {
    fn set_length(&self, _len: u64) {}
    fn inc(&self, _delta: u64) {
        let _ = self.ancestor.send(4);
        let _ = self.stop.send(true);
    }
}

#[tokio::test]
async fn cancelled_buffered_gap_allows_queued_reorg_but_completion_rejects_it() -> Result<()> {
    for cancel in [false, true] {
        let mut fixture = fixture(false)?;
        fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
        let config = config(&fixture.args)?;
        run(config.clone(), fixture.args.clone()).await?;
        let storage = Arc::new(Storage::open(&config)?);
        let followed = successors(&fixture.payloads[3]);
        for payload in &followed {
            storage
                .write_block_bundle_follow(&BaseChain::process_ingest(payload.clone(), None)?.0)?;
        }
        let future = successors(&followed[1]);
        let (pool, server) = serving_peer_at(vec![future[1].clone()], Some(7), 7, true);
        let (stop, stop_rx) = watch::channel(false);
        let (ancestor, mut reorg_rx) = mpsc::unbounded_channel();
        let progress = cancel.then(|| {
            Arc::new(ReorgAfterProcessing {
                stop: stop.clone(),
                ancestor,
            }) as Arc<dyn ProgressReporter>
        });
        // Only the newer block is supplied on normal completion to exercise the
        // writer's strict finalization guard. Cancellation leaves block 6 queued.
        let missing = if cancel { vec![6, 7] } else { vec![7] };
        let result = ingest(
            Arc::clone(&storage),
            pool,
            IngestPipelineConfig {
                config: &config,
                range: 6..=7,
                head_at_startup: 7,
                db_mode_override: None,
                head_cap_override: Some(7),
                stop_rx: Some(stop_rx),
                tail: None,
            },
            missing,
            progress,
        )
        .await;
        server.abort();
        if cancel {
            ensure!(
                matches!(result?, IngestPipelineOutcome::Cancelled { .. }),
                "cancelled range reported success"
            );
            ensure!(
                storage.last_indexed_block()? == Some(5),
                "gap advanced indexed head"
            );
            storage.rollback_to(reorg_rx.try_recv()?)?;
            ensure!(
                storage.last_indexed_block()? == Some(4),
                "queued rollback was bypassed"
            );
            storage.validate_snapshot_boundary()?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| eyre!("incomplete range reported success"))?;
            ensure!(
                error.to_string().contains("snapshot/P2P gap"),
                "wrong error: {error}"
            );
        }
        ensure!(
            storage.block_header(7)?.is_none(),
            "buffered block survived gap"
        );
    }
    Ok(())
}
