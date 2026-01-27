use eyre::{Result, WrapErr};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn compute_shard_hash(
    shard_start: u64,
    shard_size: u32,
    tail_block: u64,
    bitset: &[u8],
    sorted_dir: &Path,
) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(b"stateless-history-shard-v1\n");
    hasher.update(shard_start.to_le_bytes());
    hasher.update(shard_size.to_le_bytes());
    hasher.update(tail_block.to_le_bytes());
    hasher.update(bitset);

    let mut files: Vec<PathBuf> = fs::read_dir(sorted_dir)
        .wrap_err("failed to read sorted dir")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    for path in files {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default();
        hasher.update(name.as_bytes());
        hasher.update([0u8]);
        let len = fs::metadata(&path)?.len();
        hasher.update(len.to_le_bytes());
        hasher.update([0u8]);
        let mut file = fs::File::open(&path)?;
        let mut buf = vec![0u8; 1024 * 1024];
        loop {
            let read = file.read(&mut buf)?;
            if read == 0 {
                break;
            }
            hasher.update(&buf[..read]);
        }
    }

    let digest = hasher.finalize();
    Ok(to_hex(&digest))
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize]);
        out.push(HEX[(byte & 0x0f) as usize]);
    }
    String::from_utf8(out).unwrap_or_default()
}
