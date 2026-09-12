//! Checkpoints describe a job; recovered and verified storage decides its progress.
use super::{download, manifest::Plan};
use crate::{
    chain::ChainKind,
    storage::{BlockBundle, Storage, StorageCodec},
};
use eyre::{ensure, eyre, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Identity {
    format: u32,
    manifest_sha256: String,
    range: [u64; 2],
    chain_id: u64,
    genesis: alloy_primitives::B256,
    reader: String,
}

impl Identity {
    pub fn new(plan: &Plan) -> Self {
        Self {
            format: 1,
            manifest_sha256: plan.manifest_sha256.clone(),
            range: plan.requested_range,
            chain_id: 8453,
            genesis: ChainKind::Base.genesis_hash(),
            reader: "base-v2-pre-canyon-1".into(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Checkpoint {
    identity: Identity,
    pub verified_blocks: u64,
    pub complete_digest: Option<String>,
}

impl Checkpoint {
    pub fn path(output: &Path, plan: &Plan) -> PathBuf {
        output.join("snapshot-imports").join(format!(
            "{}-{}-{}.json",
            plan.manifest_sha256, plan.requested_range[0], plan.requested_range[1]
        ))
    }

    pub fn load(output: &Path, plan: &Plan) -> Result<Self> {
        let expected = Identity::new(plan);
        match fs::read(Self::path(output, plan)) {
            Ok(bytes) => {
                let checkpoint: Self = serde_json::from_slice(&bytes)?;
                ensure!(
                    checkpoint.identity == expected,
                    "import checkpoint identity/format mismatch"
                );
                Ok(checkpoint)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                identity: expected,
                verified_blocks: 0,
                complete_digest: None,
            }),
            Err(error) => Err(error.into()),
        }
    }

    pub fn save(&self, output: &Path, plan: &Plan) -> Result<()> {
        let path = Self::path(output, plan);
        fs::create_dir_all(output.join("snapshot-imports"))?;
        download::atomic_json(&path, self)?;
        crate::storage::durable::sync_directory(output)
    }

    pub fn reconcile(&mut self, storage: &Storage, plan: &Plan) -> Result<()> {
        let [start, end] = plan.requested_range;
        let missing = storage.missing_ranges_in_range(start..=end)?.1;
        self.verified_blocks = end - start + 1 - missing;
        self.complete_digest = None;
        Ok(())
    }
}

pub fn stored_bundle(storage: &Storage, number: u64) -> Result<Option<BlockBundle>> {
    let Some(header) = storage.block_header(number)? else {
        return Ok(None);
    };
    Ok(Some(BlockBundle {
        number,
        header,
        tx_hashes: storage
            .block_tx_hashes(number)?
            .ok_or_else(|| eyre!("missing transaction hashes at {number}"))?,
        transactions: storage
            .block_transactions(number)?
            .ok_or_else(|| eyre!("missing transactions at {number}"))?,
        receipts: storage
            .block_receipts(number)?
            .ok_or_else(|| eyre!("missing receipts at {number}"))?,
        size: storage
            .block_size(number)?
            .ok_or_else(|| eyre!("missing block size at {number}"))?,
    }))
}

pub fn compare(storage: &Storage, bundle: &BlockBundle) -> Result<bool> {
    let Some(stored) = stored_bundle(storage, bundle.number)? else {
        return Ok(false);
    };
    ensure!(
        stored.header.hash_slow() == bundle.header.hash_slow(),
        "conflicting block hash at {}",
        bundle.number
    );
    ensure!(
        stored == *bundle,
        "conflicting stored block data at {}",
        bundle.number
    );
    Ok(true)
}

pub fn hash_bundle(hash: &mut Sha256, bundle: &BlockBundle) -> Result<()> {
    hash.update(bundle.number.to_le_bytes());
    hash.update(bundle.header.hash_slow());
    hash.update(bundle.size.size.to_le_bytes());
    let codec = StorageCodec::OpV1;
    for bytes in [
        crate::storage::encode_bincode_value(&bundle.tx_hashes)?,
        codec.encode_transactions(&bundle.transactions)?,
        codec.encode_receipts(&bundle.receipts)?,
    ] {
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(())
}

pub fn digest(storage: &Storage, plan: &Plan) -> Result<Option<String>> {
    let mut hash = Sha256::new();
    for number in plan.requested_range[0]..=plan.requested_range[1] {
        let Some(bundle) = stored_bundle(storage, number)? else {
            return Ok(None);
        };
        hash_bundle(&mut hash, &bundle)?;
    }
    Ok(Some(format!("{:x}", hash.finalize())))
}
