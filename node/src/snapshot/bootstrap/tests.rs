use super::*;
use crate::snapshot::tests::{config, fixture};

#[test]
fn saved_boundary_resumes_without_network_and_rejects_changed_job() -> Result<()> {
    let fixture = fixture(false)?;
    let mut args = fixture.args.clone();
    args.auto_handoff = true;
    args.checkpoint_rpc = Some("http://127.0.0.1:1".into());
    let stage = args
        .staging_dir
        .as_ref()
        .ok_or_else(|| eyre!("staging missing"))?;
    let path = stage.join("bootstrap.json");
    let boundary = Boundary {
        version: 1,
        manifest_sha256: args.manifest_sha256.clone(),
        start: 0,
        end: 3,
        hash: fixture.payloads[3].header.hash_slow(),
        rpc_sha256: format!("{:x}", Sha256::digest(b"http://127.0.0.1:1")),
        overlap: args.handoff_overlap_blocks,
    };
    download::atomic_json(&path, &boundary)?;
    let before = fs::read(&path)?;
    let resolved = resolve(&config(&args)?, &args, &Progress::default())?;
    ensure!(
        resolved.follow
            && resolved.end_block == 3
            && resolved.checkpoint_hash == Some(boundary.hash),
        "resume moved boundary"
    );
    crate::snapshot::execute(&config(&resolved)?, &resolved)?;
    for changed in [0, 1, 2] {
        let mut changed_args = args.clone();
        match changed {
            0 => changed_args.handoff_overlap_blocks += 1,
            1 => changed_args.checkpoint_rpc = Some("http://127.0.0.1:2".into()),
            _ => changed_args.start_block = 1,
        }
        ensure!(
            resolve(&config(&changed_args)?, &changed_args, &Progress::default()).is_err(),
            "changed job accepted"
        );
        ensure!(
            fs::read(&path)? == before,
            "rejected job moved resume boundary"
        );
    }
    args.max_download_bytes = 1;
    ensure!(
        resolve(&config(&args)?, &args, &Progress::default()).is_err(),
        "automatic selection ignored budget"
    );
    Ok(())
}

#[test]
fn bootstrap_staging_is_exclusive_and_rejects_unrelated_files() -> Result<()> {
    let fixture = fixture(false)?;
    let root = fixture.root.join("bootstrap");
    let lease = lock(&root)?;
    ensure!(lock(&root).is_err(), "second bootstrap writer accepted");
    drop(lease);
    fs::write(root.join("bootstrap.json.tmp"), b"interrupted atomic write")?;
    drop(lock(&root)?);
    fs::write(root.join("unrelated"), b"keep")?;
    ensure!(lock(&root).is_err(), "unrelated staging adopted");
    ensure!(
        fs::read(root.join("unrelated"))? == b"keep",
        "unrelated file changed"
    );
    Ok(())
}

#[test]
fn checkpoint_endpoint_requires_https_or_local_http() -> Result<()> {
    for endpoint in [
        "https://example.com/key",
        "http://127.0.0.1:8545",
        "http://[::1]:8545",
    ] {
        validate_rpc(endpoint)?;
    }
    for endpoint in [
        "http://example.com",
        "https://user:secret@example.com",
        "file:///tmp/rpc",
    ] {
        ensure!(
            validate_rpc(endpoint).is_err(),
            "unsafe checkpoint endpoint accepted"
        );
    }
    ensure!(quantity(&json!("0x2105"))? == 8453, "bad quantity");
    ensure!(
        quantity(&json!(8453)).is_err() && quantity(&json!("invalid")).is_err(),
        "invalid quantity accepted"
    );
    Ok(())
}

#[test]
fn independent_rpc_rejects_wrong_fork_and_authenticates_matching_successor() -> Result<()> {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    let expected = B256::repeat_byte(7);
    let server = std::thread::spawn(move || -> Result<()> {
        for hash in [B256::repeat_byte(9), expected] {
            let (mut stream, _) = listener.accept()?;
            let mut reader = BufReader::new(stream.try_clone()?);
            let mut line = String::new();
            let mut size = 0;
            loop {
                line.clear();
                reader.read_line(&mut line)?;
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    size = value.trim().parse()?;
                }
            }
            let mut bytes = vec![0; size];
            reader.read_exact(&mut bytes)?;
            let request: Value = serde_json::from_slice(&bytes)?;
            ensure!(
                request["method"] == "eth_getBlockByNumber" && request["params"][0] == "0x3",
                "wrong checkpoint request"
            );
            let body =
                json!({"jsonrpc":"2.0","id":1,"result":{"number":"0x3","hash":hash}}).to_string();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )?;
        }
        Ok(())
    });
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let window = || {
        vec![peers::Window {
            first: 2,
            end: 3,
            parent: expected,
        }]
    };
    ensure!(
        verify_windows(&client, &endpoint, window(), &Progress::default()).is_err(),
        "wrong-fork peer accepted"
    );
    ensure!(
        verify_windows(&client, &endpoint, window(), &Progress::default())? == (3, expected),
        "matching checkpoint rejected"
    );
    server
        .join()
        .map_err(|_| eyre!("checkpoint server panic"))??;
    Ok(())
}
