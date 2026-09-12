//! A trusted snapshot boundary fences subsequent writes and reorgs.
use super::{shard_start, Storage};
use crate::{
    cli::NodeConfig,
    storage::{durable, BlockBundle},
};
use alloy_primitives::B256;
use eyre::{ensure, eyre, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Deserialize, Serialize)]
pub struct Boundary {
    version: u32,
    chain_id: u64,
    genesis: B256,
    pub start: u64,
    pub number: u64,
    pub hash: B256,
    pub manifest_sha256: String,
}

pub(super) fn load(config: &NodeConfig) -> Result<Option<Boundary>> {
    match fs::read(config.data_dir.join("snapshot-boundary.json")) {
        Ok(bytes) => {
            let boundary: Boundary = serde_json::from_slice(&bytes)?;
            ensure!(
                boundary.version == 1
                    && boundary.chain_id == config.chain_id
                    && boundary.genesis == config.genesis_hash
                    && boundary.start <= boundary.number,
                "snapshot boundary identity/format mismatch"
            );
            Ok(Some(boundary))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl Storage {
    /// Check a restore's lower bound without opening storage or updating resume metadata.
    pub fn validate_snapshot_restore_start(config: &NodeConfig, start: u64) -> Result<()> {
        if let Some(boundary) = load(config)? {
            ensure!(start >= boundary.start,
                "restore starts before trusted history lower bound {}; use a fresh destination for earlier history",
                boundary.start);
        }
        Ok(())
    }

    pub fn configure_snapshot_sync(config: &mut NodeConfig) -> Result<()> {
        if let Some(boundary) = load(config)? {
            ensure!(config.start_block_override.is_none_or(|start| start == boundary.start),
                "snapshot-backed sync starts at {}; omit --start-block to resume the recorded range", boundary.start);
            config.start_block = boundary.start;
        } else {
            ensure!(!config.data_dir.join("history-package-init.json").exists(),
                "incomplete history package restore; repeat history-restore before P2P sync");
            ensure!(!config.data_dir.join("snapshot-imports").exists(),
                "snapshot P2P handoff requires an independently verified boundary hash; repeat snapshot-import with --checkpoint-hash <end-block-hash>");
        }
        Ok(())
    }

    pub fn snapshot_boundary(&self) -> Option<Boundary> {
        self.handoff.lock().clone()
    }

    pub fn pin_snapshot_boundary(
        &self,
        start: u64,
        number: u64,
        hash: B256,
        manifest: &str,
    ) -> Result<()> {
        let meta = self.meta.lock();
        let start = meta
            .sync_start_block
            .map_or(start, |saved| saved.min(start));
        let boundary = Boundary {
            version: 1,
            chain_id: meta.chain_id,
            genesis: meta
                .genesis_hash
                .ok_or_else(|| eyre!("missing storage genesis"))?,
            start,
            number,
            hash,
            manifest_sha256: manifest.into(),
        };
        drop(meta);
        ensure!(
            self.block_header(number)?
                .is_some_and(|header| header.hash_slow() == hash),
            "trusted checkpoint hash does not match imported boundary block {number}"
        );
        if let Some(previous) = self.snapshot_boundary() {
            ensure!(
                number >= previous.number && start == previous.start,
                "cannot lower or replace the trusted snapshot boundary"
            );
            self.validate_snapshot_boundary()?;
        }
        self.verify_header_chain(start, number)?;
        durable::atomic_write(
            &self.data_dir.join("snapshot-boundary.json"),
            &serde_json::to_vec_pretty(&boundary)?,
        )?;
        *self.handoff.lock() = Some(boundary);
        Ok(())
    }

    /// Only compacted contiguous coverage is an indexed head, even if later shards exist.
    pub(super) fn contiguous_head(&self, start: u64, maximum: u64) -> Result<Option<u64>> {
        if start > maximum {
            return Ok(None);
        }
        let size = self.shard_size();
        let mut next = start;
        while next <= maximum {
            let base = shard_start(next, size);
            let Some(shard) = self.get_shard(base)? else {
                return Ok(next.checked_sub(1).filter(|head| *head >= start));
            };
            let state = shard.lock();
            let end = base.saturating_add(size - 1).min(maximum);
            if !state.meta.complete {
                for number in next..=end {
                    if !state.bitset.is_set((number - base) as usize) {
                        return Ok(number.checked_sub(1).filter(|head| *head >= start));
                    }
                }
            }
            if end == u64::MAX {
                return Ok(Some(end));
            }
            next = end + 1;
        }
        Ok(Some(maximum))
    }

    fn verify_header_chain(&self, start: u64, end: u64) -> Result<()> {
        let mut previous = None;
        for number in start..=end {
            let header = self.block_header(number)?.ok_or_else(|| {
                eyre!("snapshot coverage gap at {number}; resume import before P2P handoff")
            })?;
            ensure!(
                header.number == number && previous.is_none_or(|hash| header.parent_hash == hash),
                "snapshot/P2P parent-hash mismatch at {number}"
            );
            previous = Some(header.hash_slow());
        }
        Ok(())
    }

    pub fn validate_snapshot_boundary(&self) -> Result<()> {
        let Some(boundary) = self.snapshot_boundary() else {
            ensure!(!self.data_dir.join("history-package-init.json").exists(),
                "incomplete history package restore; repeat history-restore before P2P sync");
            ensure!(!self.data_dir.join("snapshot-imports").exists(),
                "snapshot P2P handoff requires an independently verified boundary hash; repeat snapshot-import with --checkpoint-hash <end-block-hash>");
            return Ok(());
        };
        ensure!(
            self.block_header(boundary.number)?
                .is_some_and(|header| header.hash_slow() == boundary.hash),
            "trusted snapshot boundary {} is missing or has changed",
            boundary.number
        );
        let head = self
            .last_indexed_block()?
            .ok_or_else(|| eyre!("snapshot coverage is empty"))?;
        ensure!(
            head >= boundary.number,
            "gap inside trusted snapshot range; resume import before P2P handoff"
        );
        self.verify_header_chain(boundary.start, head)
    }

    pub(super) fn check_snapshot_write(&self, bundles: &[BlockBundle]) -> Result<()> {
        let Some(boundary) = self.snapshot_boundary() else {
            return Ok(());
        };
        let mut head = self
            .last_indexed_block()?
            .ok_or_else(|| eyre!("snapshot indexed head is missing"))?;
        ensure!(head >= boundary.number, "gap inside trusted snapshot range");
        let mut previous = self
            .block_header(head)?
            .ok_or_else(|| eyre!("missing indexed header"))?
            .hash_slow();
        for bundle in bundles {
            ensure!(
                bundle.header.number == bundle.number,
                "snapshot/P2P header number mismatch"
            );
            if bundle.number <= head {
                ensure!(
                    self.block_header(bundle.number)?.as_ref() == Some(&bundle.header)
                        && self.block_receipts(bundle.number)?.as_ref() == Some(&bundle.receipts)
                        && self.block_transactions(bundle.number)?.as_ref()
                            == Some(&bundle.transactions)
                        && self.block_tx_hashes(bundle.number)?.as_ref() == Some(&bundle.tx_hashes)
                        && self.block_size(bundle.number)?.as_ref() == Some(&bundle.size),
                    "conflicting block at or behind snapshot/P2P indexed head: {}",
                    bundle.number
                );
                continue;
            }
            ensure!(
                head.checked_add(1) == Some(bundle.number),
                "snapshot/P2P gap: expected block {}, received {}",
                head.saturating_add(1),
                bundle.number
            );
            ensure!(
                bundle.header.parent_hash == previous,
                "snapshot/P2P parent-hash mismatch at {}; trusted history will not be rewritten",
                bundle.number
            );
            head = bundle.number;
            previous = bundle.header.hash_slow();
            if let Some(next) = head.checked_add(1) {
                if let Some(header) = self.block_header(next)? {
                    ensure!(
                        header.parent_hash == previous,
                        "snapshot/P2P parent-hash mismatch at {next}"
                    );
                    // Filling a gap can expose a previously disconnected suffix.
                    // Validate all of it before its bitset can advance the indexed head.
                    let maximum = self.meta.lock().max_present_block.unwrap_or(next);
                    if let Some(end) = self.contiguous_head(next, maximum)? {
                        self.verify_header_chain(next, end)?;
                        // The next missing bundle extends this verified suffix.
                        head = end;
                        previous = self
                            .block_header(end)?
                            .ok_or_else(|| eyre!("missing verified suffix header at {end}"))?
                            .hash_slow();
                    }
                }
            }
        }
        Ok(())
    }
}
