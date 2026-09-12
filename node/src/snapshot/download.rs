use super::{
    args::SnapshotArgs,
    manifest::{chunk_path, Chunk, Plan},
};
use eyre::{ensure, eyre, Result, WrapErr};
use parking_lot::Mutex;
use reqwest::{blocking::Client, header, StatusCode};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};

const BUFFER: usize = 64 * 1024;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
struct Identity {
    format: u32,
    manifest_sha256: String,
    requested_range: [u64; 2],
}

#[derive(Deserialize, Serialize)]
struct Partial {
    url: String,
    size: u64,
    etag: Option<String>,
}

pub struct Budget<'a> {
    root: &'a Path,
    reserve: u64,
    limit: u64,
    transferred: Mutex<u64>,
    cancelled: AtomicBool,
}

impl<'a> Budget<'a> {
    const fn new(root: &'a Path, args: &SnapshotArgs) -> Self {
        Self {
            root,
            reserve: args.min_free_bytes,
            limit: args.max_download_bytes,
            transferred: Mutex::new(0),
            cancelled: AtomicBool::new(false),
        }
    }

    fn check(&self, amount: u64) -> Result<()> {
        ensure!(!self.cancelled.load(Ordering::Relaxed), "staging cancelled");
        check_space(self.root, self.reserve, amount)
    }

    fn write(&self, target: &mut File, bytes: &[u8], network: bool) -> Result<()> {
        let mut transferred = self.transferred.lock();
        self.check(bytes.len() as u64)?;
        if network {
            let next = transferred
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| eyre!("transfer size overflow"))?;
            ensure!(
                next <= self.limit,
                "per-run transfer budget exceeded, including retries"
            );
            *transferred = next;
        }
        target.write_all(bytes)?;
        Ok(())
    }
}

pub fn check_space(path: &Path, reserve: u64, amount: u64) -> Result<()> {
    ensure!(
        fs2::available_space(path)?
            >= reserve
                .checked_add(amount)
                .ok_or_else(|| eyre!("disk budget overflow"))?,
        "insufficient free disk space at {}",
        path.display()
    );
    Ok(())
}

pub fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let temporary = path.with_extension("json.tmp");
    let mut file = File::create(&temporary)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn reject_symlinks(root: &Path) -> Result<()> {
    ensure!(
        !fs::symlink_metadata(root)?.file_type().is_symlink(),
        "symlink in staging: {}",
        root.display()
    );
    if root.is_dir() {
        for entry in fs::read_dir(root)? {
            reject_symlinks(&entry?.path())?;
        }
    }
    Ok(())
}

pub fn lock_staging(root: &Path, plan: &Plan, raw: &[u8]) -> Result<File> {
    if root.exists() {
        reject_symlinks(root)?;
    }
    fs::create_dir_all(root)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join(".lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock).wrap_err("snapshot staging is already in use")?;
    let identity = Identity {
        format: 1,
        manifest_sha256: plan.manifest_sha256.clone(),
        requested_range: plan.requested_range,
    };
    let marker = root.join("job.json");
    if marker.exists() {
        ensure!(
            serde_json::from_slice::<Identity>(&fs::read(marker)?)? == identity,
            "staging belongs to another manifest/range"
        );
        ensure!(
            fs::read(root.join("manifest.json"))? == raw,
            "staged manifest changed"
        );
    } else {
        for entry in fs::read_dir(root)? {
            ensure!(
                entry?.file_name() == ".lock",
                "refusing to adopt nonempty staging directory"
            );
        }
        fs::write(root.join("manifest.json"), raw)?;
        atomic_json(&marker, &identity)?;
    }
    for folder in ["archives", "partial", "chunks"] {
        fs::create_dir_all(root.join(folder))?;
    }
    Ok(lock)
}

pub fn stage(root: &Path, plan: &Plan, args: &SnapshotArgs) -> Result<()> {
    progress(&format!(
        "Input: {} compressed / {} extracted bytes; limits: {} / {}, free reserve: {} bytes",
        plan.compressed_bytes,
        plan.extracted_bytes,
        args.max_download_bytes,
        args.max_extracted_bytes,
        args.min_free_bytes
    ));
    let budget = Budget::new(root, args);
    atomic_json(
        &root.join("download-report.json"),
        &serde_json::json!({"complete": false, "manifest_sha256": plan.manifest_sha256}),
    )?;
    let mut remaining = 0u64;
    for chunk in &plan.chunks {
        let temp = root.join("partial").join(&chunk.name);
        if temp.exists() {
            fs::remove_dir_all(temp)?;
        }
        if !chunk_path(root, chunk).exists() {
            remaining = remaining
                .checked_add(chunk.extracted)
                .ok_or_else(|| eyre!("disk size overflow"))?;
            let archive = root
                .join("archives")
                .join(format!("{}.tar.zst", chunk.name));
            if !archive.exists() {
                let part = archive.with_extension("zst.part");
                let present = if part.exists() {
                    fs::metadata(part)?.len()
                } else {
                    0
                };
                remaining = remaining
                    .checked_add(chunk.compressed.saturating_sub(present))
                    .ok_or_else(|| eyre!("disk size overflow"))?;
            }
        }
    }
    budget.check(remaining.saturating_add(1024 * 1024))?;
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(args.timeout_seconds))
        .build()?;
    let next = AtomicUsize::new(0);
    let started = Instant::now();
    let results = std::thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..args.concurrency {
            let (budget, next, client) = (&budget, &next, &client);
            workers.push(scope.spawn(move || -> Result<()> {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(chunk) = plan.chunks.get(index) else {
                        return Ok(());
                    };
                    let result = stage_chunk(root, chunk, args, budget, client);
                    if result.is_err() {
                        budget.cancelled.store(true, Ordering::Relaxed);
                    }
                    result?;
                }
            }));
        }
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .unwrap_or_else(|_| Err(eyre!("snapshot worker panicked")))
            })
            .collect::<Vec<_>>()
    });
    let mut failures = results
        .into_iter()
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    if !failures.is_empty() {
        let index = failures
            .iter()
            .position(|e| e.to_string() != "staging cancelled")
            .unwrap_or(0);
        return Err(failures.swap_remove(index));
    }
    atomic_json(
        &root.join("download-report.json"),
        &serde_json::json!({
            "complete": true, "manifest_sha256": plan.manifest_sha256,
            "requested_range": plan.requested_range, "dependency_range": plan.dependency_range,
            "compressed_bytes": plan.compressed_bytes, "extracted_bytes": plan.extracted_bytes,
            "transferred_bytes_this_run": *budget.transferred.lock(), "elapsed_seconds": started.elapsed().as_secs_f64(),
        }),
    )?;
    Ok(())
}

fn stage_chunk(
    root: &Path,
    chunk: &Chunk,
    args: &SnapshotArgs,
    budget: &Budget<'_>,
    client: &Client,
) -> Result<()> {
    budget.check(0)?;
    if chunk_path(root, chunk).exists() {
        verify_files(&chunk_path(root, chunk), chunk)?;
    } else {
        ensure!(
            !args.offline,
            "offline staging is missing verified chunk {}",
            chunk.name
        );
        progress(&format!(
            "Downloading {} ({} bytes)",
            chunk.name, chunk.compressed
        ));
        let archive = download(root, chunk, args, budget, client)?;
        extract(root, chunk, &archive, budget)?;
    }
    progress(&format!(
        "Verified {}; transferred {} bytes this run",
        chunk.name,
        *budget.transferred.lock()
    ));
    Ok(())
}

enum Attempt {
    Complete,
    Retry(String),
}

fn download(
    root: &Path,
    chunk: &Chunk,
    args: &SnapshotArgs,
    budget: &Budget<'_>,
    client: &Client,
) -> Result<std::path::PathBuf> {
    let archive = root
        .join("archives")
        .join(format!("{}.tar.zst", chunk.name));
    if archive.exists() {
        ensure!(
            fs::metadata(&archive)?.len() == chunk.compressed,
            "cached archive has wrong size"
        );
        return Ok(archive);
    }
    for attempt in 0..=args.retries {
        budget.check(0)?;
        match attempt_download(&archive, chunk, budget, client)? {
            Attempt::Complete => return Ok(archive),
            Attempt::Retry(reason) => {
                ensure!(
                    attempt < args.retries,
                    "{}: download retries exhausted: {reason}",
                    chunk.name
                );
                progress(&format!("Retrying {}: {reason}", chunk.name));
                std::thread::sleep(Duration::from_secs(1u64 << attempt.min(3)));
            }
        }
    }
    Err(eyre!("download did not complete"))
}

fn attempt_download(
    archive: &Path,
    chunk: &Chunk,
    budget: &Budget<'_>,
    client: &Client,
) -> Result<Attempt> {
    let part = archive.with_extension("zst.part");
    let metadata = archive.with_extension("zst.json");
    let saved = if metadata.exists() {
        Some(serde_json::from_slice::<Partial>(&fs::read(&metadata)?)?)
    } else {
        None
    };
    if let Some(saved) = &saved {
        ensure!(saved.url == chunk.url, "partial URL differs from manifest");
        ensure!(
            saved.size == chunk.compressed,
            "partial size differs from manifest"
        );
    }
    let mut offset = if part.exists() {
        fs::metadata(&part)?.len()
    } else {
        0
    };
    ensure!(
        offset <= chunk.compressed,
        "partial archive exceeds expected size"
    );
    if offset == chunk.compressed {
        fs::rename(part, archive)?;
        return Ok(Attempt::Complete);
    }
    let etag = saved
        .and_then(|p| p.etag)
        .filter(|v| v.starts_with('"') && v.ends_with('"'));
    if offset > 0 && etag.is_none() {
        fs::remove_file(&part)?;
        offset = 0;
    }
    let mut request = client
        .get(&chunk.url)
        .header(header::ACCEPT_ENCODING, "identity");
    if let Some(etag) = &etag {
        if offset > 0 {
            request = request
                .header(header::RANGE, format!("bytes={offset}-"))
                .header(header::IF_RANGE, etag);
        }
    }
    let mut response = match request.send() {
        Ok(response) => response,
        Err(error) => return Ok(Attempt::Retry(error.to_string())),
    };
    let status = response.status();
    if status == StatusCode::RANGE_NOT_SATISFIABLE {
        if part.exists() {
            fs::remove_file(part)?;
        }
        if metadata.exists() {
            fs::remove_file(metadata)?;
        }
        return Ok(Attempt::Retry("server refused byte range".to_owned()));
    }
    if status.is_server_error()
        || status == StatusCode::TOO_MANY_REQUESTS
        || status == StatusCode::REQUEST_TIMEOUT
    {
        return Ok(Attempt::Retry(format!("HTTP {status}")));
    }
    let (offset, current) = validate_response(&response, chunk, offset, etag.as_deref())?;
    let mut target = OpenOptions::new()
        .create(true)
        .write(true)
        .append(offset > 0)
        .truncate(offset == 0)
        .open(&part)?;
    atomic_json(
        &metadata,
        &Partial {
            url: chunk.url.clone(),
            size: chunk.compressed,
            etag: current,
        },
    )?;
    if let Attempt::Retry(reason) =
        copy_response(&mut response, &mut target, chunk, budget, offset)?
    {
        return Ok(Attempt::Retry(reason));
    }
    fs::rename(part, archive)?;
    Ok(Attempt::Complete)
}

fn validate_response(
    response: &reqwest::blocking::Response,
    chunk: &Chunk,
    mut offset: u64,
    etag: Option<&str>,
) -> Result<(u64, Option<String>)> {
    let status = response.status();
    let current = response
        .headers()
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .filter(|v| v.starts_with('"') && v.ends_with('"'))
        .map(str::to_owned);
    ensure!(
        response
            .headers()
            .get(header::CONTENT_ENCODING)
            .is_none_or(|v| v == "identity"),
        "unexpected content encoding"
    );
    if status == StatusCode::PARTIAL_CONTENT {
        ensure!(
            offset > 0 && etag == current.as_deref(),
            "HTTP validator changed during resume"
        );
        let expected = format!(
            "bytes {offset}-{}/{}",
            chunk.compressed - 1,
            chunk.compressed
        );
        ensure!(
            response
                .headers()
                .get(header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                == Some(expected.as_str()),
            "invalid Content-Range"
        );
    } else {
        ensure!(
            status == StatusCode::OK,
            "unexpected HTTP status {status}; redirects are refused"
        );
        offset = 0;
    }
    ensure!(
        response
            .content_length()
            .is_none_or(|len| len == chunk.compressed - offset),
        "HTTP size differs from pinned archive"
    );
    Ok((offset, current))
}

fn copy_response(
    response: &mut reqwest::blocking::Response,
    target: &mut File,
    chunk: &Chunk,
    budget: &Budget<'_>,
    mut offset: u64,
) -> Result<Attempt> {
    let mut buffer = vec![0u8; BUFFER];
    loop {
        let read = match response.read(&mut buffer) {
            Ok(read) => read,
            Err(error) => return Ok(Attempt::Retry(error.to_string())),
        };
        if read == 0 {
            break;
        }
        ensure!(
            offset + read as u64 <= chunk.compressed,
            "archive exceeds expected size"
        );
        budget.write(target, &buffer[..read], true)?;
        offset += read as u64;
    }
    target.sync_all()?;
    if offset != chunk.compressed {
        return Ok(Attempt::Retry("truncated response".to_owned()));
    }
    Ok(Attempt::Complete)
}

pub fn verify_files(directory: &Path, chunk: &Chunk) -> Result<()> {
    reject_symlinks(directory)?;
    let mut paths = Vec::new();
    collect_files(directory, directory, &mut paths)?;
    ensure!(
        paths.len() == chunk.files.len()
            && paths
                .iter()
                .all(|p| chunk.files.iter().any(|f| f.path == *p)),
        "unexpected or missing staged files"
    );
    for expected in &chunk.files {
        let path = directory.join(&expected.path);
        ensure!(
            fs::metadata(&path)?.len() == expected.size,
            "wrong output size: {}",
            path.display()
        );
        let mut source = File::open(&path)?;
        let mut hash = blake3::Hasher::new();
        let mut buffer = vec![0; BUFFER];
        loop {
            let read = source.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hash.update(&buffer[..read]);
        }
        ensure!(
            hash.finalize().to_hex().as_str() == expected.blake3,
            "BLAKE3 mismatch: {}",
            path.display()
        );
    }
    Ok(())
}

fn collect_files(root: &Path, path: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else {
            files.push(
                entry
                    .path()
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    Ok(())
}

fn extract(root: &Path, chunk: &Chunk, archive: &Path, budget: &Budget<'_>) -> Result<()> {
    let temporary = root.join("partial").join(&chunk.name);
    fs::create_dir(&temporary)?;
    let mut decoder = zstd::stream::read::Decoder::new(File::open(archive)?)?;
    decoder.window_log_max(27)?;
    let limit = chunk
        .extracted
        .checked_add(1024 * 1024)
        .ok_or_else(|| eyre!("tar size overflow"))?;
    let mut tar = tar::Archive::new(decoder.take(limit + 1));
    let mut seen = std::collections::BTreeSet::new();
    for entry in tar.entries()?.raw(true) {
        budget.check(0)?;
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if entry.header().entry_type().is_dir()
            && (path == Path::new(".") || path == Path::new("static_files"))
        {
            continue;
        }
        let expected = chunk
            .files
            .iter()
            .find(|f| Path::new(&f.path) == path)
            .ok_or_else(|| eyre!("unexpected archive path: {}", path.display()))?;
        ensure!(
            entry.header().entry_type() == tar::EntryType::Regular
                && seen.insert(expected.path.clone()),
            "unsupported or duplicate archive entry"
        );
        ensure!(entry.size() == expected.size, "tar entry size mismatch");
        let output = temporary.join(&path);
        fs::create_dir_all(
            output
                .parent()
                .ok_or_else(|| eyre!("invalid output parent"))?,
        )?;
        let mut target = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output)?;
        let mut hash = blake3::Hasher::new();
        let mut buffer = vec![0u8; BUFFER];
        let mut written = 0;
        loop {
            let read = entry.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            budget.write(&mut target, &buffer[..read], false)?;
            written += read as u64;
            hash.update(&buffer[..read]);
        }
        ensure!(
            written == expected.size && hash.finalize().to_hex().as_str() == expected.blake3,
            "BLAKE3 mismatch: {}",
            expected.path
        );
        target.sync_all()?;
    }
    let mut stream = tar.into_inner();
    let mut buffer = vec![0; BUFFER];
    while stream.read(&mut buffer)? > 0 {
        budget.check(0)?;
    }
    ensure!(stream.limit() > 0, "tar exceeds payload/metadata budget");
    ensure!(seen.len() == chunk.files.len(), "archive is missing files");
    fs::rename(temporary, chunk_path(root, chunk))?;
    Ok(())
}

#[expect(clippy::print_stderr, reason = "snapshot CLI progress output")]
pub fn progress(message: &str) {
    eprintln!("{message}");
}

#[cfg(test)]
mod tests;
