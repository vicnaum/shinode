use super::*;
use crate::{
    chain::BaseChain,
    snapshot::{
        handoff_tests::{serving_peer_at, successors},
        tests::{config, fixture, Fixture},
    },
    sync::historical::{
        self, IngestChain, IngestPipelineConfig, IngestPipelineTrackers, MissingBlocks,
    },
};
use serde_json::json;

fn prepared() -> Result<(Fixture, NodeConfig, RestoreArgs)> {
    let mut fixture = fixture(false)?;
    fixture.args.checkpoint_hash = Some(fixture.payloads[3].header.hash_slow());
    let config = config(&fixture.args)?;
    crate::snapshot::execute(&config, &fixture.args)?;
    let export = ExportArgs {
        output_dir: fixture.root.join("distribution"),
        start_block: 0,
        end_block: 3,
        checkpoint_hash: None,
        max_extracted_bytes: 10_000_000,
        min_free_bytes: 0,
    };
    export_local(&config, &export)?;
    let manifest = export.output_dir.join("manifest.json");
    let restore = RestoreArgs {
        manifest_sha256: format::file_sha(&manifest)?,
        manifest,
        checkpoint_hash: fixture.payloads[3].header.hash_slow(),
        output_dir: fixture.root.join("recipient"),
        max_archive_bytes: 10_000_000,
        max_extracted_bytes: 10_000_000,
        min_free_bytes: 0,
        batch_blocks: 1,
    };
    Ok((fixture, config, restore))
}

fn repin(args: &mut RestoreArgs, manifest: &Manifest) -> Result<()> {
    fs::write(&args.manifest, serde_json::to_vec_pretty(manifest)?)?;
    args.manifest_sha256 = format::file_sha(&args.manifest)?;
    Ok(())
}

fn recipient(config: &NodeConfig, args: &RestoreArgs) -> NodeConfig {
    let mut config = config.clone();
    config.data_dir = args.output_dir.clone();
    config
}

#[test]
fn restores_without_execution_inputs_and_reuses_matching_overlaps() -> Result<()> {
    let (fixture, config, args) = prepared()?;
    let source = Storage::open(&config)?;
    let expected = range_digest(&source, 0, 3)?;
    drop(source);
    // Remove all original input and native source data. Only the package remains.
    fs::remove_dir_all(&config.data_dir)?;
    fs::remove_dir_all(
        fixture
            .args
            .staging_dir
            .as_ref()
            .ok_or_else(|| eyre!("missing fixture value"))?,
    )?;
    fs::remove_file(&fixture.args.manifest)?;
    restore_local(config.clone(), &args)?;
    restore_local(config.clone(), &args)?;
    let config = recipient(&config, &args);
    let storage = Storage::open(&config)?;
    storage.validate_snapshot_boundary()?;
    check_eq(&(range_digest(&storage, 0, 3)?), &(expected))?;
    check_eq(&(storage.aggregate_stats().total_blocks), &(4))?;
    check_eq(&(storage.last_indexed_block()?), &(Some(3)))?;
    for start in [0, 2] {
        storage.verified_sealed_shard(start)?;
    }
    let manifest = read_manifest(&args)?;
    check_eq(&(manifest.shards.len()), &(2))?;
    ensure!(
        manifest.shards.iter().all(|shard| shard.files.len() == 17),
        "unexpected test result"
    );
    ensure!(
        !String::from_utf8(fs::read(&args.manifest)?)?
            .contains(&fixture.root.display().to_string()),
        "unexpected test result"
    );
    Ok(())
}

#[test]
fn malformed_manifests_limits_and_corrupt_archives_leave_no_destination() -> Result<()> {
    let (_fixture, config, args) = prepared()?;
    let original = read_manifest(&args)?;
    for change in 0..10 {
        let mut args = args.clone();
        let mut manifest = original.clone();
        match change {
            0 => manifest.version += 1,
            1 => manifest.genesis = B256::ZERO,
            2 => manifest.codec = StorageCodec::EthereumV1,
            3 => manifest.shards[0].archive = "../outside.tar.zst".into(),
            4 => {
                manifest.shards.remove(0);
            }
            5 => manifest.shards[1].first_parent = B256::ZERO,
            6 => manifest.shards[0].files[0].name = "../outside".into(),
            7 => args.checkpoint_hash = B256::ZERO,
            8 => args.max_extracted_bytes = 1,
            _ => args.max_archive_bytes = 1,
        }
        repin(&mut args, &manifest)?;
        ensure!(
            restore_local(config.clone(), &args).is_err(),
            "case {change}"
        );
        ensure!(!args.output_dir.exists(), "unexpected test result");
    }
    let mut args = args;
    repin(&mut args, &original)?;
    let path = args
        .manifest
        .parent()
        .ok_or_else(|| eyre!("missing fixture value"))?
        .join(&original.shards[1].archive);
    fs::write(path, b"truncated archive")?;
    ensure!(
        restore_local(config, &args).is_err(),
        "unexpected test result"
    );
    ensure!(!args.output_dir.exists(), "unexpected test result");
    Ok(())
}

#[test]
fn archive_rejects_links_unexpected_duplicate_and_missing_entries() -> Result<()> {
    let (_fixture, config, mut args) = prepared()?;
    let original = read_manifest(&args)?;
    let source = Storage::open(&config)?;
    let files = source.verified_sealed_shard(0)?.files;
    for case in 0..5 {
        let mut manifest = original.clone();
        let shard = &mut manifest.shards[0];
        let path = args
            .manifest
            .parent()
            .ok_or_else(|| eyre!("missing fixture value"))?
            .join(&shard.archive);
        let encoder = zstd::stream::write::Encoder::new(fs::File::create(&path)?, 3)?;
        let mut builder = tar::Builder::new(encoder);
        for (i, (name, source)) in files.iter().enumerate() {
            if case == 4 && i == 0 {
                continue;
            }
            let mut header = tar::Header::new_gnu();
            let data = fs::read(source)?;
            header.set_mode(0o644);
            header.set_size(data.len() as u64);
            if i == 0 && case < 2 {
                header.set_entry_type(if case == 0 {
                    tar::EntryType::Symlink
                } else {
                    tar::EntryType::Link
                });
                header.set_link_name("../../outside")?;
            }
            header.set_cksum();
            let name = if i == 0 && case == 2 {
                "unexpected"
            } else {
                name.as_str()
            };
            builder.append_data(&mut header, name, data.as_slice())?;
            if i == 0 && case == 3 {
                builder.append_data(&mut header, name, data.as_slice())?;
            }
        }
        builder.into_inner()?.finish()?.sync_all()?;
        shard.archive_bytes = fs::metadata(&path)?.len();
        shard.archive_sha256 = format::file_sha(&path)?;
        repin(&mut args, &manifest)?;
        ensure!(restore_local(config.clone(), &args).is_err(), "case {case}");
        ensure!(!args.output_dir.exists(), "unexpected test result");
    }
    Ok(())
}

#[test]
fn destination_conflicts_and_gaps_fail_before_adding_blocks() -> Result<()> {
    for case in 0..3 {
        let (_fixture, config, args) = prepared()?;
        let source = Storage::open(&config)?;
        let destination = recipient(&config, &args);
        let storage = Storage::open(&destination)?;
        let mut bundle = journal::stored_bundle(&source, if case == 0 { 2 } else { 0 })?
            .ok_or_else(|| eyre!("missing fixture value"))?;
        if case == 0 {
            bundle.header.extra_data = vec![123].into();
        }
        let mut retained = vec![bundle];
        if case == 2 {
            let mut parent = journal::stored_bundle(&source, 1)?
                .ok_or_else(|| eyre!("missing fixture value"))?;
            parent.header.extra_data = vec![123].into();
            retained.push(parent);
        }
        storage.write_block_bundles_wal(&retained)?;
        storage.compact_all_dirty()?;
        drop(storage);
        let mut manifest = read_manifest(&args)?;
        let mut args = args;
        if case > 0 {
            // Incoming shard 2..3 cannot join a missing or incompatible prefix.
            manifest.start = 2;
            manifest.shards.remove(0);
            repin(&mut args, &manifest)?;
        }
        ensure!(
            restore_local(config, &args).is_err(),
            "unexpected test result"
        );
        let storage = Storage::open(&destination)?;
        for bundle in &retained {
            ensure!(
                journal::compare(&storage, bundle)?,
                "unexpected test result"
            );
        }
        ensure!(storage.block_header(3)?.is_none(), "unexpected test result");
        ensure!(
            storage.snapshot_boundary().is_none(),
            "unexpected test result"
        );
    }
    Ok(())
}

#[test]
fn interrupted_restore_recovers_wal_and_blocks_unanchored_sync() -> Result<()> {
    let (_fixture, config, args) = prepared()?;
    let source = Storage::open(&config)?;
    let mut destination = recipient(&config, &args);
    let storage = Storage::open(&destination)?;
    durable::atomic_write(
        &args.output_dir.join("history-package-init.json"),
        &fs::read(&args.manifest)?,
    )?;
    storage.write_block_bundles_wal(&[
        journal::stored_bundle(&source, 0)?.ok_or_else(|| eyre!("missing fixture value"))?
    ])?;
    ensure!(
        storage.validate_snapshot_boundary().is_err(),
        "unexpected test result"
    );
    ensure!(
        Storage::configure_snapshot_sync(&mut destination).is_err(),
        "unexpected test result"
    );
    drop(storage);
    restore_local(config, &args)?;
    let storage = Storage::open(&destination)?;
    storage.validate_snapshot_boundary()?;
    Storage::configure_snapshot_sync(&mut destination)?;
    check_eq(
        &(range_digest(&storage, 0, 3)?),
        &(range_digest(&source, 0, 3)?),
    )?;
    Ok(())
}

#[test]
fn export_requires_whole_verified_sealed_shards_and_a_new_directory() -> Result<()> {
    let (fixture, config, args) = prepared()?;
    let mut export = ExportArgs {
        output_dir: fixture.root.join("second-package"),
        start_block: 1,
        end_block: 3,
        checkpoint_hash: None,
        max_extracted_bytes: 10_000_000,
        min_free_bytes: 0,
    };
    ensure!(
        export_local(&config, &export).is_err(),
        "unexpected test result"
    );
    export.start_block = 0;
    export.output_dir = args
        .manifest
        .parent()
        .ok_or_else(|| eyre!("missing fixture value"))?
        .into();
    ensure!(
        export_local(&config, &export).is_err(),
        "unexpected test result"
    );
    export.output_dir = fixture.root.join("second-package");
    export.min_free_bytes = u64::MAX;
    ensure!(
        export_local(&config, &export).is_err(),
        "unexpected test result"
    );
    ensure!(!export.output_dir.exists(), "unexpected test result");
    export.min_free_bytes = 0;
    let file = config.data_dir.join("static/shards/2/sorted/headers");
    let mut bytes = fs::read(&file)?;
    bytes[0] ^= 1;
    fs::write(file, bytes)?;
    ensure!(
        export_local(&config, &export).is_err(),
        "unexpected test result"
    );
    ensure!(!export.output_dir.exists(), "unexpected test result");
    Ok(())
}

#[test]
fn package_cli_infers_base_and_requires_recipient_pins() -> Result<()> {
    let (_fixture, config, args) = prepared()?;
    let source = config
        .data_dir
        .to_str()
        .ok_or_else(|| eyre!("missing fixture value"))?;
    let export = NodeConfig::try_from_args([
        "shinode",
        "--data-dir",
        source,
        "history-export",
        "--output-dir",
        "package",
        "--start-block",
        "0",
        "--end-block",
        "3",
    ])?;
    check_eq(&(export.chain), &(ChainKind::Base))?;
    check_eq(&(export.shard_size), &(2))?;
    let hash = format!("{:#x}", args.checkpoint_hash);
    let prefix = [
        "history-restore",
        "--manifest",
        "manifest.json",
        "--manifest-sha256",
        &args.manifest_sha256,
        "--checkpoint-hash",
        &hash,
        "--output-dir",
        "recipient",
    ];
    let restore = NodeConfig::try_from_args(std::iter::once("shinode").chain(prefix))?;
    check_eq(&(restore.chain), &(ChainKind::Base))?;
    ensure!(
        NodeConfig::try_from_args(["shinode", "--chain", "mainnet"].into_iter().chain(prefix))
            .is_err(),
        "unexpected test result"
    );
    ensure!(
        NodeConfig::try_from_args(["shinode", "--repair"].into_iter().chain(prefix)).is_err(),
        "unexpected test result"
    );
    ensure!(
        NodeConfig::try_from_args([
            "shinode",
            "history-restore",
            "--manifest",
            "manifest.json",
            "--output-dir",
            "recipient"
        ])
        .is_err(),
        "unexpected test result"
    );
    Ok(())
}

#[tokio::test]
async fn recipient_rpc_then_production_p2p_continuation_and_restart() -> Result<()> {
    let (fixture, source_config, args) = tokio::task::spawn_blocking(prepared).await??;
    restore(source_config.clone(), args.clone()).await?;
    let mut config = recipient(&source_config, &args);
    config.end_block = Some(5);
    config.rollback_window = 0;
    config.fast_sync_max_buffered_blocks = 1;
    let storage = Arc::new(Storage::open(&config)?);
    let source = Arc::new(Storage::open(&source_config)?);
    for number in 0..=3 {
        let params = json!([format!("0x{number:x}"), false]);
        check_eq(
            &(rpc(
                &config,
                Arc::clone(&storage),
                "eth_getBlockByNumber",
                params.clone(),
            )
            .await?),
            &(rpc(
                &source_config,
                Arc::clone(&source),
                "eth_getBlockByNumber",
                params,
            )
            .await?),
        )?;
    }
    let payloads = successors(&fixture.payloads[3]);
    let (pool, server) = serving_peer_at(payloads.clone(), Some(4), 5, true);
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
    for payload in payloads {
        ensure!(
            journal::compare(&storage, &BaseChain::process_ingest(payload, None)?.0)?,
            "unexpected test result"
        );
    }
    let params = json!([{"fromBlock":"0x2", "toBlock":"0x5"}]);
    let logs = rpc(&config, Arc::clone(&storage), "eth_getLogs", params.clone()).await?;
    check_eq(
        &(logs
            .as_array()
            .ok_or_else(|| eyre!("missing fixture value"))?
            .len()),
        &(4),
    )?;
    drop(storage);
    let storage = Arc::new(Storage::open(&config)?);
    storage.validate_snapshot_boundary()?;
    check_eq(
        &(rpc(&config, Arc::clone(&storage), "eth_blockNumber", json!([])).await?),
        &("0x5"),
    )?;
    check_eq(
        &(rpc(&config, storage, "eth_getLogs", params).await?),
        &(logs),
    )?;
    Ok(())
}

async fn rpc(
    config: &NodeConfig,
    storage: Arc<Storage>,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let module = crate::rpc::test_module(config, storage)?;
    let (response, _) = module
        .raw_json_request(
            &json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params}).to_string(),
            1,
        )
        .await?;
    let response: serde_json::Value = serde_json::from_str(response.get())?;
    ensure!(response.get("error").is_none(), "RPC error: {response}");
    Ok(response["result"].clone())
}

#[tokio::test]
#[ignore = "requires the verified 4984..5527 native sample and a new local evidence directory"]
async fn packaged_sample_matches_rpc_reference() -> Result<()> {
    let source = PathBuf::from(std::env::var("SHINODE_HISTORY_SOURCE")?);
    let output = PathBuf::from(std::env::var("SHINODE_HISTORY_OUTPUT")?);
    ensure!(!output.exists(), "evidence output must be new");
    let config = NodeConfig::try_from_args([
        "shinode",
        "--data-dir",
        source
            .to_str()
            .ok_or_else(|| eyre!("missing fixture value"))?,
        "history-export",
        "--start-block",
        "4984",
        "--end-block",
        "5527",
        "--output-dir",
        output
            .join("package")
            .to_str()
            .ok_or_else(|| eyre!("missing fixture value"))?,
    ])?;
    let Some(crate::cli::Command::HistoryExport(export_args)) = config.command.clone() else {
        return Err(eyre!("export command missing"));
    };
    let began = std::time::Instant::now();
    export(config.clone(), export_args).await?;
    let export_seconds = began.elapsed().as_secs_f64();
    let manifest = output.join("package/manifest.json");
    let manifest_sha256 = format::file_sha(&manifest)?;
    let destination = output.join("recipient");
    let restore_config = NodeConfig::try_from_args([
        "shinode",
        "history-restore",
        "--manifest",
        manifest
            .to_str()
            .ok_or_else(|| eyre!("missing fixture value"))?,
        "--manifest-sha256",
        &manifest_sha256,
        "--checkpoint-hash",
        "0x2246525ba06cf8771e47607736d502e446cf5cffe00a2646ceb12de043d4205f",
        "--output-dir",
        destination
            .to_str()
            .ok_or_else(|| eyre!("missing fixture value"))?,
    ])?;
    let Some(crate::cli::Command::HistoryRestore(args)) = restore_config.command.clone() else {
        return Err(eyre!("restore command missing"));
    };
    let manifest = read_manifest(&args)?;
    let began = std::time::Instant::now();
    restore(restore_config, args).await?;
    let restore_seconds = began.elapsed().as_secs_f64();
    let mut config = config;
    config.command = None;
    config.start_block = 4984;
    config.end_block = Some(5527);
    let original = Storage::open(&config)?;
    config.data_dir = destination;
    let storage = Arc::new(Storage::open(&config)?);
    check_eq(
        &(range_digest(&storage, 4984, 5527)?),
        &(range_digest(&original, 4984, 5527)?),
    )?;
    storage.validate_snapshot_boundary()?;
    check_eq(&(storage.last_indexed_block()?), &(Some(5527)))?;
    let reference: serde_json::Value =
        serde_json::from_slice(&fs::read(source.join("proof.json"))?)?;
    verify_reference(&config, &storage, &reference).await?;
    let (compressed, native) = manifest.sizes()?;
    fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(&json!({
            "range": [4984,5527], "blocks": 544, "shards": manifest.shards.len(),
            "native_bytes": native, "archive_bytes": compressed,
            "manifest_bytes": fs::metadata(output.join("package/manifest.json"))?.len(),
            "manifest_sha256": manifest_sha256, "export_seconds": export_seconds,
            "restore_seconds": restore_seconds, "rpc_reference_samples": 16,
            "full_bundle_digest_matched": true, "execution_snapshot_inputs_used": false,
        }))?,
    )?;
    Ok(())
}

#[test]
fn adjoining_packages_extend_the_anchor_and_older_overlaps_preserve_it() -> Result<()> {
    let (fixture, config, mut args) = prepared()?;
    let full = read_manifest(&args)?;
    let mut first = full.clone();
    first.shards.truncate(1);
    first.end = 1;
    first.end_hash = fixture.payloads[1].header.hash_slow();
    args.checkpoint_hash = first.end_hash;
    repin(&mut args, &first)?;
    restore_local(config.clone(), &args)?;
    let mut second = full;
    second.shards.remove(0);
    second.start = 2;
    args.checkpoint_hash = second.end_hash;
    repin(&mut args, &second)?;
    restore_local(config.clone(), &args)?;
    args.checkpoint_hash = first.end_hash;
    repin(&mut args, &first)?;
    restore_local(config.clone(), &args)?;
    let storage = Storage::open(&recipient(&config, &args))?;
    let boundary = storage
        .snapshot_boundary()
        .ok_or_else(|| eyre!("missing fixture value"))?;
    check_eq(&(boundary.start, boundary.number), &(0, 3))?;
    check_eq(&(boundary.hash), &(fixture.payloads[3].header.hash_slow()))?;
    storage.validate_snapshot_boundary()?;
    Ok(())
}

fn check_eq<L: PartialEq<R> + std::fmt::Debug, R: std::fmt::Debug>(
    left: &L,
    right: &R,
) -> Result<()> {
    ensure!(left == right, "expected {right:?}, got {left:?}");
    Ok(())
}

async fn verify_reference(
    config: &NodeConfig,
    storage: &Arc<Storage>,
    reference: &serde_json::Value,
) -> Result<()> {
    for entry in reference["blocks"]
        .as_array()
        .ok_or_else(|| eyre!("missing reference blocks"))?
    {
        let number = entry["number"]
            .as_u64()
            .ok_or_else(|| eyre!("missing fixture value"))?;
        let tag = format!("0x{number:x}");
        check_eq(
            &(rpc(
                config,
                Arc::clone(storage),
                "eth_getBlockByNumber",
                json!([tag, false]),
            )
            .await?),
            &(entry["block"]),
        )?;
        check_eq(
            &(rpc(
                config,
                Arc::clone(storage),
                "eth_getLogs",
                json!([{"fromBlock":tag,"toBlock":tag}]),
            )
            .await?),
            &(entry["logs"]),
        )?;
        let receipts = storage
            .block_receipts(number)?
            .ok_or_else(|| eyre!("missing fixture value"))?;
        let deposits: Vec<_> = receipts.iter().enumerate().filter_map(|(index, receipt)| {
            if let crate::storage::StoredReceiptRef::Op(op_alloy_consensus::OpReceipt::Deposit(r)) = receipt {
                Some(json!({"transaction_index":index,"nonce":r.deposit_nonce,"receipt_version":r.deposit_receipt_version}))
            } else { None }
        }).collect();
        check_eq(&(json!(deposits)), &(entry["deposits"]))?;
    }
    Ok(())
}

#[test]
fn restore_recovers_an_interruption_before_storage_metadata() -> Result<()> {
    let (_fixture, config, args) = prepared()?;
    fs::create_dir_all(args.output_dir.join("peers"))?;
    fs::write(args.output_dir.join("meta.json.tmp"), b"partial metadata")?;
    durable::atomic_write(
        &args.output_dir.join("history-package-init.json"),
        &fs::read(&args.manifest)?,
    )?;
    restore_local(config.clone(), &args)?;
    let storage = Storage::open(&recipient(&config, &args))?;
    storage.validate_snapshot_boundary()?;
    check_eq(&storage.last_indexed_block()?, &Some(3))?;
    Ok(())
}

#[test]
fn rejected_earlier_package_preserves_resume_boundary_and_valid_restores() -> Result<()> {
    let (fixture, config, mut args) = prepared()?;
    let full = read_manifest(&args)?;
    let mut suffix = full.clone();
    suffix.start = 2;
    suffix.shards.remove(0);
    repin(&mut args, &suffix)?;
    restore_local(config.clone(), &args)?;
    let meta_path = args.output_dir.join("meta.json");
    let boundary_path = args.output_dir.join("snapshot-boundary.json");
    let original_meta = fs::read(&meta_path)?;
    let original_boundary = fs::read(&boundary_path)?;

    let mut prefix = full;
    prefix.end = 1;
    prefix.end_hash = fixture.payloads[1].header.hash_slow();
    prefix.shards.truncate(1);
    args.checkpoint_hash = prefix.end_hash;
    repin(&mut args, &prefix)?;
    ensure!(
        restore_local(config.clone(), &args).is_err(),
        "earlier package accepted"
    );
    check_eq(&fs::read(&meta_path)?, &original_meta)?;
    check_eq(&fs::read(&boundary_path)?, &original_boundary)?;
    check_eq(
        &Storage::read_resume_start(&args.output_dir, config.chain_id, config.genesis_hash)?,
        &Some(2),
    )?;

    args.checkpoint_hash = suffix.end_hash;
    repin(&mut args, &suffix)?;
    restore_local(config, &args)?;
    let mut restart = NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        args.output_dir
            .to_str()
            .ok_or_else(|| eyre!("invalid fixture path"))?,
    ])?;
    Storage::configure_snapshot_sync(&mut restart)?;
    check_eq(&restart.start_block, &2)?;
    let storage = Storage::open(&restart)?;
    storage.validate_snapshot_boundary()?;
    check_eq(&storage.last_indexed_block()?, &Some(3))?;
    ensure!(
        storage.block_header(0)?.is_none(),
        "rejected package wrote block 0"
    );
    Ok(())
}

#[test]
fn unanchored_overlap_rejection_preserves_resume_and_allows_a_matching_restore() -> Result<()> {
    let (fixture, config, mut args) = prepared()?;
    let mut suffix = read_manifest(&args)?;
    suffix.start = 2;
    suffix.shards.remove(0);
    let mut destination = recipient(&config, &args);
    destination.start_block = 2;
    let storage = Storage::open(&destination)?;
    let mut bundles = fixture
        .payloads
        .iter()
        .cloned()
        .map(|payload| BaseChain::process_ingest(payload, None).map(|result| result.0))
        .collect::<Result<Vec<_>>>()?;
    storage.write_block_bundles_wal(&bundles[2..])?;
    storage.compact_all_dirty()?;
    ensure!(
        storage.snapshot_boundary().is_none(),
        "expected unanchored P2P history"
    );
    drop(storage);
    let meta_path = args.output_dir.join("meta.json");
    let before = fs::read(&meta_path)?;

    // A separately valid package follows a conflicting branch at block 2.
    bundles[2].header.extra_data = vec![123].into();
    bundles[3].header.parent_hash = bundles[2].header.hash_slow();
    let conflicting_hash = bundles[3].header.hash_slow();
    let mut publisher = config.clone();
    publisher.data_dir = fixture.root.join("conflicting-publisher");
    let storage = Storage::open(&publisher)?;
    storage.write_block_bundles_wal(&bundles)?;
    storage.compact_all_dirty()?;
    storage.pin_snapshot_boundary(0, 3, conflicting_hash, &args.manifest_sha256)?;
    drop(storage);
    let package = fixture.root.join("conflicting-package");
    export_local(
        &publisher,
        &ExportArgs {
            output_dir: package.clone(),
            start_block: 0,
            end_block: 3,
            checkpoint_hash: None,
            max_extracted_bytes: args.max_extracted_bytes,
            min_free_bytes: 0,
        },
    )?;
    let mut conflicting = args.clone();
    conflicting.manifest = package.join("manifest.json");
    conflicting.manifest_sha256 = format::file_sha(&conflicting.manifest)?;
    conflicting.checkpoint_hash = conflicting_hash;
    let error = restore_local(config.clone(), &conflicting)
        .err()
        .ok_or_else(|| eyre!("conflicting overlap must fail"))?;
    ensure!(
        error.to_string().contains("conflicting block hash"),
        "unexpected rejection: {error}"
    );
    ensure!(
        fs::read(&meta_path)? == before,
        "rejected restore changed resume metadata"
    );
    ensure!(
        !args.output_dir.join("history-package-init.json").exists(),
        "rejected restore fenced P2P history"
    );
    check_eq(
        &Storage::read_resume_start(&args.output_dir, config.chain_id, config.genesis_hash)?,
        &Some(2),
    )?;

    repin(&mut args, &suffix)?;
    restore_local(config, &args)?;
    let storage = Storage::open(&destination)?;
    storage.validate_snapshot_boundary()?;
    check_eq(&storage.last_indexed_block()?, &Some(3))?;
    ensure!(
        storage.block_header(0)?.is_none(),
        "rejected package wrote block 0"
    );
    Ok(())
}

#[test]
fn unanchored_resume_changes_only_after_combined_parent_validation() -> Result<()> {
    for conflicting_parent in [false, true] {
        let (fixture, config, mut args) = prepared()?;
        let mut prefix = read_manifest(&args)?;
        prefix.end = 1;
        prefix.end_hash = fixture.payloads[1].header.hash_slow();
        prefix.shards.truncate(1);
        args.checkpoint_hash = prefix.end_hash;
        repin(&mut args, &prefix)?;

        let mut destination = recipient(&config, &args);
        destination.start_block = 2;
        let storage = Storage::open(&destination)?;
        let mut bundles = fixture.payloads[2..]
            .iter()
            .cloned()
            .map(|payload| BaseChain::process_ingest(payload, None).map(|result| result.0))
            .collect::<Result<Vec<_>>>()?;
        if conflicting_parent {
            bundles[0].header.parent_hash = B256::ZERO;
            bundles[1].header.parent_hash = bundles[0].header.hash_slow();
        }
        storage.write_block_bundles_wal(&bundles)?;
        storage.compact_all_dirty()?;
        drop(storage);
        let meta_path = args.output_dir.join("meta.json");
        let before = fs::read(&meta_path)?;
        let result = restore_local(config.clone(), &args);
        if conflicting_parent {
            let error = result
                .err()
                .ok_or_else(|| eyre!("conflicting parent accepted"))?;
            ensure!(
                error.to_string().contains("restore parent mismatch"),
                "unexpected rejection: {error}"
            );
            ensure!(
                fs::read(&meta_path)? == before,
                "parent rejection changed resume metadata"
            );
        } else {
            result?;
            destination.start_block = 0;
        }
        check_eq(
            &Storage::read_resume_start(&args.output_dir, config.chain_id, config.genesis_hash)?,
            &Some(destination.start_block),
        )?;
        let storage = Storage::open(&destination)?;
        if conflicting_parent {
            ensure!(
                storage.block_header(0)?.is_none(),
                "rejected package wrote block 0"
            );
            ensure!(
                storage.snapshot_boundary().is_none(),
                "rejected package pinned a boundary"
            );
        } else {
            storage.validate_snapshot_boundary()?;
            check_eq(&storage.last_indexed_block()?, &Some(3))?;
        }
    }
    Ok(())
}
