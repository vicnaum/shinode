use super::*;
use crate::chain::ChainKind;
use eyre::ensure;

fn config(root: PathBuf, chain: ChainKind) -> NodeConfig {
    let mut config = crate::test_utils::base_config(root);
    config.chain = chain;
    config.chain_id = chain.chain_id();
    config.genesis_hash = chain.genesis_hash();
    config.shard_size = 10;
    config
}

fn populate(config: &NodeConfig) -> Result<()> {
    let storage = Storage::open(config)?;
    let bundle = BlockBundle {
        number: 0,
        header: Header::default(),
        tx_hashes: StoredTxHashes { hashes: Vec::new() },
        transactions: StoredTransactions { txs: Vec::new() },
        receipts: match config.chain {
            ChainKind::Mainnet => StoredReceipts::Ethereum(Vec::new()),
            ChainKind::Base => StoredReceipts::Op(Vec::new()),
        },
        size: StoredBlockSize { size: 0 },
    };
    storage.write_block_bundles_wal(&[bundle])?;
    Ok(())
}

// Include empty directories as well as file bytes to detect cache creation/recovery.
type FileSnapshot = Vec<(PathBuf, Option<Vec<u8>>)>;

fn snapshot(root: &Path) -> Result<FileSnapshot> {
    let mut entries = Vec::new();
    if !root.exists() {
        return Ok(entries);
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            entries.push((path.clone(), None));
            entries.extend(snapshot(&path)?);
        } else {
            entries.push((path.clone(), Some(fs::read(path)?)));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

#[test]
fn switching_populated_chains_fails_before_any_mutation() -> Result<()> {
    for chain in [ChainKind::Mainnet, ChainKind::Base] {
        let root = crate::test_utils::temp_dir("identity-switch");
        let initial = config(root.clone(), chain);
        populate(&initial)?;
        let other = if chain == ChainKind::Base {
            ChainKind::Mainnet
        } else {
            ChainKind::Base
        };
        let mut wrong = config(root.clone(), other);
        wrong.peer_cache_dir = Some(root.join("must-not-create-cache"));
        let before = snapshot(&root)?;
        ensure!(Storage::open(&wrong).is_err(), "foreign storage opened");
        ensure!(Storage::repair(&wrong).is_err(), "foreign storage repaired");
        ensure!(
            Storage::read_resume_start(&root, wrong.chain_id, wrong.genesis_hash).is_err(),
            "foreign storage resumed"
        );
        ensure!(
            snapshot(&root)? == before,
            "identity failure changed storage"
        );
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn invalid_metadata_never_reaches_recovery_or_stats() -> Result<()> {
    let root = crate::test_utils::temp_dir("identity-invalid-meta");
    let config = config(root.clone(), ChainKind::Base);
    populate(&config)?;
    let shard_dir = shards_dir(&root).join("0");
    let mut shard = load_shard_meta(&shard_dir)?;
    shard.compaction_phase = Some("writing".to_owned());
    persist_shard_meta(&shard_dir, &shard)?;
    fs::create_dir_all(shard_dir.join("sorted.tmp"))?;
    fs::write(shard_dir.join("sorted.tmp/sentinel"), b"must not recover")?;
    let path = meta_path(&root);
    let original: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    for (field, value) in [
        ("genesis_hash", serde_json::Value::Null),
        (
            "genesis_hash",
            serde_json::json!(ChainKind::Mainnet.genesis_hash()),
        ),
        ("chain_id", serde_json::json!(999)),
        ("storage_codec", serde_json::Value::Null),
        ("storage_codec", serde_json::json!("ethereum_v1")),
        ("storage_codec", serde_json::json!("op_v999")),
        ("schema_version", serde_json::json!(999)),
        ("shard_size", serde_json::json!(0)),
    ] {
        let mut metadata = original.clone();
        metadata[field] = value;
        fs::write(&path, serde_json::to_vec(&metadata)?)?;
        let before = snapshot(&root)?;
        ensure!(
            Storage::open(&config).is_err(),
            "invalid metadata opened: {field}"
        );
        ensure!(
            Storage::repair(&config).is_err(),
            "invalid metadata repaired: {field}"
        );
        ensure!(
            Storage::disk_usage_at(&root).is_err(),
            "invalid metadata accepted by stats: {field}"
        );
        ensure!(
            Storage::read_resume_start(&root, config.chain_id, config.genesis_hash).is_err(),
            "invalid metadata resumed: {field}"
        );
        ensure!(
            snapshot(&root)? == before,
            "metadata failure mutated files: {field}"
        );
    }
    fs::remove_file(&path)?;
    let before = snapshot(&root)?;
    ensure!(
        Storage::open(&config).is_err() && Storage::repair(&config).is_err(),
        "orphan static data adopted"
    );
    ensure!(
        Storage::disk_usage_at(&root).is_err(),
        "orphan static data reported as valid"
    );
    ensure!(snapshot(&root)? == before, "orphan data changed");
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn legacy_ethereum_resumes_and_repairs_but_mislabeled_data_is_rejected() -> Result<()> {
    let root = crate::test_utils::temp_dir("identity-legacy");
    let config = config(root.clone(), ChainKind::Mainnet);
    populate(&config)?;
    let path = meta_path(&root);
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    for field in ["genesis_hash", "storage_codec"] {
        metadata
            .as_object_mut()
            .ok_or_else(|| eyre!("invalid fixture"))?
            .remove(field);
    }
    fs::write(&path, serde_json::to_vec(&metadata)?)?;
    ensure!(
        Storage::read_resume_start(&root, 1, ChainKind::Mainnet.genesis_hash())? == Some(0),
        "legacy Ethereum resume failed"
    );
    Storage::repair(&config)?;
    let storage = Storage::open(&config)?;
    storage.compact_all_dirty()?;
    ensure!(storage.block_header(0)?.is_some(), "legacy data lost");
    drop(storage);
    metadata["chain_id"] = serde_json::json!(8453);
    fs::write(&path, serde_json::to_vec(&metadata)?)?;
    let before = snapshot(&root)?;
    let mut resolved = config.clone();
    ensure!(
        Storage::configure_maintenance(&mut resolved, false).is_err(),
        "mislabeled legacy storage adopted"
    );
    ensure!(
        Storage::repair(&config).is_err(),
        "mislabeled storage repaired"
    );
    ensure!(
        snapshot(&root)? == before,
        "mislabeled legacy storage mutated"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}
