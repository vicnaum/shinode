use super::*;
use std::{net::TcpListener, thread::JoinHandle};

struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Result<Self> {
        let root = crate::test_utils::temp_dir("snapshot-download");
        fs::create_dir_all(root.join("archives"))?;
        fs::create_dir_all(root.join("chunks"))?;
        fs::create_dir_all(root.join("partial"))?;
        Ok(Self(root))
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

type ServerThread = JoinHandle<Result<Vec<String>>>;

fn server(responses: Vec<Vec<u8>>) -> Result<(String, ServerThread)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let url = format!("http://{}/archive", listener.local_addr()?);
    listener.set_nonblocking(true)?;
    let worker = std::thread::spawn(move || {
        let mut requests = Vec::new();
        for response in responses {
            let started = Instant::now();
            let (mut socket, _) = loop {
                match listener.accept() {
                    Ok(value) => break value,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        ensure!(
                            started.elapsed() < Duration::from_secs(10),
                            "test HTTP server timed out"
                        );
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => return Err(error.into()),
                }
            };
            socket.set_nonblocking(false)?;
            socket.set_read_timeout(Some(Duration::from_secs(5)))?;
            let mut raw = Vec::new();
            let mut byte = [0];
            while !raw.ends_with(b"\r\n\r\n") {
                socket.read_exact(&mut byte)?;
                raw.push(byte[0]);
                ensure!(raw.len() < 8192, "oversized test request");
            }
            requests.push(String::from_utf8(raw)?.to_ascii_lowercase());
            socket.write_all(&response)?;
        }
        Ok(requests)
    });
    Ok((url, worker))
}

fn response(status: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    let mut value = format!("HTTP/1.1 {status}\r\nConnection: close\r\n{headers}\r\n").into_bytes();
    value.extend_from_slice(body);
    value
}

fn chunk(url: String, size: u64) -> Chunk {
    Chunk {
        name: "headers-0-1".to_owned(),
        component: "headers".to_owned(),
        start: 0,
        end: 1,
        url,
        compressed: size,
        extracted: 0,
        files: Vec::new(),
    }
}

#[test]
fn truncated_http_transfer_resumes_with_validator() -> Result<()> {
    let temp = Temp::new()?;
    let (url, worker) = server(vec![
        response("200 OK", "Content-Length: 10\r\nETag: \"v1\"\r\n", b"12345"),
        response(
            "206 Partial Content",
            "Content-Length: 5\r\nETag: \"v1\"\r\nContent-Range: bytes 5-9/10\r\n",
            b"67890",
        ),
    ])?;
    let args = crate::snapshot::tests::arguments(&temp.0);
    let budget = Budget::new(&temp.0, &args);
    let path = download(&temp.0, &chunk(url, 10), &args, &budget, &Client::new())?;
    ensure!(fs::read(path)? == b"1234567890", "resume changed bytes");
    let requests = worker.join().map_err(|_| eyre!("server panicked"))??;
    ensure!(
        requests[1].contains("range: bytes=5-") && requests[1].contains("if-range: \"v1\""),
        "missing resume validators"
    );
    Ok(())
}

#[test]
fn ignored_range_restarts_and_changed_partial_is_rejected() -> Result<()> {
    for partial_response in [false, true] {
        let temp = Temp::new()?;
        let reply = if partial_response {
            response(
                "206 Partial Content",
                "Content-Length: 5\r\nETag: \"v2\"\r\nContent-Range: bytes 5-9/10\r\n",
                b"67890",
            )
        } else {
            response(
                "200 OK",
                "Content-Length: 10\r\nETag: \"v2\"\r\n",
                b"1234567890",
            )
        };
        let (url, worker) = server(vec![reply])?;
        let chunk = chunk(url, 10);
        let part = temp.0.join("archives/headers-0-1.tar.zst.part");
        fs::write(&part, b"12345")?;
        atomic_json(
            &temp.0.join("archives/headers-0-1.tar.zst.json"),
            &Partial {
                url: chunk.url.clone(),
                size: 10,
                etag: Some("\"v1\"".to_owned()),
            },
        )?;
        let args = crate::snapshot::tests::arguments(&temp.0);
        let budget = Budget::new(&temp.0, &args);
        let result = download(&temp.0, &chunk, &args, &budget, &Client::new());
        if partial_response {
            ensure!(
                result.is_err() && fs::read(part)? == b"12345",
                "changed source appended"
            );
        } else {
            ensure!(
                fs::read(result?)? == b"1234567890",
                "ignored range was appended"
            );
        }
        worker.join().map_err(|_| eyre!("server panicked"))??;
    }
    Ok(())
}

#[test]
fn bounded_tar_extraction_verifies_and_rejects_corruption() -> Result<()> {
    for corrupt in [false, true] {
        let temp = Temp::new()?;
        let mut chunk = chunk("https://example.invalid/unused".to_owned(), 1);
        let mut tar = tar::Builder::new(Vec::new());
        for suffix in ["", ".conf", ".off"] {
            let path = format!("static_files/static_file_headers_0_1{suffix}");
            let data = b"fixture";
            chunk.files.push(crate::snapshot::manifest::OutputFile {
                path: path.clone(),
                size: data.len() as u64,
                blake3: if corrupt {
                    "0".repeat(64)
                } else {
                    blake3::hash(data).to_hex().to_string()
                },
            });
            chunk.extracted += data.len() as u64;
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o600);
            header.set_cksum();
            tar.append_data(&mut header, path, data.as_slice())?;
        }
        let archive = temp.0.join("archive.zst");
        fs::write(&archive, zstd::encode_all(tar.into_inner()?.as_slice(), 1)?)?;
        let args = crate::snapshot::tests::arguments(&temp.0);
        let budget = Budget::new(&temp.0, &args);
        let result = extract(&temp.0, &chunk, &archive, &budget);
        if corrupt {
            ensure!(
                result.is_err() && !chunk_path(&temp.0, &chunk).exists(),
                "corrupt output published"
            );
        } else {
            result?;
            verify_files(&chunk_path(&temp.0, &chunk), &chunk)?;
        }
    }
    Ok(())
}

#[test]
fn disk_and_transfer_limits_fail_before_writing() -> Result<()> {
    let temp = Temp::new()?;
    let mut args = crate::snapshot::tests::arguments(&temp.0);
    let mut target = File::create(temp.0.join("target"))?;
    args.max_download_bytes = 1;
    let budget = Budget::new(&temp.0, &args);
    ensure!(
        budget.write(&mut target, b"abc", true).is_err(),
        "transfer budget ignored"
    );
    args.min_free_bytes = u64::MAX;
    let budget = Budget::new(&temp.0, &args);
    ensure!(
        budget.write(&mut target, b"abc", false).is_err(),
        "free-space reserve ignored"
    );
    ensure!(
        target.metadata()?.len() == 0,
        "bytes written after budget failure"
    );
    Ok(())
}

#[test]
fn links_duplicates_and_missing_archive_members_never_publish() -> Result<()> {
    for mode in ["link", "duplicate", "missing"] {
        let temp = Temp::new()?;
        let mut chunk = chunk("https://example.invalid/unused".to_owned(), 1);
        let path = "static_files/static_file_headers_0_1";
        chunk.files.push(crate::snapshot::manifest::OutputFile {
            path: path.to_owned(),
            size: 0,
            blake3: blake3::hash(b"").to_hex().to_string(),
        });
        let mut builder = tar::Builder::new(Vec::new());
        if mode != "missing" {
            let mut header = tar::Header::new_gnu();
            header.set_size(0);
            header.set_mode(0o600);
            if mode == "link" {
                header.set_entry_type(tar::EntryType::Symlink);
                header.set_link_name("../../escape")?;
            }
            header.set_cksum();
            builder.append_data(&mut header, path, &b""[..])?;
            if mode == "duplicate" {
                builder.append_data(&mut header, path, &b""[..])?;
            }
        }
        let archive = temp.0.join("bad.zst");
        fs::write(
            &archive,
            zstd::encode_all(builder.into_inner()?.as_slice(), 1)?,
        )?;
        let args = crate::snapshot::tests::arguments(&temp.0);
        let budget = Budget::new(&temp.0, &args);
        ensure!(
            extract(&temp.0, &chunk, &archive, &budget).is_err(),
            "{mode} archive accepted"
        );
        ensure!(!chunk_path(&temp.0, &chunk).exists(), "bad chunk published");
    }
    Ok(())
}

#[test]
fn staging_identity_and_lock_are_enforced() -> Result<()> {
    let root = crate::test_utils::temp_dir("snapshot-lock");
    let temp = Temp(root);
    let plan = crate::snapshot::manifest::Plan {
        manifest_sha256: "fixture".to_owned(),
        snapshot_height: 0,
        producer: String::new(),
        requested_range: [0, 0],
        dependency_range: [0, 0],
        dependency_reason: "fixture",
        supported_format: "fixture",
        compressed_bytes: 0,
        extracted_bytes: 0,
        chunks: Vec::new(),
    };
    let lock = lock_staging(&temp.0, &plan, b"fixture")?;
    ensure!(
        lock_staging(&temp.0, &plan, b"fixture").is_err(),
        "second staging writer accepted"
    );
    drop(lock);
    ensure!(
        lock_staging(&temp.0, &plan, b"changed").is_err(),
        "changed manifest accepted"
    );
    Ok(())
}
