//! Read-only verification of native sealed shard files for distribution.
use super::{bitset_path, compute_shard_hash, load_shard_meta, sorted_dir, wal_path, Storage};
use eyre::{ensure, eyre, Result};
use std::{fs, path::PathBuf};

pub struct SealedShardFiles {
    pub hash: String,
    pub files: Vec<(String, PathBuf)>,
}

pub fn file_names() -> Vec<String> {
    let mut names = vec!["shard.json".into(), "present.bitset".into()];
    for segment in ["headers", "tx_hashes", "tx_meta", "receipts", "block_sizes"] {
        for suffix in ["", ".conf", ".off"] {
            names.push(format!("sorted/{segment}{suffix}"));
        }
    }
    names.sort();
    names
}

impl Storage {
    pub fn sealed_shard_file_names() -> Vec<String> {
        file_names()
    }

    pub fn verified_sealed_shard(&self, start: u64) -> Result<SealedShardFiles> {
        let shard = self
            .get_shard(start)?
            .ok_or_else(|| eyre!("missing shard {start}"))?;
        let state = shard.lock();
        // A cached seal cannot authenticate the metadata actually being shipped.
        let meta = load_shard_meta(&state.dir)?;
        let size = self.shard_size();
        let end = start
            .checked_add(size - 1)
            .ok_or_else(|| eyre!("shard end overflow"))?;
        ensure!(
            start.is_multiple_of(size)
                && meta.shard_start == start
                && meta.shard_size == size
                && meta.tail_block == Some(end)
                && meta.complete
                && meta.sorted
                && meta.sealed
                && meta.compaction_phase.is_none()
                && u64::from(meta.present_count) == size,
            "shard {start} is not complete, sorted and sealed; run db compact first"
        );
        ensure!(
            !wal_path(&state.dir).exists(),
            "pending WAL in shard {start}"
        );
        ensure!(
            (0..size).all(|offset| state.bitset.is_set(offset as usize)),
            "coverage gap in shard {start}"
        );
        for dir in [&state.dir, &sorted_dir(&state.dir)] {
            ensure!(
                fs::symlink_metadata(dir)?.file_type().is_dir(),
                "invalid shard directory"
            );
        }
        ensure!(
            fs::read(bitset_path(&state.dir))? == state.bitset.bytes(),
            "stored bitset changed"
        );
        let mut files = Vec::new();
        for name in file_names() {
            let path = state.dir.join(&name);
            ensure!(
                fs::symlink_metadata(&path)?.file_type().is_file(),
                "invalid shard file {name}"
            );
            files.push((name, path));
        }
        ensure!(
            fs::read_dir(sorted_dir(&state.dir))?.count() == 15,
            "unexpected sorted shard files"
        );
        let hash = compute_shard_hash(
            start,
            u32::try_from(size)?,
            end,
            state.bitset.bytes(),
            &sorted_dir(&state.dir),
        )?;
        ensure!(
            meta.content_hash_algo == "sha256" && meta.content_hash.as_ref() == Some(&hash),
            "sealed shard checksum mismatch at {start}"
        );
        Ok(SealedShardFiles { hash, files })
    }
}
