//! Filesystem barriers used before publishing durable metadata or reclaiming inputs.
use eyre::{ensure, eyre, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)?.sync_all()?;
    Ok(())
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut name = path
        .file_name()
        .ok_or_else(|| eyre!("invalid durable file path"))?
        .to_os_string();
    name.push(".tmp");
    let temporary = path.with_file_name(name);
    let mut file = File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    if let Some(parent) = path.parent() {
        sync_directory(parent)?;
    }
    Ok(())
}

pub fn sync_tree(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        !metadata.file_type().is_symlink(),
        "symlink in durable storage: {}",
        path.display()
    );
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            sync_tree(&entry?.path())?;
        }
    }
    File::open(path)?.sync_all()?;
    Ok(())
}
