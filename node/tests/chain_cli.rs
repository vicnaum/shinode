//! Check that CLI errors happen before node startup has side effects.

use eyre::{ensure, Result};
use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn invalid_base_range_prevents_storage_creation() -> Result<()> {
    let dir = std::env::temp_dir().join(format!(
        "shinode-cli-guard-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_shinode"))
        .args(["--chain", "base", "--data-dir"])
        .arg(&dir)
        .args(["--start-block", "10", "--end-block", "9"])
        .output()?;
    ensure!(!output.status.success(), "invalid Base range accepted");
    ensure!(
        String::from_utf8_lossy(&output.stderr).contains("--end-block must be at least --start-block"),
        "unexpected error: {:?}",
        output
    );
    ensure!(
        !dir.exists(),
        "Base startup created storage before rejection"
    );
    Ok(())
}

#[test]
fn help_exposes_chain_and_peer_options() -> Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_shinode"))
        .arg("--help")
        .output()?;
    ensure!(output.status.success(), "help failed");
    let help = String::from_utf8(output.stdout)?;
    for flag in [
        "--chain ",
        "--chain-id ",
        "--p2p-port ",
        "--trusted-peer ",
        "--start-block ",
    ] {
        ensure!(help.contains(flag), "missing help flag: {flag}");
    }
    Ok(())
}

fn run(args: &[&str], dir: &std::path::Path) -> Result<std::process::Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_shinode"))
        .arg("--data-dir")
        .arg(dir)
        .args(args)
        .output()?)
}

#[test]
fn maintenance_selects_stored_chain_and_honors_directory_overrides() -> Result<()> {
    let root = std::env::temp_dir().join(format!(
        "shinode-maintenance-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let base = root.join("base");
    let legacy = root.join("legacy");
    for (dir, chain) in [(&base, "base"), (&legacy, "mainnet")] {
        let output = run(
            &["--chain", chain, "--shard-size", "2", "db", "compact"],
            dir,
        )?;
        ensure!(
            output.status.success(),
            "initial maintenance failed: {:?}",
            output
        );
    }
    let path = legacy.join("meta.json");
    let mut meta: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let object = meta
        .as_object_mut()
        .ok_or_else(|| eyre::eyre!("invalid fixture metadata"))?;
    object.remove("genesis_hash");
    object.remove("storage_codec");
    std::fs::write(&path, serde_json::to_vec(&meta)?)?;
    let bogus = root.join("bogus-global");
    std::fs::create_dir_all(&bogus)?;
    std::fs::write(bogus.join("meta.json"), "not json")?;
    for dir in [&base, &legacy] {
        for command in ["stats", "compact", "rebuild-cache"] {
            let output = Command::new(env!("CARGO_BIN_EXE_shinode"))
                .arg("--data-dir")
                .arg(&bogus)
                .args(["db", command, "--data-dir"])
                .arg(dir)
                .output()?;
            ensure!(
                output.status.success(),
                "maintenance {command} failed: {:?}",
                output
            );
        }
        ensure!(run(&["--repair"], dir)?.status.success(), "repair failed");
        let output = run(&["db", "stats", "--json"], dir)?;
        ensure!(output.status.success(), "stats failed");
        let _: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let metadata: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("meta.json"))?)?;
        ensure!(
            metadata["shard_size"] == 2 && metadata["sync_start_block"].is_null(),
            "maintenance changed stored settings"
        );
    }
    ensure!(
        run(&["--chain-id", "8453", "db", "stats"], &base)?
            .status
            .success(),
        "matching ID assertion failed"
    );
    ensure!(
        !run(&["--chain-id", "1", "db", "stats"], &base)?
            .status
            .success(),
        "foreign ID assertion accepted"
    );
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn explicit_maintenance_mismatch_fails_before_logs_or_cache_creation() -> Result<()> {
    let root = std::env::temp_dir().join(format!(
        "shinode-maintenance-reject-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let output = run(&["--chain", "base", "db", "compact"], &root)?;
    ensure!(output.status.success(), "fixture creation failed");
    let metadata = std::fs::read(root.join("meta.json"))?;
    let cache = root.join("must-not-create-cache");
    let logs = root.join("must-not-create-logs");
    for args in [
        vec!["db", "stats"],
        vec!["db", "compact"],
        vec!["db", "rebuild-cache"],
        vec!["--repair"],
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_shinode"));
        command
            .args(["--chain", "mainnet", "--data-dir"])
            .arg(&root)
            .arg("--peer-cache-dir")
            .arg(&cache)
            .args(&args);
        if args == ["db", "compact"] {
            command.arg("--log-json").arg("--log-output-dir").arg(&logs);
        }
        let output = command.output()?;
        ensure!(!output.status.success(), "explicit foreign chain accepted");
        ensure!(
            String::from_utf8_lossy(&output.stderr).contains("does not match stored"),
            "unexpected rejection: {:?}",
            output
        );
        ensure!(
            !cache.exists() && !logs.exists(),
            "rejection created cache/logs"
        );
        ensure!(
            std::fs::read(root.join("meta.json"))? == metadata,
            "rejection rewrote metadata"
        );
    }
    std::fs::remove_dir_all(root)?;
    Ok(())
}
