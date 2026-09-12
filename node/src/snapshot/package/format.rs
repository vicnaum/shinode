use crate::{
    chain::ChainKind,
    storage::{Storage, StorageCodec, StorageConfigKey},
};
use alloy_primitives::B256;
use eyre::{ensure, eyre, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub storage_schema: u32,
    pub chain_id: u64,
    pub genesis: B256,
    pub codec: StorageCodec,
    pub storage_key: StorageConfigKey,
    pub shard_size: u64,
    pub start: u64,
    pub end: u64,
    pub end_hash: B256,
    pub source: Source,
    pub shards: Vec<Shard>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub format: String,
    pub manifest_sha256: String,
    pub boundary_number: u64,
    pub boundary_hash: B256,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shard {
    pub start: u64,
    pub end: u64,
    pub first_parent: B256,
    pub end_hash: B256,
    pub sealed_hash: String,
    pub bundle_sha256: String,
    pub archive: String,
    pub archive_bytes: u64,
    pub archive_sha256: String,
    pub files: Vec<File>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
}

pub fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn file_sha(path: &Path) -> Result<String> {
    ensure!(
        fs::symlink_metadata(path)?.file_type().is_file(),
        "not a regular file: {}",
        path.display()
    );
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

impl Manifest {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1 && self.storage_schema == 2,
            "unsupported history package format"
        );
        ensure!(
            self.chain_id == 8453
                && self.genesis == ChainKind::Base.genesis_hash()
                && self.codec == StorageCodec::OpV1,
            "package chain/genesis/codec mismatch"
        );
        ensure!(
            self.shard_size > 0
                && u32::try_from(self.shard_size).is_ok()
                && self.start <= self.end
                && self.start.is_multiple_of(self.shard_size)
                && self
                    .end
                    .checked_add(1)
                    .is_some_and(|end| end.is_multiple_of(self.shard_size)),
            "package range must consist of complete shards"
        );
        ensure!(
            self.source.format == "shinode-trusted-boundary-v1"
                && valid_sha(&self.source.manifest_sha256),
            "invalid source provenance"
        );
        ensure!(
            self.shards.len() <= 100_000 && !self.shards.is_empty(),
            "invalid shard count"
        );
        let mut next = self.start;
        let mut parent = None;
        for shard in &self.shards {
            ensure!(
                shard.start == next
                    && shard.end
                        == next
                            .checked_add(self.shard_size - 1)
                            .ok_or_else(|| eyre!("range overflow"))?,
                "package coverage gap or overlapping shard"
            );
            ensure!(
                shard.archive == format!("shard-{}.tar.zst", shard.start)
                    && valid_sha(&shard.archive_sha256)
                    && valid_sha(&shard.sealed_hash)
                    && valid_sha(&shard.bundle_sha256)
                    && shard.archive_bytes > 0,
                "invalid shard archive identity"
            );
            ensure!(
                parent.is_none_or(|hash| hash == shard.first_parent),
                "package parent mismatch"
            );
            ensure!(
                shard
                    .files
                    .iter()
                    .map(|file| file.name.clone())
                    .collect::<Vec<_>>()
                    == Storage::sealed_shard_file_names(),
                "unexpected or duplicate package file paths"
            );
            ensure!(
                shard.files.iter().all(|file| valid_sha(&file.sha256)),
                "invalid file checksum"
            );
            next = shard
                .end
                .checked_add(1)
                .ok_or_else(|| eyre!("range overflow"))?;
            parent = Some(shard.end_hash);
        }
        ensure!(
            next == self.end + 1 && parent == Some(self.end_hash),
            "package claims unsupported coverage"
        );
        Ok(())
    }

    pub fn sizes(&self) -> Result<(u64, u64)> {
        let sum = |values: Vec<u64>| {
            values.into_iter().try_fold(0u64, |sum, n| {
                sum.checked_add(n)
                    .ok_or_else(|| eyre!("package size overflow"))
            })
        };
        Ok((
            sum(self.shards.iter().map(|s| s.archive_bytes).collect())?,
            sum(self
                .shards
                .iter()
                .flat_map(|s| s.files.iter().map(|f| f.bytes))
                .collect())?,
        ))
    }
}
