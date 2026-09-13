use super::{
    tests::{config, fixture, Fixture},
    *,
};
use std::io::Write;

fn output(fixture: &Fixture) -> Result<&Path> {
    fixture
        .args
        .output_dir
        .as_deref()
        .ok_or_else(|| eyre!("missing output"))
}

fn staging(fixture: &Fixture) -> Result<&Path> {
    fixture
        .args
        .staging_dir
        .as_deref()
        .ok_or_else(|| eyre!("missing staging"))
}

fn assert_complete(fixture: &Fixture) -> Result<()> {
    let storage = Storage::open(&config(&fixture.args)?)?;
    for payload in fixture.payloads.clone() {
        ensure!(
            journal::compare(&storage, &BaseChain::process_ingest(payload, None)?.0)?,
            "missing imported block"
        );
    }
    let (_, plan) = manifest::read(&fixture.args)?;
    let checkpoint = journal::Checkpoint::load(output(fixture)?, &plan)?;
    ensure!(
        checkpoint.verified_blocks == 4
            && checkpoint.complete_digest == journal::digest(&storage, &plan)?,
        "incorrect completed checkpoint"
    );
    ensure!(
        storage.aggregate_stats().total_blocks == 4,
        "duplicate block counts"
    );
    let transactions: u64 = fixture
        .payloads
        .iter()
        .map(|p| p.receipts.len() as u64)
        .sum();
    ensure!(
        storage.aggregate_stats().total_transactions == transactions
            && storage.aggregate_stats().total_receipts == transactions,
        "incorrect recovered transaction counts"
    );
    Ok(())
}

#[test]
fn interruptions_reconcile_coverage_and_resume_idempotently() -> Result<()> {
    for point in [
        "after-wal",
        "after-compaction",
        "before-checkpoint",
        "after-checkpoint",
        "after-complete-checkpoint",
    ] {
        let fixture = fixture(false)?;
        INTERRUPT_AT.with(|target| target.replace(Some(point)));
        let error = execute(&config(&fixture.args)?, &fixture.args)
            .err()
            .ok_or_else(|| eyre!("interruption not reached"))?;
        ensure!(
            error.to_string().contains(point),
            "unexpected error: {error:?}"
        );
        // The advisory counter can be stale in either direction; it cannot skip gaps.
        let (_, plan) = manifest::read(&fixture.args)?;
        let mut checkpoint = journal::Checkpoint::load(output(&fixture)?, &plan)?;
        checkpoint.verified_blocks = 99;
        checkpoint.save(output(&fixture)?, &plan)?;
        fs::write(
            journal::Checkpoint::path(output(&fixture)?, &plan).with_extension("json.tmp"),
            b"torn checkpoint",
        )?;
        execute(&config(&fixture.args)?, &fixture.args)?;
        assert_complete(&fixture)?;
        execute(&config(&fixture.args)?, &fixture.args)?;
        assert_complete(&fixture)?;
    }
    Ok(())
}

#[test]
fn overlaps_fill_gaps_and_reject_conflicts_before_new_writes() -> Result<()> {
    for conflict in [false, true] {
        let fixture = fixture(false)?;
        let storage = Storage::open(&config(&fixture.args)?)?;
        let first = BaseChain::process_ingest(fixture.payloads[0].clone(), None)?.0;
        let mut later = BaseChain::process_ingest(fixture.payloads[2].clone(), None)?.0;
        if conflict {
            later.header.extra_data = vec![42].into();
        }
        storage.write_block_bundles_wal(&[first, later.clone()])?;
        storage.compact_all_dirty()?;
        drop(storage);
        let result = execute(&config(&fixture.args)?, &fixture.args);
        if conflict {
            ensure!(
                result
                    .err()
                    .is_some_and(|e| e.to_string().contains("conflicting block hash at 2")),
                "conflict accepted"
            );
            let storage = Storage::open(&config(&fixture.args)?)?;
            ensure!(
                storage.block_header(1)?.is_none() && storage.block_header(3)?.is_none(),
                "new blocks written before checking all overlaps"
            );
            ensure!(
                storage.block_header(2)? == Some(later.header),
                "conflicting block changed"
            );
        } else {
            result?;
            assert_complete(&fixture)?;
        }
    }
    Ok(())
}

#[test]
fn writer_leases_cover_import_normal_storage_repair_and_aliases() -> Result<()> {
    let fixture = fixture(false)?;
    let config = config(&fixture.args)?;
    let storage = Storage::open(&config)?;
    ensure!(Storage::open(&config).is_err(), "second writer accepted");
    ensure!(
        Storage::repair(&config).is_err(),
        "repair bypassed writer lock"
    );
    ensure!(
        execute(&config, &fixture.args).is_err(),
        "import bypassed writer lock"
    );
    #[cfg(unix)]
    {
        let alias = fixture.root.join("alias");
        std::os::unix::fs::symlink(&config.data_dir, &alias)?;
        ensure!(
            DataDirLock::acquire(&alias).is_err(),
            "symlink alias bypassed lock"
        );
    }
    drop(storage);
    execute(&config, &fixture.args)?;
    assert_complete(&fixture)
}

#[test]
fn partial_wal_and_interrupted_compaction_are_recovered() -> Result<()> {
    for phase in ["writing", "swapping", "cleanup"] {
        let fixture = fixture(false)?;
        INTERRUPT_AT.with(|target| {
            target.replace(Some(if phase == "cleanup" {
                "after-compaction"
            } else {
                "after-wal"
            }))
        });
        ensure!(
            execute(&config(&fixture.args)?, &fixture.args).is_err(),
            "interruption missed"
        );
        let shard = output(&fixture)?.join("static/shards/0");
        let meta_path = shard.join("shard.json");
        let mut meta: serde_json::Value = serde_json::from_slice(&fs::read(&meta_path)?)?;
        meta["compaction_phase"] = serde_json::json!(phase);
        meta["total_transactions"] = serde_json::json!(0);
        meta["total_receipts"] = serde_json::json!(0);
        download::atomic_json(&meta_path, &meta)?;
        if phase != "cleanup" {
            fs::create_dir(shard.join("sorted.tmp"))?;
            fs::write(shard.join("sorted.tmp/partial"), b"partial segment")?;
            fs::write(shard.join("bitset.tmp"), [0])?;
            fs::OpenOptions::new()
                .append(true)
                .open(shard.join("state/staging.wal"))?
                .write_all(b"partial WAL")?;
        }
        execute(&config(&fixture.args)?, &fixture.args)?;
        assert_complete(&fixture)?;
        ensure!(
            !shard.join("sorted.tmp").exists(),
            "compaction scratch not recovered"
        );
    }
    Ok(())
}

#[test]
fn checkpoint_identity_mismatch_does_not_change_storage() -> Result<()> {
    let fixture = fixture(false)?;
    execute(&config(&fixture.args)?, &fixture.args)?;
    let (_, plan) = manifest::read(&fixture.args)?;
    let path = journal::Checkpoint::path(output(&fixture)?, &plan);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    value["identity"]["format"] = serde_json::json!(999);
    download::atomic_json(&path, &value)?;
    let before = fs::read(output(&fixture)?.join("meta.json"))?;
    ensure!(
        execute(&config(&fixture.args)?, &fixture.args).is_err(),
        "foreign checkpoint accepted"
    );
    ensure!(
        fs::read(output(&fixture)?.join("meta.json"))? == before,
        "storage changed on mismatch"
    );
    Ok(())
}

fn extracted_inputs(fixture: &Fixture) -> Result<manifest::Plan> {
    let root = staging(fixture)?;
    let (_, plan) = manifest::read(&fixture.args)?;
    for chunk in &plan.chunks {
        let encoder = zstd::stream::write::Encoder::new(Vec::new(), 0)?;
        let mut tar = tar::Builder::new(encoder);
        for file in &chunk.files {
            tar.append_path_with_name(
                manifest::chunk_path(root, chunk).join(&file.path),
                &file.path,
            )?;
        }
        let compressed = tar.into_inner()?.finish()?;
        fs::create_dir_all(root.join("archives"))?;
        fs::write(
            root.join("archives")
                .join(format!("{}.tar.zst", chunk.name)),
            compressed,
        )?;
        fs::remove_dir_all(manifest::chunk_path(root, chunk))?;
    }
    // Extract directly: fixture compressed sizes intentionally describe no HTTP source.
    let ownership = scratch::Ownership::prepare(root, &plan)?;
    for chunk in &plan.chunks {
        let archive = fs::File::open(
            root.join("archives")
                .join(format!("{}.tar.zst", chunk.name)),
        )?;
        let decoder = zstd::stream::read::Decoder::new(archive)?;
        tar::Archive::new(decoder).unpack(manifest::chunk_path(root, chunk))?;
    }
    drop(ownership);
    Ok(plan)
}

#[test]
fn cleanup_resumes_without_inputs_and_preserves_adopted_or_changed_files() -> Result<()> {
    let mut fixture = fixture(false)?;
    fixture.args.keep_scratch = false;
    let plan = extracted_inputs(&fixture)?;
    INTERRUPT_AT.with(|target| target.replace(Some("after-complete-checkpoint")));
    ensure!(
        execute(&config(&fixture.args)?, &fixture.args).is_err(),
        "completion interruption missed"
    );
    let root = staging(&fixture)?;
    let preserved = manifest::chunk_path(root, &plan.chunks[0]).join("user-notes.txt");
    fs::write(&preserved, b"unrelated")?;
    let changed = manifest::chunk_path(root, &plan.chunks[0]).join(&plan.chunks[0].files[0].path);
    fs::write(&changed, b"user replacement")?;
    execute(&config(&fixture.args)?, &fixture.args)?;
    assert_complete(&fixture)?;
    ensure!(
        fs::read(preserved)? == b"unrelated" && fs::read(changed)? == b"user replacement",
        "user files removed"
    );
    ensure!(
        root.join("manifest.json").exists(),
        "source manifest deleted"
    );
    for chunk in &plan.chunks {
        ensure!(
            root.join("archives")
                .join(format!("{}.tar.zst", chunk.name))
                .exists(),
            "adopted archive deleted"
        );
    }
    ensure!(
        !manifest::chunk_path(root, &plan.chunks[1]).exists(),
        "owned inputs retained"
    );
    execute(&config(&fixture.args)?, &fixture.args)?;
    assert_complete(&fixture)
}

#[test]
fn keep_scratch_retains_owned_inputs() -> Result<()> {
    let fixture = fixture(false)?;
    let plan = extracted_inputs(&fixture)?;
    execute(&config(&fixture.args)?, &fixture.args)?;
    for chunk in &plan.chunks {
        ensure!(
            manifest::chunk_path(staging(&fixture)?, chunk).exists(),
            "keep-scratch ignored"
        );
    }
    assert_complete(&fixture)
}

#[test]
fn owned_archives_and_chunks_are_reclaimed_idempotently() -> Result<()> {
    let fixture = fixture(false)?;
    let (_, plan) = manifest::read(&fixture.args)?;
    let root = fixture.root.join("owned-staging");
    fs::create_dir(&root)?;
    let mut ownership = scratch::Ownership::prepare(&root, &plan)?;
    fs::create_dir(root.join("archives"))?;
    let note = root.join("archives/notes.txt");
    fs::write(&note, b"keep this")?;
    for chunk in &plan.chunks {
        fs::write(
            root.join("archives")
                .join(format!("{}.tar.zst", chunk.name)),
            b"owned archive",
        )?;
        for file in &chunk.files {
            let path = manifest::chunk_path(&root, chunk).join(&file.path);
            fs::create_dir_all(path.parent().ok_or_else(|| eyre!("missing parent"))?)?;
            fs::copy(
                manifest::chunk_path(staging(&fixture)?, chunk).join(&file.path),
                path,
            )?;
        }
    }
    ownership.verified(&root, &plan)?;
    fs::remove_file(
        root.join("archives")
            .join(format!("{}.tar.zst", plan.chunks[0].name)),
    )?;
    scratch::Ownership::cleanup(&root, &plan)?;
    scratch::Ownership::cleanup(&root, &plan)?;
    ensure!(
        fs::read(note)? == b"keep this",
        "unrelated archive file removed"
    );
    for chunk in &plan.chunks {
        ensure!(
            !root
                .join("archives")
                .join(format!("{}.tar.zst", chunk.name))
                .exists(),
            "owned archive retained"
        );
        ensure!(
            !manifest::chunk_path(&root, chunk).exists(),
            "owned chunk retained"
        );
    }
    Ok(())
}

#[test]
fn torn_initialization_metadata_can_be_retried() -> Result<()> {
    let fixture = fixture(false)?;
    let root = fixture.root.join("initial-staging");
    fs::create_dir(&root)?;
    let (raw, plan) = manifest::read(&fixture.args)?;
    fs::write(root.join("manifest.json"), &raw)?;
    fs::write(root.join("job.json.tmp"), b"torn job")?;
    drop(download::lock_staging(&root, &plan, &raw)?);
    fs::create_dir(output(&fixture)?)?;
    fs::write(
        output(&fixture)?.join("snapshot-import-init.json.tmp"),
        b"torn initialization",
    )?;
    execute(&config(&fixture.args)?, &fixture.args)?;
    assert_complete(&fixture)
}

#[test]
fn incomplete_extraction_preserves_unrelated_files_and_retries_empty_directories() -> Result<()> {
    let fixture = fixture(false)?;
    let root = staging(&fixture)?;
    let (_, plan) = manifest::read(&fixture.args)?;
    let partial = root.join("partial").join(&plan.chunks[0].name);
    fs::create_dir_all(partial.join("static_files"))?;
    let notes = partial.join("notes.txt");
    fs::write(&notes, b"keep this")?;
    ensure!(
        execute(&config(&fixture.args)?, &fixture.args).is_err(),
        "unrelated partial accepted"
    );
    ensure!(
        fs::read(&notes)? == b"keep this",
        "unrelated partial deleted"
    );
    fs::remove_file(notes)?;
    execute(&config(&fixture.args)?, &fixture.args)?;
    ensure!(
        !partial.exists(),
        "empty interrupted extraction blocked resume"
    );
    assert_complete(&fixture)
}

#[test]
fn completed_checkpoint_cannot_hide_gaps_or_authorize_cleanup_of_changed_output() -> Result<()> {
    let fixture = fixture(false)?;
    execute(&config(&fixture.args)?, &fixture.args)?;
    let storage = Storage::open(&config(&fixture.args)?)?;
    storage.rollback_to(1)?;
    drop(storage);
    execute(&config(&fixture.args)?, &fixture.args)?;
    assert_complete(&fixture)?;
    let (_, plan) = manifest::read(&fixture.args)?;
    let mut checkpoint = journal::Checkpoint::load(output(&fixture)?, &plan)?;
    checkpoint.complete_digest = Some("wrong digest".into());
    checkpoint.save(output(&fixture)?, &plan)?;
    let mut args = fixture.args.clone();
    args.keep_scratch = false;
    ensure!(
        execute(&config(&args)?, &args).is_err(),
        "mismatched completion digest accepted"
    );
    for chunk in &plan.chunks {
        ensure!(
            manifest::chunk_path(staging(&fixture)?, chunk).exists(),
            "input deleted before output verification"
        );
    }
    Ok(())
}
