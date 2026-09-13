//! Resolve and persist an automatic archive/P2P boundary before downloading data.
mod peers;
use super::{absolute, args::SnapshotArgs, download, manifest};
use crate::{cli::NodeConfig, ui::archive::Progress};
use alloy_primitives::B256;
use eyre::{ensure, eyre, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path, time::Duration};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    version: u32,
    manifest_sha256: String,
    start: u64,
    end: u64,
    hash: B256,
    rpc_sha256: String,
    overlap: u64,
}

pub(super) fn resolve(
    config: &NodeConfig,
    args: &SnapshotArgs,
    progress: &Progress,
) -> Result<SnapshotArgs> {
    if !args.auto_handoff {
        return Ok(args.clone());
    }
    progress.check()?;
    let endpoint = args
        .checkpoint_rpc
        .as_deref()
        .ok_or_else(|| eyre!("--auto-handoff requires --checkpoint-rpc"))?;
    validate_rpc(endpoint)?;
    let fingerprint = format!("{:x}", Sha256::digest(endpoint.as_bytes()));
    let mut resolved = args.clone();
    resolved.end_block = args.start_block;
    let (_, source) = manifest::read(&resolved)?;
    let staging = absolute(
        args.staging_dir
            .as_deref()
            .ok_or_else(|| eyre!("missing staging directory"))?,
    )?;
    let output = absolute(
        args.output_dir
            .as_deref()
            .ok_or_else(|| eyre!("missing output directory"))?,
    )?;
    ensure!(
        !staging.starts_with(&output) && !output.starts_with(&staging),
        "staging and output must be separate"
    );
    let _lease = lock(&staging)?;
    let path = staging.join("bootstrap.json");
    let boundary = if path.exists() {
        ensure!(
            fs::metadata(&path)?.len() <= 64 * 1024,
            "oversized bootstrap journal"
        );
        let saved: Boundary = serde_json::from_slice(&fs::read(&path)?)?;
        ensure!(
            saved.version == 1
                && saved.manifest_sha256 == args.manifest_sha256
                && saved.start == args.start_block
                && saved.rpc_sha256 == fingerprint
                && saved.overlap == args.handoff_overlap_blocks
                && saved.end >= saved.start
                && saved.end <= source.snapshot_height,
            "bootstrap journal belongs to another job"
        );
        progress.message(&format!(
            "Resuming pinned archive boundary at {}",
            saved.end
        ));
        saved
    } else {
        choose(
            config,
            args,
            source.snapshot_height,
            endpoint,
            fingerprint,
            progress,
        )?
    };
    resolved.end_block = boundary.end;
    resolved.checkpoint_hash = Some(boundary.hash);
    resolved.follow = true;
    // Check the complete selected input budget before committing the automatic choice.
    let (_, plan) = manifest::read(&resolved)?;
    ensure!(plan.compressed_bytes <= args.max_download_bytes && plan.extracted_bytes <= args.max_extracted_bytes,
        "automatic archive selection needs {} compressed / {} extracted bytes; increase explicit download/extraction limits or choose a later start", plan.compressed_bytes, plan.extracted_bytes);
    download::atomic_json(&path, &boundary)?;
    progress.message(&format!(
        "Archive {}..={} → P2P history → live follow",
        boundary.start, boundary.end
    ));
    Ok(resolved)
}

fn lock(root: &Path) -> Result<fs::File> {
    if root.exists() {
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            ensure!(
                !entry.file_type()?.is_symlink(),
                "symlink in bootstrap staging"
            );
            if !root.join("bootstrap.json").exists() {
                ensure!(
                    [".bootstrap.lock", "bootstrap.json.tmp"]
                        .iter()
                        .any(|name| entry.file_name() == *name),
                    "automatic handoff requires fresh or matching bootstrap staging"
                );
            }
        }
    }
    fs::create_dir_all(root)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(".bootstrap.lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock)
        .map_err(|_| eyre!("bootstrap staging is already in use"))?;
    Ok(lock)
}

fn choose(
    config: &NodeConfig,
    args: &SnapshotArgs,
    height: u64,
    endpoint: &str,
    fingerprint: String,
    progress: &Progress,
) -> Result<Boundary> {
    progress.phase("Discovering peer history", 0, "probes");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    ensure!(
        rpc(&client, endpoint, "eth_chainId", &json!([]))? == "0x2105",
        "checkpoint RPC is not Base mainnet"
    );
    let head = rpc(&client, endpoint, "eth_blockNumber", &json!([]))?;
    let head = quantity(&head)?;
    let windows = tokio::runtime::Handle::current().block_on(peers::discover(
        config,
        args.start_block,
        height.min(head.saturating_sub(1)),
        args.handoff_overlap_blocks,
        progress,
    ))?;
    let (end, hash) = verify_windows(&client, endpoint, windows, progress)?;
    Ok(Boundary {
        version: 1,
        manifest_sha256: args.manifest_sha256.clone(),
        start: args.start_block,
        end,
        hash,
        rpc_sha256: fingerprint,
        overlap: args.handoff_overlap_blocks,
    })
}

fn verify_windows(
    client: &reqwest::blocking::Client,
    endpoint: &str,
    windows: Vec<peers::Window>,
    progress: &Progress,
) -> Result<(u64, B256)> {
    for window in windows {
        progress.check()?;
        let block = rpc(
            client,
            endpoint,
            "eth_getBlockByNumber",
            &json!([format!("0x{:x}", window.end), false]),
        )?;
        ensure!(
            quantity(&block["number"])? == window.end,
            "checkpoint RPC returned wrong block number"
        );
        let hash: B256 = serde_json::from_value(block["hash"].clone())?;
        if hash == window.parent {
            progress.message(&format!(
                "Verified peer history begins near {}; archive overlap ends at {}",
                window.first, window.end
            ));
            return Ok((window.end, hash));
        }
    }
    Err(eyre!(
        "no probed archive/P2P boundary matches the independent checkpoint RPC"
    ))
}

fn validate_rpc(endpoint: &str) -> Result<()> {
    let url = reqwest::Url::parse(endpoint)?;
    let local = url
        .host_str()
        .is_some_and(|host| ["localhost", "127.0.0.1", "[::1]"].contains(&host));
    ensure!(
        (url.scheme() == "https" || (url.scheme() == "http" && local))
            && url.has_host()
            && url.username().is_empty()
            && url.password().is_none(),
        "checkpoint RPC must use HTTPS (or local HTTP), without URL credentials"
    );
    Ok(())
}

fn rpc(
    client: &reqwest::blocking::Client,
    endpoint: &str,
    method: &str,
    params: &Value,
) -> Result<Value> {
    let response = client
        .post(endpoint)
        .header("content-type", "application/json")
        .body(json!({"jsonrpc":"2.0", "id":1, "method":method, "params":params}).to_string())
        .send()?
        .error_for_status()?;
    let mut bytes = Vec::new();
    response.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 1024 * 1024,
        "checkpoint RPC response too large"
    );
    let value: Value = serde_json::from_slice(&bytes)?;
    ensure!(
        value["error"].is_null() && !value["result"].is_null(),
        "checkpoint RPC returned an error or missing result"
    );
    Ok(value["result"].clone())
}

fn quantity(value: &Value) -> Result<u64> {
    let text = value
        .as_str()
        .and_then(|s| s.strip_prefix("0x"))
        .ok_or_else(|| eyre!("invalid RPC quantity"))?;
    Ok(u64::from_str_radix(text, 16)?)
}

#[cfg(test)]
#[path = "bootstrap/tests.rs"]
mod tests;
