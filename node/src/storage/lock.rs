//! A persistent lock inode shared by every storage writer, including maintenance.
use eyre::{ensure, Result, WrapErr};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

pub struct DataDirLock {
    path: PathBuf,
    _file: File,
}

impl DataDirLock {
    pub fn acquire(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        let path = fs::canonicalize(path)?;
        let lock_path = path.join(".writer.lock");
        if let Ok(metadata) = fs::symlink_metadata(&lock_path) {
            ensure!(
                metadata.is_file() && !metadata.file_type().is_symlink(),
                "invalid storage lock file"
            );
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;
        fs2::FileExt::try_lock_exclusive(&file).wrap_err_with(|| {
            format!("storage destination is already in use: {}", path.display())
        })?;
        Ok(Self { path, _file: file })
    }

    pub fn validate(&self, path: &Path) -> Result<()> {
        ensure!(
            fs::canonicalize(path)? == self.path,
            "storage lock belongs to another directory"
        );
        Ok(())
    }
}
