use super::*;
use crate::{storage::Storage, test_utils};

#[test]
fn chain_selection_resolves_identity_and_defaults() -> Result<()> {
    let mainnet = NodeConfig::try_from_args(["shinode"])?;
    ensure!(
        mainnet.chain == ChainKind::Mainnet && mainnet.chain_id == 1,
        "mainnet default changed"
    );
    ensure!(
        mainnet.genesis_hash == ChainKind::Mainnet.genesis_hash(),
        "wrong mainnet genesis"
    );
    ensure!(
        mainnet.start_block == DEFAULT_START_BLOCK,
        "mainnet range default changed"
    );
    ensure!(
        mainnet.p2p_port == DEFAULT_P2P_PORT && mainnet.trusted_peers.is_empty(),
        "peer defaults changed"
    );
    mainnet.ensure_runtime_supported()?;
    let base = NodeConfig::try_from_args(["shinode", "--chain", "base", "--start-block", "42"])?;
    ensure!(
        base.chain_id == 8453 && base.genesis_hash == ChainKind::Base.genesis_hash(),
        "wrong Base identity"
    );
    ensure!(
        base.start_block == 42 && base.chain_id_override.is_none(),
        "Base resolution failed"
    );
    ensure!(
        base.ensure_runtime_supported().is_err(),
        "Base must remain gated until typed ingestion is ready"
    );
    // Serialized reports contain the resolved values, not raw CLI assertions.
    let json = serde_json::to_value(&base)?;
    ensure!(
        json["chain"] == "base" && json["chain_id"] == 8453 && json["start_block"] == 42,
        "wrong report config"
    );
    ensure!(
        json.get("chain_id_override").is_none(),
        "raw assertion leaked into config report"
    );
    Ok(())
}

#[test]
fn chain_id_is_only_a_compatibility_assertion() -> Result<()> {
    for (chain, id) in [("mainnet", "1"), ("base", "8453")] {
        NodeConfig::try_from_args([
            "shinode",
            "--chain",
            chain,
            "--chain-id",
            id,
            "--start-block",
            "0",
        ])?;
    }
    for args in [
        vec!["shinode", "--chain-id", "8453"],
        vec![
            "shinode",
            "--chain",
            "base",
            "--chain-id",
            "1",
            "--start-block",
            "0",
        ],
        vec!["shinode", "--chain", "optimism"],
        vec!["shinode", "--chain-id", "99"],
    ] {
        ensure!(
            NodeConfig::try_from_args(args).is_err(),
            "accepted conflicting or unsupported chain"
        );
    }
    Ok(())
}

#[test]
fn fresh_base_requires_start_but_maintenance_does_not() -> Result<()> {
    let dir = test_utils::temp_dir("base-cli-fresh");
    let path = dir.to_string_lossy();
    let args = ["shinode", "--chain", "base", "--data-dir", &path];
    let error = NodeConfig::try_from_args(args)
        .err()
        .ok_or_else(|| eyre!("missing start accepted"))?;
    ensure!(
        error.to_string().contains("--start-block is required"),
        "wrong error: {error}"
    );
    for cmd in ["stats", "compact", "rebuild-cache"] {
        let config = NodeConfig::try_from_args([
            "shinode",
            "--chain",
            "base",
            "--data-dir",
            &path,
            "db",
            cmd,
        ])?;
        ensure!(
            config.chain_id == 8453 && config.start_block == 0,
            "maintenance used an ingest default"
        );
    }
    NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        &path,
        "--repair",
    ])?;
    ensure!(
        !dir.exists(),
        "config parsing must not create data directories"
    );
    ensure!(
        NodeConfig::try_from_args(["shinode", "--start-block", "10", "--end-block", "9"]).is_err(),
        "invalid range accepted"
    );
    Ok(())
}

#[test]
fn peer_flags_validate_ports_and_enodes() -> Result<()> {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/base-mainnet-protocol.json"
    ))?;
    let first = fixture["bootnodes"][0]
        .as_str()
        .ok_or_else(|| eyre!("missing bootnode"))?;
    let second = first.replace("3.231.138.188", "archive.example.org");
    let config = NodeConfig::try_from_args([
        "shinode",
        "--p2p-port",
        "30309",
        "--trusted-peer",
        first,
        "--trusted-peer",
        &second,
    ])?;
    ensure!(
        config.p2p_port == 30309 && config.trusted_peers.len() == 2,
        "peer flags lost values"
    );
    for port in ["0", "65536", "-1", "abc"] {
        ensure!(
            NodeConfig::try_from_args(["shinode", "--p2p-port", port]).is_err(),
            "invalid port accepted"
        );
    }
    for peer in [
        "https://example.org",
        "enode://bad@127.0.0.1:30303",
        "not-an-enode",
    ] {
        ensure!(
            NodeConfig::try_from_args(["shinode", "--trusted-peer", peer]).is_err(),
            "invalid peer accepted"
        );
    }
    let zero_port = first.replace(":30301", ":0");
    ensure!(
        NodeConfig::try_from_args(["shinode", "--trusted-peer", &zero_port]).is_err(),
        "zero peer port accepted"
    );
    Ok(())
}

#[test]
fn base_resume_retains_lower_bound_and_rejects_foreign_metadata() -> Result<()> {
    let dir = test_utils::temp_dir("base-cli-resume");
    let path = dir.to_string_lossy();
    let initial = NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        &path,
        "--start-block",
        "42000",
    ])?;
    // Empty storage is sufficient to exercise persisted configuration without
    // introducing Ethereum receipts into a Base directory.
    drop(Storage::open(&initial)?);
    let args = ["shinode", "--chain", "base", "--data-dir", &path];
    ensure!(
        NodeConfig::try_from_args(args)?.start_block == 42000,
        "resume lost original start"
    );
    let lower = NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        &path,
        "--start-block",
        "41000",
    ])?;
    drop(Storage::open(&lower)?);
    let higher = NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        &path,
        "--start-block",
        "43000",
    ])?;
    drop(Storage::open(&higher)?);
    ensure!(
        NodeConfig::try_from_args(args)?.start_block == 41000,
        "resume would skip earlier holes"
    );
    let metadata_path = dir.join("meta.json");
    let original = std::fs::read(&metadata_path)?;
    for field in ["chain_id", "genesis_hash", "schema_version", "storage_codec"] {
        let mut metadata: serde_json::Value = serde_json::from_slice(&original)?;
        metadata[field] = if field == "genesis_hash" {
            serde_json::Value::Null
        } else {
            serde_json::json!(999)
        };
        std::fs::write(&metadata_path, serde_json::to_vec(&metadata)?)?;
        ensure!(
            NodeConfig::try_from_args(args).is_err(),
            "foreign/legacy metadata accepted: {field}"
        );
    }
    std::fs::write(&metadata_path, "invalid json")?;
    ensure!(
        NodeConfig::try_from_args(args).is_err(),
        "corrupt metadata accepted"
    );
    std::fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn legacy_ethereum_metadata_remains_readable() -> Result<()> {
    let dir = test_utils::temp_dir("legacy-chain-config");
    let config = test_utils::base_config(dir.clone());
    drop(Storage::open(&config)?);
    let path = dir.join("meta.json");
    let mut meta: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let object = meta
        .as_object_mut()
        .ok_or_else(|| eyre!("invalid fixture metadata"))?;
    object.remove("genesis_hash");
    object.remove("sync_start_block");
    std::fs::write(&path, serde_json::to_vec(&meta)?)?;
    drop(Storage::open(&config)?);
    std::fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn maintenance_does_not_seed_or_change_a_sync_range() -> Result<()> {
    let dir = test_utils::temp_dir("base-maintenance-range");
    let path = dir.to_string_lossy();
    let maintenance = NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        &path,
        "db",
        "compact",
    ])?;
    drop(Storage::open(&maintenance)?);
    ensure!(
        Storage::read_resume_start(&dir, 8453, ChainKind::Base.genesis_hash())?.is_none(),
        "maintenance seeded an implicit sync start"
    );
    let sync = NodeConfig::try_from_args([
        "shinode",
        "--chain",
        "base",
        "--data-dir",
        &path,
        "--start-block",
        "42000",
    ])?;
    drop(Storage::open(&sync)?);
    drop(Storage::open(&maintenance)?);
    ensure!(
        Storage::read_resume_start(&dir, 8453, ChainKind::Base.genesis_hash())? == Some(42000),
        "maintenance changed the saved range"
    );
    std::fs::remove_dir_all(dir)?;
    Ok(())
}
