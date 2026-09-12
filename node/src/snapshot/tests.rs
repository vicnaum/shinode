use super::*;
use crate::{
    chain::ChainTypes,
    sync::{
        validation::tests::{base_payload, commit_payload, CANYON},
        BlockPayload,
    },
};
use alloy_consensus::{BlockBody, TxReceipt};
use reth_chainspec::EthChainSpec;
use reth_codecs::Compact;
use reth_nippy_jar::{NippyJar, NippyJarWriter};
use reth_static_file_types::{SegmentHeader, SegmentRangeInclusive, StaticFileSegment};
use serde_json::json;
use sha2::{Digest, Sha256};

pub(super) fn arguments(root: &Path) -> SnapshotArgs {
    SnapshotArgs {
        chain: ChainKind::Base,
        manifest: root.join("manifest.json"),
        manifest_sha256: String::new(),
        start_block: 0,
        end_block: 3,
        staging_dir: Some(root.join("staging")),
        output_dir: Some(root.join("output")),
        dry_run: false,
        offline: true,
        max_download_bytes: 512 * 1024 * 1024,
        max_extracted_bytes: 1024 * 1024 * 1024,
        min_free_bytes: 0,
        concurrency: 2,
        retries: 1,
        timeout_seconds: 10,
        batch_blocks: 2,
        max_transactions_per_block: 100,
    }
}

fn config(args: &SnapshotArgs) -> Result<NodeConfig> {
    let mut config = crate::test_utils::base_config(
        args.output_dir
            .clone()
            .ok_or_else(|| eyre!("output missing"))?,
    );
    config.chain = ChainKind::Base;
    config.chain_id = 8453;
    config.genesis_hash = ChainKind::Base.genesis_hash();
    config.start_block = args.start_block;
    config.shard_size = 2;
    Ok(config)
}

struct Fixture {
    root: PathBuf,
    args: SnapshotArgs,
    payloads: Vec<BlockPayload<BaseChain>>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn bytes(value: &impl Compact) -> Vec<u8> {
    let mut bytes = Vec::new();
    value.to_compact(&mut bytes);
    bytes
}

fn fixture_payloads(bad_last: bool) -> Vec<BlockPayload<BaseChain>> {
    let genesis = BaseChain::chain_spec().genesis_header().clone();
    let mut payloads = vec![BlockPayload {
        header: genesis,
        body: BlockBody::default(),
        receipts: Vec::new(),
    }];
    for number in 1..4 {
        let mut payload = base_payload(CANYON - 1);
        payload.header.number = number;
        payload.header.parent_hash = payloads[number as usize - 1].header.hash_slow();
        payload.header.gas_used = payload.receipts[0].cumulative_gas_used();
        if bad_last && number == 3 {
            payload.header.receipts_root = alloy_primitives::B256::repeat_byte(99);
        }
        payloads.push(payload);
    }
    payloads
}

fn fixture(bad_last: bool) -> Result<Fixture> {
    let root = crate::test_utils::temp_dir("snapshot");
    fs::create_dir_all(&root)?;
    let mut args = arguments(&root);
    let staging = args
        .staging_dir
        .as_ref()
        .ok_or_else(|| eyre!("staging missing"))?;
    let payloads = fixture_payloads(bad_last);
    let mut components = serde_json::Map::new();
    for (name, kind) in [
        ("headers", StaticFileSegment::Headers),
        ("transactions", StaticFileSegment::Transactions),
        ("receipts", StaticFileSegment::Receipts),
    ] {
        let mut urls = Vec::new();
        let mut sizes = Vec::new();
        let mut outputs = Vec::new();
        for start in [0u64, 2] {
            let end = start + 1;
            let range = SegmentRangeInclusive::new(start, end);
            let tx_range = (kind != StaticFileSegment::Headers)
                .then(|| SegmentRangeInclusive::new(start.saturating_sub(1), end - 1));
            let header = SegmentHeader::new(range, Some(range), tx_range, kind);
            let dir = staging.join("chunks").join(format!("{name}-{start}-{end}"));
            fs::create_dir_all(dir.join("static_files"))?;
            let relative = format!("static_files/static_file_{name}_{start}_{end}");
            let path = dir.join(&relative);
            let mut rows: Vec<Vec<Vec<u8>>> = vec![Vec::new(); kind.columns()];
            for payload in &payloads[start as usize..=end as usize] {
                match kind {
                    StaticFileSegment::Headers => {
                        rows[0].push(bytes(&payload.header));
                        rows[1].push(vec![0]);
                        rows[2].push(payload.header.hash_slow().as_slice().to_vec());
                    }
                    StaticFileSegment::Transactions => {
                        rows[0].extend(payload.body.transactions.iter().map(bytes));
                    }
                    StaticFileSegment::Receipts => {
                        rows[0].extend(payload.receipts.iter().map(bytes));
                    }
                    _ => return Err(eyre!("unexpected fixture segment")),
                }
            }
            let count = rows[0].len() as u64;
            let mut writer = NippyJarWriter::new(NippyJar::new(kind.columns(), &path, header))?;
            let columns = rows
                .iter()
                .map(|column| {
                    column.iter().map(|row| {
                        Ok::<&[u8], Box<dyn std::error::Error + Send + Sync>>(row.as_slice())
                    })
                })
                .collect::<Vec<_>>();
            writer.append_rows(columns, count)?;
            writer.commit()?;
            drop(writer);
            let files = ["", ".conf", ".off"].into_iter().map(|suffix| -> Result<serde_json::Value> {
                let name = format!("{relative}{suffix}");
                let contents = fs::read(dir.join(&name))?;
                Ok(json!({"path": name, "size": contents.len(), "blake3": blake3::hash(&contents).to_hex().as_str()}))
            }).collect::<Result<Vec<_>>>()?;
            sizes.push(
                files
                    .iter()
                    .map(|f| f["size"].as_u64().unwrap_or(0))
                    .sum::<u64>(),
            );
            urls.push(format!("static_files/{name}-{start}-{end}.tar.zst"));
            outputs.push(files);
        }
        components.insert(name.to_owned(), json!({"blocks_per_file": 2, "total_blocks": 4,
            "chunk_files": urls, "chunk_sizes": [1, 1], "chunk_decompressed_sizes": sizes, "chunk_output_files": outputs}));
    }
    components.insert("state".to_owned(), json!({"must_not_fetch": true}));
    let raw = serde_json::to_vec(&json!({"block": 3, "chain_id": 8453, "storage_version": 2,
        "reth_version": "2.5.2-dev (76a8261)", "base_url": "https://snapshot.example", "components": components}))?;
    args.manifest_sha256 = format!("{:x}", Sha256::digest(&raw));
    fs::write(&args.manifest, &raw)?;
    // The fixture already contains staged files. Record exactly the production identity.
    fs::write(staging.join("manifest.json"), &raw)?;
    download::atomic_json(
        &staging.join("job.json"),
        &json!({"format":1, "manifest_sha256": args.manifest_sha256, "requested_range": [0,3]}),
    )?;
    Ok(Fixture {
        root,
        args,
        payloads,
    })
}

#[test]
fn imports_across_archive_boundaries_and_reopens() -> Result<()> {
    let fixture = fixture(false)?;
    let config = config(&fixture.args)?;
    execute(&config, &fixture.args)?;
    let storage = Storage::open(&config)?;
    for payload in fixture.payloads.clone() {
        let bundle = BaseChain::process_ingest(payload, None)?.0;
        ensure!(
            storage.block_header(bundle.number)? == Some(bundle.header),
            "header changed"
        );
        ensure!(
            storage.block_receipts(bundle.number)? == Some(bundle.receipts),
            "receipts changed"
        );
        ensure!(
            storage.block_transactions(bundle.number)? == Some(bundle.transactions),
            "transaction metadata changed"
        );
    }
    ensure!(
        storage.missing_ranges_in_range(0..=3)?.0.is_empty(),
        "coverage incomplete"
    );
    ensure!(
        execute(&config, &fixture.args).is_err(),
        "existing output was adopted"
    );
    Ok(())
}

#[test]
fn working_space_grows_with_existing_shards() -> Result<()> {
    let fixture = fixture(false)?;
    let mut config = config(&fixture.args)?;
    config.shard_size = 4;
    let storage = Storage::open(&config)?;
    let bundles = fixture
        .payloads
        .iter()
        .cloned()
        .map(|payload| BaseChain::process_ingest(payload, None).map(|result| result.0))
        .collect::<Result<Vec<_>>>()?;
    let batch = &bundles[2..];
    let incoming = working_space(&storage, batch)?;
    let mut previous = incoming;
    for bundle in &bundles[..2] {
        storage.write_block_bundles_wal(std::slice::from_ref(bundle))?;
        storage.compact_all_dirty()?;
        let shard = config.data_dir.join("static/shards/0");
        let mut copied = fs::metadata(shard.join("present.bitset"))?.len();
        for file in fs::read_dir(shard.join("sorted"))? {
            copied += file?.metadata()?.len();
        }
        let budget = working_space(&storage, batch)?;
        ensure!(
            budget > previous,
            "growing shard did not increase the budget"
        );
        ensure!(
            budget == incoming + copied,
            "existing shard files must be budgeted once for a multi-block batch"
        );
        previous = budget;
    }

    let mut unrelated = bundles[0].clone();
    unrelated.number = 8;
    unrelated.header.number = 8;
    storage.write_block_bundles_wal(std::slice::from_ref(&unrelated))?;
    storage.compact_all_dirty()?;
    ensure!(
        working_space(&storage, batch)? == previous,
        "untouched shard was added to the batch budget"
    );
    let other_copy = storage.compaction_copy_bytes([8])?;
    ensure!(other_copy > 0, "second shard fixture has no files");
    let mut spanning = batch.to_vec();
    spanning.push(unrelated.clone());
    ensure!(
        working_space(&storage, &spanning)? == previous + working_space(&storage, &[unrelated])?,
        "batch spanning multiple shards missed existing data"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_parent_traversal_imports_into_the_target_parent() -> Result<()> {
    let fixture = fixture(false)?;
    let data = fixture.root.join("data");
    let disk = fixture.root.join("disk");
    fs::create_dir(&data)?;
    fs::create_dir_all(disk.join("sub"))?;
    std::os::unix::fs::symlink(disk.join("sub"), data.join("link"))?;
    let mut args = fixture.args.clone();
    args.output_dir = Some(data.join("link/../imported"));
    execute(&config(&args)?, &args)?;
    ensure!(
        disk.join("imported/snapshot-import.json").is_file(),
        "import did not follow symlink before parent traversal"
    );
    ensure!(!data.join("imported").exists(), "wrong destination created");

    // The same resolution must protect the staging directory from nested output.
    let staging = args
        .staging_dir
        .as_ref()
        .ok_or_else(|| eyre!("staging missing"))?;
    fs::create_dir(staging.join("sub"))?;
    std::os::unix::fs::symlink(staging.join("sub"), data.join("staging-link"))?;
    args.output_dir = Some(data.join("staging-link/../nested-output"));
    let error = execute(&config(&args)?, &args)
        .err()
        .ok_or_else(|| eyre!("nested output accepted"))?;
    ensure!(
        error.to_string().contains("not nested"),
        "wrong failure: {error}"
    );
    ensure!(
        !staging.join("nested-output").exists(),
        "nested output created"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn directory_resolution_handles_missing_tails_and_rejects_invalid_components() -> Result<()> {
    let fixture = fixture(false)?;
    let root = fs::canonicalize(&fixture.root)?;
    ensure!(
        absolute(&root.join("new/../new/child"))? == root.join("new/child"),
        "missing tail was not normalized"
    );
    ensure!(
        !root.join("new").exists(),
        "path resolution wrote directories"
    );
    std::os::unix::fs::symlink(root.join("missing"), root.join("dangling"))?;
    ensure!(
        absolute(&root.join("dangling/../output")).is_err(),
        "dangling link accepted"
    );
    fs::write(root.join("file"), b"keep this")?;
    ensure!(
        absolute(&root.join("file/../output")).is_err(),
        "file traversal accepted"
    );
    ensure!(
        absolute(Path::new("/../"))? == Path::new("/"),
        "root parent changed"
    );
    Ok(())
}

#[test]
fn invalid_later_block_does_not_create_output() -> Result<()> {
    let fixture = fixture(true)?;
    let error = execute(&config(&fixture.args)?, &fixture.args)
        .err()
        .ok_or_else(|| eyre!("bad receipt root accepted"))?;
    ensure!(
        format!("{error:?}").contains("receipts root mismatch"),
        "wrong failure: {error:?}"
    );
    ensure!(
        !fixture.args.output_dir.as_ref().is_some_and(|p| p.exists()),
        "output created before validation completed"
    );
    Ok(())
}

#[test]
fn manifest_limits_identity_and_dry_run_are_read_only() -> Result<()> {
    let fixture = fixture(false)?;
    let mut args = fixture.args.clone();
    let raw = fs::read(&args.manifest)?;
    let plan = manifest::inspect(&raw, &args)?;
    ensure!(
        plan.chunks.len() == 6 && plan.dependency_range == [0, 3],
        "wrong dependencies"
    );
    ensure!(
        plan.chunks.iter().all(|c| c.component != "state"),
        "state selected"
    );
    args.manifest_sha256 = "0".repeat(64);
    ensure!(
        execute(&config(&args)?, &args).is_err(),
        "wrong SHA accepted"
    );
    args = fixture.args.clone();
    args.max_download_bytes = 1;
    ensure!(execute(&config(&args)?, &args).is_err(), "budget ignored");
    args.dry_run = true;
    execute(&config(&args)?, &args)?;
    ensure!(
        !args.output_dir.as_ref().is_some_and(|p| p.exists()),
        "dry run modified output"
    );
    Ok(())
}

#[test]
fn reconstruction_requires_transaction_root_and_complete_stream() -> Result<()> {
    let mut payload = base_payload(CANYON - 1);
    payload.header.gas_used = payload.receipts[0].cumulative_gas_used();
    payload
        .body
        .transactions
        .push(payload.body.transactions[0].clone());
    payload.receipts.push(payload.receipts[0].clone());
    commit_payload(&mut payload);
    let mut rows = payload
        .body
        .transactions
        .clone()
        .into_iter()
        .zip(payload.receipts.clone());
    let reconstructed = reader::reconstruct(payload.header.clone(), 10, || {
        rows.next().ok_or_else(|| eyre!("truncated"))
    })?;
    ensure!(
        reconstructed.body == payload.body && reconstructed.receipts == payload.receipts,
        "ambiguous gas boundary lost a transaction"
    );
    let mut rows = std::iter::once((
        payload.body.transactions[0].clone(),
        payload.receipts[0].clone(),
    ));
    ensure!(
        reader::reconstruct(payload.header, 10, || rows
            .next()
            .ok_or_else(|| eyre!("truncated")))
        .is_err(),
        "truncated rows accepted"
    );
    Ok(())
}

#[test]
fn reconstruction_rejects_later_forks_gas_and_transaction_limits() -> Result<()> {
    let payload = base_payload(CANYON);
    let error = reader::reconstruct(payload.header, 10, || Err(eyre!("unexpected row read")))
        .err()
        .ok_or_else(|| eyre!("later fork accepted"))?;
    ensure!(error.to_string().contains("pre-Canyon"), "wrong failure");
    let mut payload = base_payload(CANYON - 1);
    payload.header.gas_used = 1;
    ensure!(
        reader::reconstruct(payload.header.clone(), 10, || Ok((
            payload.body.transactions[0].clone(),
            payload.receipts[0].clone()
        )))
        .is_err(),
        "invalid gas accepted"
    );
    ensure!(
        reader::reconstruct(payload.header, 0, || Err(eyre!("unexpected read"))).is_err(),
        "transaction limit ignored"
    );
    Ok(())
}

#[test]
fn cli_selects_base_and_rejects_conflicting_chain() -> Result<()> {
    let args = [
        "shinode",
        "snapshot-import",
        "--manifest",
        "missing.json",
        "--manifest-sha256",
        "abc",
        "--start-block",
        "1",
        "--end-block",
        "2",
        "--dry-run",
    ];
    let config = NodeConfig::try_from_args(args)?;
    ensure!(
        config.chain == ChainKind::Base && config.chain_id == 8453,
        "snapshot chain not normalized"
    );
    let conflict = ["shinode", "--chain", "mainnet"]
        .into_iter()
        .chain(args[1..].iter().copied());
    ensure!(
        NodeConfig::try_from_args(conflict).is_err(),
        "conflicting chain accepted"
    );
    Ok(())
}

#[test]
fn existing_output_and_nested_staging_are_preserved() -> Result<()> {
    let fixture = fixture(false)?;
    let output = fixture
        .args
        .output_dir
        .as_ref()
        .ok_or_else(|| eyre!("output missing"))?;
    fs::create_dir(output)?;
    fs::write(output.join("unrelated"), b"keep this")?;
    ensure!(
        execute(&config(&fixture.args)?, &fixture.args).is_err(),
        "existing output accepted"
    );
    ensure!(
        fs::read(output.join("unrelated"))? == b"keep this",
        "existing data changed"
    );
    let mut args = fixture.args.clone();
    args.output_dir = Some(
        args.staging_dir
            .as_ref()
            .ok_or_else(|| eyre!("staging missing"))?
            .join("nested-output"),
    );
    ensure!(
        execute(&config(&args)?, &args).is_err(),
        "nested output accepted"
    );
    ensure!(
        !args.output_dir.as_ref().is_some_and(|p| p.exists()),
        "nested output created"
    );
    Ok(())
}

#[test]
fn unsupported_producer_and_pruned_metadata_fail_inspection() -> Result<()> {
    let fixture = fixture(false)?;
    for producer in [true, false] {
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.args.manifest)?)?;
        if producer {
            value["reth_version"] = json!("unknown");
        } else {
            value["components"]["receipts"]["chunk_files"] = json!([]);
        }
        let raw = serde_json::to_vec(&value)?;
        let mut args = fixture.args.clone();
        args.manifest_sha256 = format!("{:x}", Sha256::digest(&raw));
        ensure!(
            manifest::inspect(&raw, &args).is_err(),
            "unsupported metadata accepted"
        );
    }
    Ok(())
}
