//! Local, pinned distribution of complete native history shards.
mod archive;
pub mod args;
mod format;

use self::{
    args::{ExportArgs, RestoreArgs},
    format::{Manifest, Shard, Source},
};
use super::{absolute, download, journal};
use crate::ui::archive::{self as archive_ui, Progress};
use crate::{
    chain::ChainKind,
    cli::NodeConfig,
    storage::{durable, DataDirLock, Storage, StorageCodec, StorageConfigKey},
};
use alloy_primitives::B256;
use eyre::{ensure, eyre, Result};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

pub async fn export(config: NodeConfig, args: ExportArgs) -> Result<()> {
    archive_ui::run(config.no_tui, "History export", move |progress| {
        export_progress(&config, &args, &progress)
    })
    .await
}

pub async fn restore(config: NodeConfig, args: RestoreArgs) -> Result<()> {
    archive_ui::run(config.no_tui, "History restore", move |progress| {
        restore_progress(config, &args, &progress)
    })
    .await
}

fn separate(first: &Path, second: &Path) -> Result<()> {
    ensure!(
        !first.starts_with(second) && !second.starts_with(first),
        "package and data directories must be separate, not nested"
    );
    Ok(())
}

struct Scratch(PathBuf);
impl Scratch {
    fn new(parent: &Path) -> Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        fs::create_dir_all(parent)?;
        for _ in 0..100 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos();
            let path = parent.join(format!(
                ".shinode-package-{}-{now}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        Err(eyre!("cannot allocate private package scratch directory"))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
fn range_digest(storage: &Storage, start: u64, end: u64) -> Result<(String, B256, B256)> {
    range_digest_progress(storage, start, end, &Progress::default())
}

fn range_digest_progress(
    storage: &Storage,
    start: u64,
    end: u64,
    progress: &Progress,
) -> Result<(String, B256, B256)> {
    let mut digest = Sha256::new();
    let mut previous = None;
    let mut first_parent = B256::ZERO;
    for number in start..=end {
        progress.check()?;
        let bundle = journal::stored_bundle(storage, number)?
            .ok_or_else(|| eyre!("package coverage gap at {number}"))?;
        ensure!(
            bundle.header.number == number
                && previous.is_none_or(|hash| bundle.header.parent_hash == hash),
            "package parent mismatch at {number}"
        );
        ensure!(
            bundle.tx_hashes.hashes.len() == bundle.transactions.txs.len()
                && bundle.tx_hashes.hashes.len() == bundle.receipts.len(),
            "incomplete stored payload at {number}"
        );
        if number == start {
            first_parent = bundle.header.parent_hash;
        }
        previous = Some(bundle.header.hash_slow());
        journal::hash_bundle(&mut digest, &bundle)?;
    }
    Ok((
        format!("{:x}", digest.finalize()),
        first_parent,
        previous.ok_or_else(|| eyre!("empty package range"))?,
    ))
}

#[cfg(test)]
fn export_local(config: &NodeConfig, args: &ExportArgs) -> Result<()> {
    export_progress(config, args, &Progress::default())
}

fn export_progress(config: &NodeConfig, args: &ExportArgs, progress: &Progress) -> Result<()> {
    progress.check()?;
    progress.detail(format!(
        "Blocks {}–{} → {}",
        args.start_block,
        args.end_block,
        args.output_dir.display()
    ));
    ensure!(
        config.chain == ChainKind::Base,
        "history packages currently support Base only"
    );
    let output = absolute(&args.output_dir)?;
    let input = absolute(&config.data_dir)?;
    separate(&input, &output)?;
    ensure!(!output.exists(), "export output directory already exists");
    ensure!(
        args.start_block <= args.end_block
            && config.shard_size > 0
            && args.start_block.is_multiple_of(config.shard_size)
            && args
                .end_block
                .checked_add(1)
                .is_some_and(|end| end.is_multiple_of(config.shard_size)),
        "export requires a whole-shard range"
    );
    Storage::validate_existing(config)?;
    let storage = Storage::open_with_progress(config, |_, _| {})?;
    storage.validate_snapshot_boundary()?;
    let boundary = storage
        .snapshot_boundary()
        .ok_or_else(|| eyre!("export requires a trusted snapshot boundary"))?;
    ensure!(
        args.start_block >= boundary.start
            && args.end_block <= storage.last_indexed_block()?.unwrap_or(0),
        "export exceeds contiguous indexed coverage"
    );
    let checkpoint = args
        .checkpoint_hash
        .or_else(|| (args.end_block == boundary.number).then_some(boundary.hash))
        .ok_or_else(|| eyre!("supply an independent --checkpoint-hash for this end block"))?;
    ensure!(
        storage
            .block_header(args.end_block)?
            .is_some_and(|h| h.hash_slow() == checkpoint),
        "export checkpoint mismatch"
    );
    let scratch = Scratch::new(
        output
            .parent()
            .ok_or_else(|| eyre!("invalid output parent"))?,
    )?;
    let mut manifest = Manifest {
        version: 1,
        storage_schema: 2,
        chain_id: config.chain_id,
        genesis: config.genesis_hash,
        codec: StorageCodec::OpV1,
        storage_key: StorageConfigKey::from(config),
        shard_size: config.shard_size,
        start: args.start_block,
        end: args.end_block,
        end_hash: checkpoint,
        source: Source {
            format: "shinode-trusted-boundary-v1".into(),
            manifest_sha256: boundary.manifest_sha256,
            boundary_number: boundary.number,
            boundary_hash: boundary.hash,
        },
        shards: Vec::new(),
    };
    let mut extracted = 0u64;
    let mut compressed = 0u64;
    progress.phase(
        "Verifying / packaging",
        (args.end_block - args.start_block + 1) / config.shard_size,
        "shards",
    );
    for start in (args.start_block..=args.end_block).step_by(usize::try_from(config.shard_size)?) {
        progress.check()?;
        let shard = package_shard(&storage, args, start, &scratch.0, &mut extracted, progress)?;
        compressed += shard.archive_bytes;
        progress.output(compressed);
        progress.advance(1);
        progress.message(&format!("Packaged {}..={}", shard.start, shard.end));
        manifest.shards.push(shard);
    }
    publish_export(&manifest, &scratch, &output, args.min_free_bytes, progress)
}

fn publish_export(
    manifest: &Manifest,
    scratch: &Scratch,
    output: &Path,
    reserve: u64,
    progress: &Progress,
) -> Result<()> {
    progress.check()?;
    progress.phase("Publishing package", 0, "files");
    manifest.validate()?;
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    ensure!(
        bytes.len() <= 8 * 1024 * 1024,
        "manifest exceeds supported size"
    );
    download::check_space(&scratch.0, reserve, bytes.len() as u64)?;
    durable::atomic_write(&scratch.0.join("manifest.json"), &bytes)?;
    durable::sync_tree(&scratch.0)?;
    ensure!(!output.exists(), "export output appeared during packaging");
    fs::rename(&scratch.0, output)?;
    durable::sync_directory(
        output
            .parent()
            .ok_or_else(|| eyre!("missing output parent"))?,
    )?;
    let (compressed, extracted) = manifest.sizes()?;
    progress.message(&format!("Exported {} shards: {extracted} native bytes -> {compressed} archive bytes; manifest SHA-256 {}", manifest.shards.len(), format::sha(&bytes)));
    Ok(())
}

fn package_shard(
    storage: &Storage,
    args: &ExportArgs,
    start: u64,
    directory: &Path,
    extracted: &mut u64,
    progress: &Progress,
) -> Result<Shard> {
    progress.activity(&format!("Verifying shard {start}"));
    let sealed = storage.verified_sealed_shard(start)?;
    let end = start + storage.shard_size() - 1;
    let (bundle_sha256, first_parent, end_hash) =
        range_digest_progress(storage, start, end, progress)?;
    let mut files = Vec::new();
    for (name, path) in &sealed.files {
        let bytes = fs::metadata(path)?.len();
        *extracted = extracted
            .checked_add(bytes)
            .ok_or_else(|| eyre!("size overflow"))?;
        ensure!(
            *extracted <= args.max_extracted_bytes,
            "export exceeds --max-extracted-bytes"
        );
        files.push(format::File {
            name: name.clone(),
            bytes,
            sha256: format::file_sha_progress(path, progress)?,
        });
    }
    let archive = format!("shard-{start}.tar.zst");
    let path = directory.join(&archive);
    progress.activity(&format!("Compressing shard {start}"));
    archive::pack(&sealed.files, &path, args.min_free_bytes, progress)?;
    Ok(Shard {
        start,
        end,
        first_parent,
        end_hash,
        sealed_hash: sealed.hash,
        bundle_sha256,
        archive,
        archive_bytes: fs::metadata(&path)?.len(),
        archive_sha256: format::file_sha_progress(&path, progress)?,
        files,
    })
}

fn read_manifest(args: &RestoreArgs) -> Result<Manifest> {
    ensure!(
        format::valid_sha(&args.manifest_sha256),
        "invalid manifest SHA-256"
    );
    ensure!(
        fs::symlink_metadata(&args.manifest)?.file_type().is_file(),
        "manifest is not a regular file"
    );
    let mut bytes = Vec::new();
    fs::File::open(&args.manifest)?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 8 * 1024 * 1024 && format::sha(&bytes) == args.manifest_sha256,
        "manifest checksum/size mismatch"
    );
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    manifest.validate()?;
    ensure!(
        manifest.end_hash == args.checkpoint_hash,
        "restore checkpoint mismatch"
    );
    let (archives, extracted) = manifest.sizes()?;
    ensure!(
        archives <= args.max_archive_bytes && extracted <= args.max_extracted_bytes,
        "package exceeds archive/extraction limits"
    );
    Ok(manifest)
}

fn validate_destination(config: &NodeConfig, manifest: &Manifest) -> Result<()> {
    Storage::validate_existing(config)?;
    // This runs before staging and again under the writer lease. Opening storage
    // in ingest mode would otherwise persist an earlier start before rejection.
    Storage::validate_snapshot_restore_start(config, manifest.start)?;
    if config.data_dir.exists() && !config.data_dir.join("meta.json").exists() {
        let marker = config.data_dir.join("history-package-init.json");
        if marker.exists() {
            ensure!(
                fs::symlink_metadata(&marker)?.file_type().is_file()
                    && fs::metadata(&marker)?.len() <= 8 * 1024 * 1024,
                "invalid unfinished restore marker"
            );
            let saved: Manifest = serde_json::from_slice(&fs::read(marker)?)?;
            ensure!(
                serde_json::to_value(saved)? == serde_json::to_value(manifest)?,
                "unfinished destination belongs to another history package"
            );
            return Ok(());
        }
        for entry in fs::read_dir(&config.data_dir)? {
            let entry = entry?;
            ensure!(
                entry.file_name() == ".writer.lock"
                    || (entry.file_name() == "history-package-init.json.tmp"
                        && entry.file_type()?.is_file()),
                "refusing unrelated nonempty restore destination"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
fn restore_local(config: NodeConfig, args: &RestoreArgs) -> Result<()> {
    restore_progress(config, args, &Progress::default())
}

fn restore_progress(mut config: NodeConfig, args: &RestoreArgs, progress: &Progress) -> Result<()> {
    progress.check()?;
    let manifest = read_manifest(args)?;
    let (compressed, extracted) = manifest.sizes()?;
    progress.selected(compressed, extracted);
    progress.detail(format!(
        "Blocks {}–{} → {}",
        manifest.start,
        manifest.end,
        args.output_dir.display()
    ));
    ensure!(
        config.chain == ChainKind::Base,
        "restore requires Base identity"
    );
    config.data_dir = absolute(&args.output_dir)?;
    config.shard_size = manifest.shard_size;
    config.start_block = manifest.start;
    config.start_block_override = Some(manifest.start);
    config.end_block = Some(manifest.end);
    config.command = None;
    config.peer_cache_dir = None;
    ensure!(
        StorageConfigKey::from(&config) == manifest.storage_key,
        "package storage configuration mismatch"
    );
    validate_destination(&config, &manifest)?;
    let root = fs::canonicalize(
        args.manifest
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".")),
    )?;
    separate(&root, &config.data_dir)?;
    let scratch = Scratch::new(
        config
            .data_dir
            .parent()
            .ok_or_else(|| eyre!("invalid output parent"))?,
    )?;
    download::check_space(&scratch.0, args.min_free_bytes, manifest.sizes()?.1)?;
    let mut staged_config = config.clone();
    staged_config.data_dir = scratch.0.join("database");
    drop(Storage::open_with_progress(&staged_config, |_, _| {})?);
    progress.phase(
        "Extracting / verifying archives",
        manifest.shards.len() as u64,
        "shards",
    );
    for shard in &manifest.shards {
        progress.check()?;
        archive::unpack(
            &root,
            shard,
            &staged_config
                .data_dir
                .join(format!("static/shards/{}", shard.start)),
            args.min_free_bytes,
            progress,
        )?;
        progress.advance(1);
    }
    let staged = Storage::open_with_progress(&staged_config, |_, _| {})?;
    progress.phase(
        "Verifying native shards",
        manifest.shards.len() as u64,
        "shards",
    );
    for shard in &manifest.shards {
        progress.check()?;
        ensure!(
            staged.verified_sealed_shard(shard.start)?.hash == shard.sealed_hash,
            "shard seal mismatch"
        );
        ensure!(
            range_digest_progress(&staged, shard.start, shard.end, progress)?
                == (
                    shard.bundle_sha256.clone(),
                    shard.first_parent,
                    shard.end_hash
                ),
            "package data/coverage mismatch"
        );
        progress.advance(1);
    }
    staged.pin_snapshot_boundary(
        manifest.start,
        manifest.end,
        manifest.end_hash,
        &args.manifest_sha256,
    )?;
    merge(&config, args, &manifest, &staged, progress)
}

fn merge(
    config: &NodeConfig,
    args: &RestoreArgs,
    manifest: &Manifest,
    staged: &Storage,
    progress: &Progress,
) -> Result<()> {
    // Validation above completes before the destination is created. Existing writers
    // are excluded across recovery, overlap checks, import, reopen and publication.
    progress.check()?;
    progress.phase("Recovering destination", 0, "shards");
    let lease = Arc::new(DataDirLock::acquire(&config.data_dir)?);
    validate_destination(config, manifest)?;
    if !config.data_dir.join("meta.json").exists() {
        // Record ownership before storage creates peer directories or metadata.
        durable::atomic_write(
            &config.data_dir.join("history-package-init.json"),
            &serde_json::to_vec_pretty(manifest)?,
        )?;
    }
    let storage = Storage::open_locked_preserving_resume(config, Arc::clone(&lease), |_, _| {})?;
    let dirty = storage.dirty_shards()?;
    let budget = storage
        .compaction_copy_bytes(dirty.iter().map(|s| s.shard_start))?
        .saturating_add(dirty.iter().fold(0u64, |sum, s| {
            sum.saturating_add(s.wal_bytes.saturating_mul(4))
                .saturating_add(1024 * 1024)
        }));
    download::check_space(&config.data_dir, args.min_free_bytes, budget)?;
    storage.compact_all_dirty_with_progress(|_| {})?;
    progress.output(storage.disk_usage_cached()?.total_bytes);
    if storage.snapshot_boundary().is_some() {
        ensure!(
            manifest.start <= storage.last_indexed_block()?.unwrap_or(0).saturating_add(1),
            "restore would leave a gap before imported history"
        );
        storage.validate_snapshot_boundary()?;
    }
    progress.phase(
        "Checking overlaps",
        manifest.end - manifest.start + 1,
        "blocks",
    );
    for number in manifest.start..=manifest.end {
        progress.check()?;
        progress.position(number - manifest.start);
        journal::compare(
            &storage,
            &journal::stored_bundle(staged, number)?
                .ok_or_else(|| eyre!("missing staged block"))?,
        )?;
    }
    verify_combined_chain(config, &storage, staged, manifest, progress)?;
    // A crash before the first boundary is published must require restore to
    // finish, rather than exposing a partially imported database to normal sync.
    if storage.snapshot_boundary().is_none() {
        durable::atomic_write(
            &config.data_dir.join("history-package-init.json"),
            &serde_json::to_vec_pretty(manifest)?,
        )?;
    }
    progress.phase("Restoring", manifest.end - manifest.start + 1, "blocks");
    let mut batch = Vec::new();
    for number in manifest.start..=manifest.end {
        progress.check()?;
        progress.position(number - manifest.start);
        let bundle =
            journal::stored_bundle(staged, number)?.ok_or_else(|| eyre!("missing staged block"))?;
        if !journal::compare(&storage, &bundle)? {
            batch.push(bundle);
        }
        if batch.len() >= usize::from(args.batch_blocks) {
            flush(&storage, config, args, &mut batch, progress)?;
        }
    }
    flush(&storage, config, args, &mut batch, progress)?;
    storage.recount_stats_in_range(manifest.start, manifest.end)?;
    progress.phase("Sealing restored shards", 0, "shards");
    storage.seal_completed_shards_with_progress(|_| progress.advance(1))?;
    durable::sync_tree(&config.data_dir)?;
    drop(storage);
    finish_restore(config, args, manifest, staged, lease, progress)
}

fn finish_restore(
    config: &NodeConfig,
    args: &RestoreArgs,
    manifest: &Manifest,
    staged: &Storage,
    lease: Arc<DataDirLock>,
    progress: &Progress,
) -> Result<()> {
    // Only adopt the package's lower bound after overlap/parent preflight and
    // durable writes succeed. Keep the same writer lease across this reopen.
    progress.phase(
        "Verifying restored output",
        manifest.end - manifest.start + 1,
        "blocks",
    );
    let storage = Storage::open_locked(config, lease, |_, _| {})?;
    for number in manifest.start..=manifest.end {
        progress.check()?;
        progress.position(number - manifest.start);
        ensure!(
            journal::compare(
                &storage,
                &journal::stored_bundle(staged, number)?
                    .ok_or_else(|| eyre!("missing staged block"))?
            )?,
            "restored output differs"
        );
    }
    progress.check()?;
    progress.phase("Publishing trusted boundary", 0, "items");
    if storage
        .snapshot_boundary()
        .is_none_or(|boundary| manifest.end >= boundary.number)
    {
        storage.pin_snapshot_boundary(
            manifest.start,
            manifest.end,
            manifest.end_hash,
            &args.manifest_sha256,
        )?;
    }
    storage.validate_snapshot_boundary()?;
    durable::atomic_write(
        &config
            .data_dir
            .join(format!("history-package-{}.json", args.manifest_sha256)),
        &serde_json::to_vec_pretty(manifest)?,
    )?;
    progress.message(&format!(
        "Restored and verified {}..={} ({} shards)",
        manifest.start,
        manifest.end,
        manifest.shards.len()
    ));
    Ok(())
}

fn verify_combined_chain(
    config: &NodeConfig,
    storage: &Storage,
    staged: &Storage,
    manifest: &Manifest,
    progress: &Progress,
) -> Result<()> {
    let start = Storage::read_resume_start(&config.data_dir, config.chain_id, config.genesis_hash)?
        .map_or(manifest.start, |saved| saved.min(manifest.start));
    let mut previous = None;
    progress.phase(
        "Checking parent continuity",
        manifest.end - start + 1,
        "blocks",
    );
    for number in start..=manifest.end {
        progress.check()?;
        progress.position(number - start);
        let source = if number >= manifest.start {
            staged
        } else {
            storage
        };
        let header = source
            .block_header(number)?
            .ok_or_else(|| eyre!("restore would leave a coverage gap at {number}"))?;
        ensure!(
            header.number == number && previous.is_none_or(|hash| header.parent_hash == hash),
            "restore parent mismatch at {number}"
        );
        previous = Some(header.hash_slow());
    }
    // Joining an existing suffix must not make an incompatible branch reachable.
    let mut next = manifest.end.checked_add(1);
    while let Some(number) = next {
        progress.check()?;
        let Some(header) = storage.block_header(number)? else {
            break;
        };
        ensure!(
            header.number == number && previous == Some(header.parent_hash),
            "restore parent mismatch in existing suffix at {number}"
        );
        previous = Some(header.hash_slow());
        next = number.checked_add(1);
    }
    Ok(())
}

fn flush(
    storage: &Storage,
    config: &NodeConfig,
    args: &RestoreArgs,
    batch: &mut Vec<crate::storage::BlockBundle>,
    progress: &Progress,
) -> Result<()> {
    if batch.is_empty() {
        return Ok(());
    }
    download::check_space(
        &config.data_dir,
        args.min_free_bytes,
        super::working_space(storage, batch)?,
    )?;
    progress.check()?;
    progress.activity("Writing batch");
    storage.write_block_bundles_wal(batch)?;
    progress.activity("Compacting batch");
    storage.compact_all_dirty_with_progress(|_| {})?;
    progress.activity("");
    progress.output(storage.disk_usage_cached()?.total_bytes);
    progress.message(&format!(
        "Restored batch through block {}",
        batch.last().map_or(0, |b| b.number)
    ));
    batch.clear();
    Ok(())
}

#[cfg(test)]
mod tests;
