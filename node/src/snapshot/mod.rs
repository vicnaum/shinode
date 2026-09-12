//! Base history import without execution state or an external Python runtime.
pub mod args;
mod download;
mod manifest;
mod reader;

use crate::{
    chain::{BaseChain, ChainKind},
    cli::NodeConfig,
    storage::Storage,
    sync::historical::IngestChain,
};
use alloy_consensus::TxReceipt;
use args::SnapshotArgs;
use eyre::{ensure, eyre, Result};
use std::{
    fs,
    path::{Component, Path, PathBuf},
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
    ensure!(
        !staging.starts_with(&output) && !output.starts_with(&staging),
        "staging and output directories must be separate, not nested"
    );
    ensure!(
        !output.exists(),
        "snapshot import requires a new output directory; append/resume is not implemented yet"
    );
    Storage::validate_existing(config)?;
    let _lock = download::lock_staging(&staging, &plan, &raw)?;
    download::stage(&staging, &plan, args)?;

    import_verified(config, args, &plan, &staging, &output)
}

fn import_verified(
    config: &NodeConfig,
    args: &SnapshotArgs,
    plan: &manifest::Plan,
    staging: &Path,
    output: &Path,
) -> Result<()> {
    let started = Instant::now();
    download::progress("Validating history from genesis before creating output storage");
    reader::scan(staging, plan, args.max_transactions_per_block, |payload| {
        let number = payload.header.number;
        // Check conversion as well as commitments before allowing any destination writes.
        BaseChain::process_ingest(payload, None)?;
        if number % 10_000 == 0 {
            download::progress(&format!("Validated block {number}/{}", args.end_block));
        }
        Ok(())
    })?;
    fs::create_dir_all(
        output
            .parent()
            .ok_or_else(|| eyre!("invalid output parent"))?,
    )?;
    download::check_space(
        output
            .parent()
            .ok_or_else(|| eyre!("invalid output parent"))?,
        args.min_free_bytes,
        1024 * 1024,
    )?;
    // Atomic directory creation prevents two importers from adopting the same output.
    fs::create_dir(output)?;
    let mut config = config.clone();
    config.data_dir = output.to_path_buf();
    config.peer_cache_dir = None;
    config.command = None;
    config.start_block = args.start_block;
    config.start_block_override = Some(args.start_block);
    config.end_block = Some(args.end_block);
    let storage = Storage::open_with_progress(&config, |_, _| {})?;
    let mut batch = Vec::new();
    let mut imported = 0u64;
    reader::scan(staging, plan, args.max_transactions_per_block, |payload| {
        if payload.header.number < args.start_block {
            return Ok(());
        }
        batch.push(BaseChain::process_ingest(payload, None)?.0);
        if batch.len() >= usize::from(args.batch_blocks) {
            flush(&storage, output, args, &mut batch, &mut imported, started)?;
        }
        Ok(())
    })?;
    flush(&storage, output, args, &mut batch, &mut imported, started)?;
    download::check_space(output, args.min_free_bytes, 0)?;
    storage.compact_all_dirty_with_progress(|_| {})?;
    storage.seal_completed_shards_with_progress(|_| {})?;
    drop(storage);
    let storage = Storage::open_with_progress(&config, |_, _| {})?;
    ensure!(
        storage
            .missing_ranges_in_range(args.start_block..=args.end_block)?
            .0
            .is_empty(),
        "imported range is incomplete after reopen"
    );
    let report = serde_json::json!({"manifest_sha256": plan.manifest_sha256, "chain_id": 8453,
        "genesis": ChainKind::Base.genesis_hash(), "range": [args.start_block, args.end_block],
        "imported_blocks": imported, "storage_bytes": storage.disk_usage()?.total_bytes,
        "elapsed_seconds": started.elapsed().as_secs_f64(), "complete": true});
    download::atomic_json(&output.join("snapshot-import.json"), &report)?;
    download::progress(&format!(
        "Imported {imported} blocks into {}",
        output.display()
    ));
    Ok(())
}

fn flush(
    storage: &Storage,
    output: &Path,
    args: &SnapshotArgs,
    batch: &mut Vec<crate::storage::BlockBundle>,
    imported: &mut u64,
    started: Instant,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    let estimate = working_space(storage, batch)?;
    download::check_space(output, args.min_free_bytes, estimate)?;
    storage.write_block_bundles_wal(batch)?;
    *imported += batch.len() as u64;
    storage.compact_all_dirty_with_progress(|_| {})?;
    storage.seal_completed_shards_with_progress(|_| {})?;
    batch.clear();
    download::progress(&format!(
        "Imported {imported}/{} blocks ({:.1} blocks/s), {} storage bytes",
        args.end_block - args.start_block + 1,
        *imported as f64 / started.elapsed().as_secs_f64().max(f64::EPSILON),
        storage.disk_usage_cached()?.total_bytes,
    ));
    Ok(())
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
mod tests;
