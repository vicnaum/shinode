//! Base history import without execution state or an external Python runtime.
pub mod args;
mod download;
mod journal;
mod manifest;
mod reader;
mod scratch;

use crate::{
    chain::{BaseChain, ChainKind},
    cli::NodeConfig,
    storage::{DataDirLock, Storage},
    sync::historical::IngestChain,
};
use alloy_consensus::TxReceipt;
use args::SnapshotArgs;
use eyre::{ensure, eyre, Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::Instant,
};

pub async fn run(config: NodeConfig, args: SnapshotArgs) -> Result<()> {
    tokio::task::spawn_blocking(move || execute(&config, &args)).await?
}

fn execute(config: &NodeConfig, args: &SnapshotArgs) -> Result<()> {
    ensure!(
        args.chain == ChainKind::Base && config.chain == ChainKind::Base,
        "snapshot import currently supports Base mainnet only"
    );
    let (raw, plan) = manifest::read(args)?;
    if args.dry_run {
        return print_plan(&plan);
    }
    ensure!(
        plan.compressed_bytes <= args.max_download_bytes,
        "selected archives exceed --max-download-bytes; inspect the genesis-prefix dependency"
    );
    ensure!(
        plan.extracted_bytes <= args.max_extracted_bytes,
        "selected files exceed --max-extracted-bytes"
    );
    manifest::validate_layout(&plan)?;
    ensure!(config.shard_size > 0, "--shard-size must be nonzero");
    let staging = args
        .staging_dir
        .as_ref()
        .ok_or_else(|| eyre!("missing --staging-dir"))?;
    let output = args
        .output_dir
        .as_ref()
        .ok_or_else(|| eyre!("missing --output-dir"))?;
    let staging = absolute(staging)?;
    let output = absolute(output)?;
    let mut normalized_args = args.clone();
    normalized_args.output_dir = Some(output.clone());
    let args = &normalized_args;
    ensure!(
        !staging.starts_with(&output) && !output.starts_with(&staging),
        "staging and output directories must be separate, not nested"
    );
    let mut config = config.clone();
    config.data_dir.clone_from(&output);
    config.peer_cache_dir = None;
    config.command = None;
    config.start_block = args.start_block;
    config.start_block_override = Some(args.start_block);
    config.end_block = Some(args.end_block);
    validate_output(&config, &plan)?;
    let _staging_lock = download::lock_staging(&staging, &plan, &raw)?;
    let lease = if output.exists() {
        Some(Arc::new(DataDirLock::acquire(&output)?))
    } else {
        None
    };
    if let Some(lease) = &lease {
        if resume_complete(&config, args, &plan, lease)? {
            if !args.keep_scratch {
                scratch::Ownership::cleanup(&staging, &plan)?;
            }
            return Ok(());
        }
    }
    download::stage(&staging, &plan, args)?;
    import_verified(&config, args, &plan, &staging, lease)
}

fn validate_output(config: &NodeConfig, plan: &manifest::Plan) -> Result<()> {
    Storage::validate_existing(config)?;
    let output = &config.data_dir;
    if !output.exists() || output.join("meta.json").exists() {
        return Ok(());
    }
    let marker = output.join("snapshot-import-init.json");
    if marker.exists() {
        ensure!(
            serde_json::from_slice::<journal::Identity>(&fs::read(marker)?)?
                == journal::Identity::new(plan),
            "unfinished destination belongs to another import"
        );
    } else {
        for entry in fs::read_dir(output)? {
            let name = entry?.file_name();
            ensure!(
                name == ".writer.lock" || name == "snapshot-import-init.json.tmp",
                "refusing to adopt unrelated output directory"
            );
        }
    }
    Ok(())
}

fn recover(storage: &Storage, output: &Path, args: &SnapshotArgs) -> Result<()> {
    let dirty = storage.dirty_shards()?;
    if dirty.is_empty() {
        return storage.recount_stats_in_range(args.start_block, args.end_block);
    }
    let existing = storage.compaction_copy_bytes(dirty.iter().map(|shard| shard.shard_start))?;
    let budget = dirty.iter().fold(existing, |total, shard| {
        total
            .saturating_add(shard.wal_bytes.saturating_mul(4))
            .saturating_add(1024 * 1024)
    });
    download::check_space(output, args.min_free_bytes, budget)?;
    storage.compact_all_dirty_with_progress(|_| {})?;
    storage.recount_stats_in_range(args.start_block, args.end_block)?;
    storage.seal_completed_shards_with_progress(|_| {})
}

fn resume_complete(
    config: &NodeConfig,
    args: &SnapshotArgs,
    plan: &manifest::Plan,
    lease: &Arc<DataDirLock>,
) -> Result<bool> {
    let mut checkpoint = journal::Checkpoint::load(&config.data_dir, plan)?;
    let Some(expected) = checkpoint.complete_digest.clone() else {
        return Ok(false);
    };
    let storage = Storage::open_locked(config, Arc::clone(lease), |_, _| {})?;
    recover(&storage, &config.data_dir, args)?;
    let Some(actual) = journal::digest(&storage, plan)? else {
        return Ok(false);
    };
    ensure!(
        actual == expected,
        "completed import differs from durable storage; refusing cleanup"
    );
    crate::storage::durable::sync_tree(&config.data_dir)?;
    checkpoint.reconcile(&storage, plan)?;
    checkpoint.complete_digest = Some(actual);
    checkpoint.save(&config.data_dir, plan)?;
    report(&storage, &config.data_dir, plan, 0.0)?;
    download::progress("Requested range is already complete and verified; no inputs needed");
    Ok(true)
}

fn import_verified(
    config: &NodeConfig,
    args: &SnapshotArgs,
    plan: &manifest::Plan,
    staging: &Path,
    lease: Option<Arc<DataDirLock>>,
) -> Result<()> {
    let started = Instant::now();
    let output = &config.data_dir;
    download::progress("Validating history from genesis before writing output storage");
    let mut expected = Sha256::new();
    reader::scan(staging, plan, args.max_transactions_per_block, |payload| {
        let bundle = BaseChain::process_ingest(payload, None)?.0;
        if bundle.number >= args.start_block {
            journal::hash_bundle(&mut expected, &bundle)?;
        }
        if bundle.number % 10_000 == 0 {
            download::progress(&format!(
                "Validated block {}/{}",
                bundle.number, args.end_block
            ));
        }
        Ok(())
    })?;
    let expected = format!("{:x}", expected.finalize());
    let parent = output
        .parent()
        .ok_or_else(|| eyre!("invalid output parent"))?;
    fs::create_dir_all(parent)?;
    download::check_space(parent, args.min_free_bytes, 1024 * 1024)?;
    let lease = match lease {
        Some(lease) => lease,
        None => Arc::new(DataDirLock::acquire(output)?),
    };
    validate_output(config, plan)?;
    download::atomic_json(
        &output.join("snapshot-import-init.json"),
        &journal::Identity::new(plan),
    )?;
    crate::storage::durable::sync_directory(parent)?;
    let storage = Storage::open_locked(config, Arc::clone(&lease), |_, _| {})?;
    recover(&storage, output, args)?;
    // Check every overlap before adding any new block from this job.
    reader::scan(staging, plan, args.max_transactions_per_block, |payload| {
        if payload.header.number >= args.start_block {
            journal::compare(&storage, &BaseChain::process_ingest(payload, None)?.0)?;
        }
        Ok(())
    })?;
    let mut checkpoint = journal::Checkpoint::load(output, plan)?;
    checkpoint.reconcile(&storage, plan)?;
    checkpoint.save(output, plan)?;
    let mut batch = Vec::new();
    reader::scan(staging, plan, args.max_transactions_per_block, |payload| {
        if payload.header.number < args.start_block {
            return Ok(());
        }
        let bundle = BaseChain::process_ingest(payload, None)?.0;
        if !journal::compare(&storage, &bundle)? {
            batch.push(bundle);
        }
        if batch.len() >= usize::from(args.batch_blocks) {
            flush(&storage, args, plan, &mut batch, &mut checkpoint, started)?;
        }
        Ok(())
    })?;
    flush(&storage, args, plan, &mut batch, &mut checkpoint, started)?;
    crate::storage::durable::sync_tree(output)?;
    drop(storage);
    let storage = Storage::open_locked(config, Arc::clone(&lease), |_, _| {})?;
    ensure!(
        journal::digest(&storage, plan)?.as_ref() == Some(&expected),
        "imported data differs after reopen or range is incomplete"
    );
    checkpoint.reconcile(&storage, plan)?;
    checkpoint.complete_digest = Some(expected);
    checkpoint.save(output, plan)?;
    failpoint("after-complete-checkpoint")?;
    report(&storage, output, plan, started.elapsed().as_secs_f64())?;
    if !args.keep_scratch {
        scratch::Ownership::cleanup(staging, plan)?;
    }
    download::progress(&format!(
        "Verified {} blocks in {}",
        checkpoint.verified_blocks,
        output.display()
    ));
    Ok(())
}

fn report(storage: &Storage, output: &Path, plan: &manifest::Plan, elapsed: f64) -> Result<()> {
    let [start, end] = plan.requested_range;
    let report = serde_json::json!({"manifest_sha256": plan.manifest_sha256, "chain_id": 8453,
        "genesis": ChainKind::Base.genesis_hash(), "range": [start, end],
        "imported_blocks": end - start + 1, "storage_bytes": storage.disk_usage()?.total_bytes,
        "elapsed_seconds": elapsed, "complete": true});
    download::atomic_json(&output.join("snapshot-import.json"), &report)
}

fn flush(
    storage: &Storage,
    args: &SnapshotArgs,
    plan: &manifest::Plan,
    batch: &mut Vec<crate::storage::BlockBundle>,
    checkpoint: &mut journal::Checkpoint,
    started: Instant,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let output = args
        .output_dir
        .as_ref()
        .ok_or_else(|| eyre!("missing output"))?;
    let estimate = working_space(storage, batch)?;
    download::check_space(output, args.min_free_bytes, estimate)?;
    storage.write_block_bundles_wal(batch)?;
    failpoint("after-wal")?;
    storage.compact_all_dirty_with_progress(|_| {})?;
    failpoint("after-compaction")?;
    storage.seal_completed_shards_with_progress(|_| {})?;
    for bundle in batch.iter() {
        ensure!(
            journal::compare(storage, bundle)?,
            "missing compacted block"
        );
    }
    checkpoint.reconcile(storage, plan)?;
    failpoint("before-checkpoint")?;
    checkpoint.save(output, plan)?;
    failpoint("after-checkpoint")?;
    batch.clear();
    download::progress(&format!(
        "Verified {}/{} blocks ({:.1} blocks/s), {} storage bytes",
        checkpoint.verified_blocks,
        args.end_block - args.start_block + 1,
        checkpoint.verified_blocks as f64 / started.elapsed().as_secs_f64().max(f64::EPSILON),
        storage.disk_usage_cached()?.total_bytes,
    ));
    Ok(())
}

#[cfg(not(test))]
const fn failpoint(_point: &str) -> Result<()> {
    Ok(())
}

#[cfg(test)]
thread_local! { static INTERRUPT_AT: std::cell::RefCell<Option<&'static str>> = const { std::cell::RefCell::new(None) }; }

#[cfg(test)]
fn failpoint(point: &str) -> Result<()> {
    INTERRUPT_AT.with(|target| {
        if target
            .borrow()
            .as_ref()
            .is_some_and(|target| *target == point)
        {
            target.replace(None);
            return Err(eyre!("interrupted at {point}"));
        }
        Ok(())
    })
}

fn working_space(storage: &Storage, batch: &[crate::storage::BlockBundle]) -> Result<u64> {
    let incoming = batch
        .iter()
        .map(|bundle| {
            let receipt_bytes = bundle.receipts.iter().fold(0u64, |sum, receipt| {
                receipt
                    .logs()
                    .iter()
                    .fold(sum.saturating_add(1024), |sum, log| {
                        sum.saturating_add(log.data.data.len() as u64)
                            .saturating_add(log.topics().len() as u64 * 32)
                    })
            });
            bundle
                .size
                .size
                .saturating_add(receipt_bytes)
                .saturating_mul(4)
                .saturating_add(4096)
        })
        .fold(0u64, u64::saturating_add);
    // sorted.tmp retains a full copy of the old segments until the atomic swap.
    // Count each affected shard once, even when the batch contains multiple rows.
    let existing = storage.compaction_copy_bytes(batch.iter().map(|bundle| bundle.number))?;
    Ok(incoming.saturating_add(existing))
}

fn absolute(path: &Path) -> Result<PathBuf> {
    let full = std::env::current_dir()?.join(path);
    let mut resolved = PathBuf::new();
    for component in full.components() {
        match component {
            Component::ParentDir => {
                // Existing symlinks have already been resolved, so this traverses
                // the target's parent. At a filesystem root, `..` stays at root.
                resolved.pop();
            }
            Component::CurDir => {}
            Component::Normal(value) => {
                resolved.push(value);
                match fs::symlink_metadata(&resolved) {
                    Ok(_) => {
                        // A dangling symlink must fail rather than become a new
                        // destination, and a file cannot be a directory component.
                        resolved = fs::canonicalize(&resolved)?;
                        ensure!(
                            resolved.is_dir(),
                            "snapshot path is not a directory: {}",
                            resolved.display()
                        );
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
            value => resolved.push(value),
        }
    }
    Ok(resolved)
}

#[expect(clippy::print_stdout, reason = "read-only snapshot inspection JSON")]
fn print_plan(plan: &manifest::Plan) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(plan)?);
    Ok(())
}

#[cfg(test)]
mod resume_tests;
#[cfg(test)]
mod tests;
