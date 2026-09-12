use super::format::{self, Shard};
use crate::snapshot::download;
use eyre::{ensure, eyre, Result};
use std::{
    collections::HashSet,
    fs,
    io::{self, Read, Write},
    path::Path,
};

pub fn pack(files: &[(String, std::path::PathBuf)], output: &Path, reserve: u64) -> Result<()> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    let writer = SpaceWriter {
        file,
        directory: output
            .parent()
            .ok_or_else(|| eyre!("missing archive parent"))?,
        reserve,
    };
    let encoder = zstd::stream::write::Encoder::new(writer, 3)?;
    let mut archive = tar::Builder::new(encoder);
    for (name, path) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(fs::metadata(path)?.len());
        header.set_mode(0o644);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        archive.append_data(&mut header, name, fs::File::open(path)?)?;
    }
    let encoder = archive.into_inner()?;
    encoder.finish()?.file.sync_all()?;
    Ok(())
}

struct SpaceWriter<'a> {
    file: fs::File,
    directory: &'a Path,
    reserve: u64,
}
impl Write for SpaceWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        download::check_space(self.directory, self.reserve, bytes.len() as u64)
            .map_err(|error| io::Error::other(error.to_string()))?;
        self.file.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

struct Limited<R> {
    inner: R,
    remaining: u64,
}
impl<R: Read> Read for Limited<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            let mut probe = [0];
            return if self.inner.read(&mut probe)? == 0 {
                Ok(0)
            } else {
                Err(io::Error::other(
                    "archive exceeds declared extraction budget",
                ))
            };
        }
        let limit = bytes
            .len()
            .min(usize::try_from(self.remaining).unwrap_or(usize::MAX));
        let n = self.inner.read(&mut bytes[..limit])?;
        self.remaining -= n as u64;
        Ok(n)
    }
}

pub fn unpack(root: &Path, shard: &Shard, destination: &Path, reserve: u64) -> Result<()> {
    let path = root.join(&shard.archive);
    ensure!(
        fs::metadata(&path)?.len() == shard.archive_bytes
            && format::file_sha(&path)? == shard.archive_sha256,
        "archive checksum/size mismatch: {}",
        shard.archive
    );
    let mut decoder = zstd::stream::read::Decoder::new(fs::File::open(path)?)?;
    decoder.window_log_max(27)?;
    let payload = shard.files.iter().try_fold(0u64, |sum, file| {
        sum.checked_add(file.bytes)
            .ok_or_else(|| eyre::eyre!("size overflow"))
    })?;
    let limit = payload
        .checked_add(1024 * 1024)
        .ok_or_else(|| eyre::eyre!("size overflow"))?;
    let mut archive = tar::Archive::new(Limited {
        inner: decoder,
        remaining: limit,
    });
    let mut seen = HashSet::new();
    for entry in archive.entries()?.raw(true) {
        let mut entry = entry?;
        ensure!(
            entry.header().entry_type().is_file(),
            "non-regular archive entry"
        );
        let name = entry.path()?.to_string_lossy().into_owned();
        let declared = shard
            .files
            .iter()
            .find(|file| file.name == name)
            .ok_or_else(|| eyre::eyre!("unexpected archive path: {name}"))?;
        ensure!(seen.insert(name.clone()), "duplicate archive file");
        ensure!(
            entry.size() == declared.bytes,
            "archive entry size mismatch"
        );
        let output = destination.join(&name);
        fs::create_dir_all(
            output
                .parent()
                .ok_or_else(|| eyre!("missing file parent"))?,
        )?;
        download::check_space(destination, reserve, declared.bytes)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)?;
        ensure!(
            io::copy(&mut entry, &mut file)? == declared.bytes,
            "truncated archive file"
        );
        file.sync_all()?;
        ensure!(
            format::file_sha(&output)? == declared.sha256,
            "file checksum mismatch: {name}"
        );
    }
    ensure!(
        seen.len() == shard.files.len(),
        "archive missing declared files"
    );
    let mut tail = archive.into_inner();
    let mut bytes = [0; 8192];
    loop {
        let n = tail.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        ensure!(
            bytes[..n].iter().all(|byte| *byte == 0),
            "unexpected data after tar end"
        );
    }
    Ok(())
}
