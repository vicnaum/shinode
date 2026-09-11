//! Check that CLI errors happen before node startup has side effects.

use eyre::{ensure, Result};
use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn base_runtime_guard_prevents_storage_creation() -> Result<()> {
    let dir = std::env::temp_dir().join(format!(
        "shinode-cli-guard-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    for args in [
        vec!["--start-block", "0"],
        vec!["--repair"],
        vec!["db", "compact"],
        vec!["db", "stats"],
        vec!["db", "rebuild-cache"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_shinode"))
            .args(["--chain", "base", "--data-dir"])
            .arg(&dir)
            .args(args)
            .output()?;
        ensure!(!output.status.success(), "Base runtime incorrectly enabled");
        ensure!(
            String::from_utf8_lossy(&output.stderr).contains("not enabled yet"),
            "unexpected error: {:?}",
            output
        );
        ensure!(
            !dir.exists(),
            "Base startup created storage before rejection"
        );
    }
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
