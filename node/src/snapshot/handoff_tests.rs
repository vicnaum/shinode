use super::{
    tests::{config, fixture},
    *,
};
use crate::{
    p2p::{NetworkPeer, PeerPool},
    sync::{
        historical::{self, IngestPipelineConfig, IngestPipelineTrackers, MissingBlocks},
        BlockPayload,
    },
};
use alloy_primitives::B256;
use reth_eth_wire_types::{BlockBodies, BlockHeaders, EthVersion, Receipts};
use reth_network_api::{PeerId, PeerRequest, PeerRequestSender};
use tokio::sync::mpsc;

pub(super) fn successors(previous: &BlockPayload<BaseChain>) -> Vec<BlockPayload<BaseChain>> {
    let mut payloads = Vec::new();
    let mut previous = previous.clone();
    for _ in 0..2 {
        let mut next = previous.clone();
        next.header.number += 1;
        next.header.parent_hash = previous.header.hash_slow();
        payloads.push(next.clone());
        previous = next;
    }
    payloads
}

fn serving_peer(
    payloads: Vec<BlockPayload<BaseChain>>,
    earliest: Option<u64>,
    bodies: bool,
) -> (Arc<PeerPool<BaseChain>>, tokio::task::JoinHandle<()>) {
    serving_peer_at(payloads, earliest, 5, bodies)
}

pub(super) fn serving_peer_at(
    payloads: Vec<BlockPayload<BaseChain>>,
    earliest: Option<u64>,
    head: u64,
    bodies: bool,
) -> (Arc<PeerPool<BaseChain>>, tokio::task::JoinHandle<()>) {
    let (tx, mut rx) = mpsc::channel(32);
    let peer_id = PeerId::random();
    let peer = NetworkPeer::<BaseChain> {
        peer_id,
        eth_version: EthVersion::Eth68,
        messages: PeerRequestSender::new(peer_id, tx),
        head_number: head,
        earliest_block: earliest,
        latest_block: Some(head),
    };
    let pool = Arc::new(crate::p2p::peer_pool_for_tests(vec![peer]));
    let task = tokio::spawn(async move {
        while let Some(request) = rx.recv().await {
            match request {
                PeerRequest::GetBlockHeaders { request, response } => {
                    let start = match request.start_block {
                        reth_eth_wire_types::BlockHashOrNumber::Number(number) => number,
                        reth_eth_wire_types::BlockHashOrNumber::Hash(hash) => payloads
                            .iter()
                            .find(|p| p.header.hash_slow() == hash)
                            .map_or(u64::MAX, |p| p.header.number),
                    };
                    let headers = payloads
                        .iter()
                        .filter(|p| {
                            p.header.number >= start
                                && p.header.number < start.saturating_add(request.limit)
                        })
                        .take(request.limit as usize)
                        .map(|p| p.header.clone())
                        .collect();
                    let _ = response.send(Ok(BlockHeaders(headers)));
                }
                PeerRequest::GetBlockBodies { request, response } => {
                    let result = request
                        .0
                        .iter()
                        .filter_map(|hash| payloads.iter().find(|p| p.header.hash_slow() == *hash))
                        .filter(|_| bodies)
                        .map(|p| p.body.clone())
                        .collect();
                    let _ = response.send(Ok(BlockBodies(result)));
                }
                PeerRequest::GetReceipts { request, response } => {
                    let result = request
                        .0
                        .iter()
                        .filter_map(|hash| payloads.iter().find(|p| p.header.hash_slow() == *hash))
                        .filter(|_| bodies)
                        .map(|p| {
                            p.receipts
                                .iter()
                                .cloned()
                                .map(TxReceipt::into_with_bloom)
                                .collect()
                        })
                        .collect();
                    let _ = response.send(Ok(Receipts(result)));
                }
                _ => {}
            }
        }
    });
    (pool, task)
}

async fn rpc(
    storage: Arc<Storage>,
    config: &NodeConfig,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let module = crate::rpc::test_module(config, storage)?;
    let request = serde_json::json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params});
    let (response, _) = module.raw_json_request(&request.to_string(), 1).await?;
    let response: serde_json::Value = serde_json::from_str(response.get())?;
    ensure!(response.get("error").is_none(), "RPC error: {response}");
    Ok(response["result"].clone())
}

fn check_boundary_logs(value: &serde_json::Value) -> Result<()> {
    let logs = value
        .as_array()
        .ok_or_else(|| eyre!("logs are not an array"))?;
    ensure!(
        logs.len() == 4,
        "expected one log per block across the boundary"
    );
    for (log, number) in logs.iter().zip(2..=5) {
        ensure!(
            log["blockNumber"] == format!("0x{number:x}"),
            "wrong log block"
        );
        ensure!(log["logIndex"] == "0x0", "wrong block-wide log index");
    }
    Ok(())
}

#[tokio::test]
async fn snapshot_p2p_pipeline_restart_rpc_and_rollback() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    let mut config = config(&fixture.args)?;
    config.end_block = Some(5);
    config.rollback_window = 0;
    config.fast_sync_max_buffered_blocks = 1;
    run(config.clone(), fixture.args.clone()).await?;
    let storage = Arc::new(Storage::open(&config)?);
    storage.validate_snapshot_boundary()?;
    let payloads = successors(&fixture.payloads[3]);
    let (pool, server) = serving_peer(payloads.clone(), Some(4), true);
    historical::preflight_snapshot_handoff(&storage, &pool, 5).await?;
    tokio::time::timeout(
        std::time::Duration::from_secs(20),
        historical::run_ingest_pipeline(
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
            MissingBlocks::Precomputed(vec![4, 5]),
            IngestPipelineTrackers {
                progress: None,
                stats: None,
                bench: None,
                events: None,
            },
            historical::build_peer_health_tracker(&config),
        ),
    )
    .await??;
    server.abort();
    ensure!(
        storage.last_indexed_block()? == Some(5),
        "P2P continuation did not advance"
    );
    for payload in &payloads {
        ensure!(
            journal::compare(
                &storage,
                &BaseChain::process_ingest(payload.clone(), None)?.0
            )?,
            "P2P payload changed"
        );
    }
    let before = rpc(
        Arc::clone(&storage),
        &config,
        "eth_getLogs",
        serde_json::json!([{"fromBlock":"0x2","toBlock":"0x5"}]),
    )
    .await?;
    check_boundary_logs(&before)?;
    ensure!(
        rpc(
            Arc::clone(&storage),
            &config,
            "eth_blockNumber",
            serde_json::json!([])
        )
        .await?
            == "0x5",
        "wrong RPC head"
    );
    drop(storage);
    Storage::repair(&config)?;
    let storage = Arc::new(Storage::open(&config)?);
    storage.validate_snapshot_boundary()?;
    ensure!(
        rpc(
            Arc::clone(&storage),
            &config,
            "eth_getLogs",
            serde_json::json!([{"fromBlock":"0x2","toBlock":"0x5"}])
        )
        .await?
            == before,
        "cross-boundary logs changed after restart"
    );
    ensure!(
        storage.rollback_to(2).is_err(),
        "rollback rewrote trusted history"
    );
    ensure!(
        storage.last_indexed_block()? == Some(5),
        "rejected rollback changed head"
    );
    storage.rollback_to(4)?;
    let mut replacement = payloads[1].clone();
    replacement.header.extra_data = vec![7].into();
    storage.write_block_bundle_follow(&BaseChain::process_ingest(replacement, None)?.0)?;
    storage.validate_snapshot_boundary()?;
    Ok(())
}

#[tokio::test]
async fn snapshot_restart_reorg_respects_window_and_boundary() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    let config = config(&fixture.args)?;
    run(config.clone(), fixture.args.clone()).await?;
    let storage = Storage::open(&config)?;
    let payloads = successors(&fixture.payloads[3]);
    for payload in &payloads {
        storage.write_block_bundle_follow(&BaseChain::process_ingest(payload.clone(), None)?.0)?;
    }
    drop(storage);
    let storage = Storage::open(&config)?;
    let mut fork = payloads[1].clone();
    fork.header.extra_data = vec![42].into();
    let (pool, server) = serving_peer(
        vec![fixture.payloads[3].clone(), payloads[0].clone(), fork],
        Some(3),
        true,
    );
    ensure!(
        historical::reconcile_snapshot_tip(&storage, &pool, 0)
            .await
            .is_err(),
        "escaped zero rollback window"
    );
    ensure!(
        storage.last_indexed_block()? == Some(5),
        "rejected reorg mutated storage"
    );
    ensure!(
        historical::reconcile_snapshot_tip(&storage, &pool, 2).await?,
        "shallow reorg was not recovered"
    );
    ensure!(
        storage.last_indexed_block()? == Some(4),
        "wrong common ancestor"
    );
    historical::preflight_snapshot_handoff(&storage, &pool, 5).await?;
    server.abort();
    let mut boundary_fork = fixture.payloads[3].clone();
    boundary_fork.header.extra_data = vec![99].into();
    let mut deep_fork = vec![boundary_fork.clone()];
    deep_fork.extend(successors(&boundary_fork));
    let (pool, server) = serving_peer(deep_fork, Some(3), true);
    ensure!(
        historical::reconcile_snapshot_tip(&storage, &pool, 64)
            .await
            .is_err(),
        "crossed trusted boundary"
    );
    ensure!(
        storage.last_indexed_block()? == Some(4),
        "deep conflict mutated storage"
    );
    storage.validate_snapshot_boundary()?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn handoff_rejects_pruning_unknown_availability_and_boundary_mismatch() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    run(config(&fixture.args)?, fixture.args.clone()).await?;
    let storage = Storage::open(&config(&fixture.args)?)?;
    for (earliest, bodies, mismatch) in [
        (Some(5), true, false),
        (None, false, false),
        (Some(4), true, true),
    ] {
        let mut payloads = successors(&fixture.payloads[3]);
        if mismatch {
            payloads[0].header.parent_hash = B256::ZERO;
        }
        let (pool, server) = serving_peer(payloads, earliest, bodies);
        let error = historical::preflight_snapshot_handoff(&storage, &pool, 5)
            .await
            .err()
            .ok_or_else(|| eyre!("bad handoff accepted"))?;
        ensure!(
            error.to_string().contains(if mismatch {
                "boundary mismatch"
            } else {
                "history gap"
            }),
            "wrong handoff error: {error}"
        );
        server.abort();
        ensure!(
            storage.last_indexed_block()? == Some(3) && storage.block_header(4)?.is_none(),
            "probe changed storage"
        );
    }
    Ok(())
}

#[test]
fn trusted_boundary_is_required_and_wrong_hash_never_writes_output() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(B256::ZERO);
    ensure!(
        execute(&config(&fixture.args)?, &fixture.args).is_err(),
        "wrong trusted hash accepted"
    );
    ensure!(
        !fixture.args.output_dir.as_ref().is_some_and(|p| p.exists()),
        "wrong anchor created output"
    );
    fixture.args.checkpoint_hash = None;
    execute(&config(&fixture.args)?, &fixture.args)?;
    let storage = Storage::open(&config(&fixture.args)?)?;
    ensure!(
        storage.validate_snapshot_boundary().is_err(),
        "untrusted import handed off"
    );
    drop(storage);
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    execute(&config(&fixture.args)?, &fixture.args)?;
    let storage = Storage::open(&config(&fixture.args)?)?;
    storage.validate_snapshot_boundary()?;
    let mut restart = config(&fixture.args)?;
    restart.start_block_override = None;
    restart.start_block = 99;
    Storage::configure_snapshot_sync(&mut restart)?;
    ensure!(
        restart.start_block == 0,
        "restart lost the imported lower bound"
    );
    restart.start_block_override = Some(1);
    ensure!(
        Storage::configure_snapshot_sync(&mut restart).is_err(),
        "restart skipped imported history"
    );
    Ok(())
}

#[test]
fn filling_a_gap_checks_the_entire_preexisting_suffix() -> Result<()> {
    for corrupt in [false, true] {
        let fixture = fixture(false)?;
        let config = config(&fixture.args)?;
        execute(&config, &fixture.args)?;
        let storage = Storage::open(&config)?;
        let payloads = successors(&fixture.payloads[3]);
        let mut sixth = successors(&payloads[1])[0].clone();
        if corrupt {
            sixth.header.parent_hash = B256::ZERO;
        }
        storage.write_block_bundles_wal(&[
            BaseChain::process_ingest(payloads[1].clone(), None)?.0,
            BaseChain::process_ingest(sixth, None)?.0,
        ])?;
        storage.compact_all_dirty()?;
        storage.pin_snapshot_boundary(0, 3, fixture.payloads[3].header.hash_slow(), "fixture")?;
        storage.validate_snapshot_boundary()?;
        ensure!(storage.last_indexed_block()? == Some(3), "head crossed gap");
        let result = storage
            .write_block_bundle_follow(&BaseChain::process_ingest(payloads[0].clone(), None)?.0);
        if corrupt {
            ensure!(
                result.is_err(),
                "disconnected suffix with broken parent was accepted"
            );
            ensure!(
                storage.block_header(4)?.is_none(),
                "rejected gap fill changed storage"
            );
        } else {
            result?;
            ensure!(
                storage.last_indexed_block()? == Some(6),
                "linked suffix was not joined"
            );
            storage.validate_snapshot_boundary()?;
        }
    }
    Ok(())
}

#[test]
fn anchored_import_batch_advances_across_a_stored_suffix() -> Result<()> {
    let fixture = fixture(false)?;
    let config = config(&fixture.args)?;
    execute(&config, &fixture.args)?;
    let storage = Storage::open(&config)?;
    let fourth_fifth = successors(&fixture.payloads[3]);
    let sixth_seventh = successors(&fourth_fifth[1]);
    let existing = [
        BaseChain::process_ingest(fourth_fifth[1].clone(), None)?.0,
        BaseChain::process_ingest(sixth_seventh[0].clone(), None)?.0,
    ];
    storage.write_block_bundles_wal(&existing)?;
    storage.compact_all_dirty()?;
    storage.pin_snapshot_boundary(0, 3, fixture.payloads[3].header.hash_slow(), "fixture")?;
    let missing = [
        BaseChain::process_ingest(fourth_fifth[0].clone(), None)?.0,
        BaseChain::process_ingest(sixth_seventh[1].clone(), None)?.0,
    ];
    let mut conflicting = missing.clone();
    conflicting[1].header.parent_hash = conflicting[0].header.hash_slow();
    ensure!(
        storage.write_block_bundles_wal(&conflicting).is_err(),
        "block 7 must extend the stored suffix, not block 4"
    );
    ensure!(
        storage.missing_blocks_in_range(4..=7)? == vec![4, 7],
        "rejected batch wrote partial WAL data"
    );
    storage.write_block_bundles_wal(&missing)?;
    storage.compact_all_dirty()?;
    drop(storage);
    let storage = Storage::open(&config)?;
    storage.validate_snapshot_boundary()?;
    ensure!(
        storage.last_indexed_block()? == Some(7),
        "batch did not fill both gaps"
    );
    for bundle in existing.iter().chain(&missing) {
        ensure!(journal::compare(&storage, bundle)?, "stored bundle changed");
    }
    Ok(())
}

#[test]
fn follow_cli_requires_a_trusted_hash_and_disallows_dry_run() -> Result<()> {
    let prefix = [
        "shinode",
        "snapshot-import",
        "--manifest",
        "manifest.json",
        "--manifest-sha256",
        "abc",
        "--start-block",
        "0",
        "--end-block",
        "3",
        "--staging-dir",
        "stage",
        "--output-dir",
        "output",
        "--follow",
    ];
    ensure!(
        NodeConfig::try_from_args(prefix).is_err(),
        "unanchored --follow accepted"
    );
    let hash = format!("{:#x}", B256::repeat_byte(1));
    let valid: Vec<_> = prefix
        .into_iter()
        .chain(["--checkpoint-hash", hash.as_str()])
        .collect();
    NodeConfig::try_from_args(valid.clone())?;
    ensure!(
        NodeConfig::try_from_args(valid.into_iter().chain(["--dry-run"])).is_err(),
        "dry-run started follow"
    );
    Ok(())
}

#[tokio::test]
async fn indexed_head_stops_at_gaps_and_writes_require_linked_parents() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    let config = config(&fixture.args)?;
    run(config.clone(), fixture.args.clone()).await?;
    let storage = Arc::new(Storage::open(&config)?);
    storage.set_last_indexed_block(99)?;
    ensure!(
        rpc(
            Arc::clone(&storage),
            &config,
            "eth_blockNumber",
            serde_json::json!([])
        )
        .await?
            == "0x3",
        "RPC head crossed a gap"
    );
    let payloads = successors(&fixture.payloads[3]);
    let mut next = BaseChain::process_ingest(payloads[0].clone(), None)?.0;
    next.header.parent_hash = B256::ZERO;
    ensure!(
        storage.write_block_bundle_follow(&next).is_err(),
        "unlinked follow accepted"
    );
    ensure!(
        storage.write_block_bundles_wal(&[next]).is_err(),
        "unlinked WAL accepted"
    );
    let fifth = BaseChain::process_ingest(payloads[1].clone(), None)?.0;
    ensure!(
        storage.write_block_bundle_follow(&fifth).is_err(),
        "gap skipped"
    );
    storage.write_block_bundle_follow(&BaseChain::process_ingest(payloads[0].clone(), None)?.0)?;
    ensure!(storage.last_indexed_block()? == Some(4), "gap fill failed");
    Ok(())
}
