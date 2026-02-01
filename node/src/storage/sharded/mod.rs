//! Sharded storage backend (schema v2).

mod bitset;
mod hash;
mod nippy_raw;
mod wal;

use crate::cli::NodeConfig;
use crate::storage::{
    decode_bincode, decode_bincode_compat_value, decode_u64, encode_bincode_compat_value,
    encode_bincode_value, encode_u64_value, BlockBundle, PeerCacheLoad, SegmentDiskStats,
    StorageConfigKey, StorageDiskStats, StoredBlockSize, StoredPeer, StoredReceipts,
    StoredTxHashes, ZSTD_DICT_MAX_SIZE,
};
use bitset::Bitset;
use eyre::{eyre, Result, WrapErr};
use hash::compute_shard_hash;
use nippy_raw::{NippyJarConfig, SegmentRawSource, SegmentRawWriter};
use reth_nippy_jar::{
    compression::{Compressors, Zstd},
    NippyJar, NippyJarCursor, NippyJarWriter, CONFIG_FILE_EXTENSION,
};
use reth_primitives_traits::{serde_bincode_compat::SerdeBincodeCompat, Header};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use wal::{append_records, build_slice_index, WalRecord};
use zstd::bulk::Compressor;

const SCHEMA_VERSION: u64 = 2;
const SEGMENT_CACHE_SIZE: usize = 20; // Cache up to 20 shards worth of segment readers
const META_FILE_NAME: &str = "meta.json";
const PEER_FILE_NAME: &str = "peers.json";
const STATIC_DIR_NAME: &str = "static";
const SHARDS_DIR_NAME: &str = "shards";
const SHARD_META_NAME: &str = "shard.json";
const BITSET_NAME: &str = "present.bitset";
const BITSET_TMP_NAME: &str = "bitset.tmp";
const BITSET_OLD_NAME: &str = "bitset.old";
const WAL_NAME: &str = "staging.wal";

// Compaction phase constants
const PHASE_WRITING: &str = "writing";
const PHASE_SWAPPING: &str = "swapping";
const PHASE_CLEANUP: &str = "cleanup";

/// Container for raw segment readers used during compaction.
struct ShardRawSegments {
    headers: SegmentRawSource<SegmentHeader>,
    tx_hashes: SegmentRawSource<SegmentHeader>,
    tx_meta: SegmentRawSource<SegmentHeader>,
    receipts: SegmentRawSource<SegmentHeader>,
    sizes: SegmentRawSource<SegmentHeader>,
}

/// Container for raw segment writers used during compaction.
struct ShardRawWriters {
    headers: SegmentRawWriter<SegmentHeader>,
    tx_hashes: SegmentRawWriter<SegmentHeader>,
    tx_meta: SegmentRawWriter<SegmentHeader>,
    receipts: SegmentRawWriter<SegmentHeader>,
    sizes: SegmentRawWriter<SegmentHeader>,
}

impl ShardRawWriters {
    fn finish(self) -> Result<()> {
        self.headers.finish()?;
        self.tx_hashes.finish()?;
        self.tx_meta.finish()?;
        self.receipts.finish()?;
        self.sizes.finish()?;
        Ok(())
    }

    fn push_empty(&mut self) -> Result<()> {
        self.headers.push_empty()?;
        self.tx_hashes.push_empty()?;
        self.tx_meta.push_empty()?;
        self.receipts.push_empty()?;
        self.sizes.push_empty()?;
        Ok(())
    }
}

const fn create_segment_config(
    shard_start: u64,
    compression: SegmentCompression,
    existing_max_row_size: usize,
) -> NippyJarConfig<SegmentHeader> {
    NippyJarConfig {
        version: 1,
        user_header: SegmentHeader { start_block: shard_start },
        columns: 1,
        rows: 0,
        compressor: segment_compressor(compression),
        max_row_size: existing_max_row_size,
    }
}

fn create_compaction_writers(
    temp_dir: &Path,
    shard_start: u64,
    existing: Option<&ShardRawSegments>,
) -> Result<ShardRawWriters> {
    let headers_cfg = create_segment_config(
        shard_start,
        SegmentCompression::None,
        existing.map_or(0, |s| s.headers.config().max_row_size),
    );
    let tx_hashes_cfg = create_segment_config(
        shard_start,
        SegmentCompression::None,
        existing.map_or(0, |s| s.tx_hashes.config().max_row_size),
    );
    let tx_meta_cfg = create_segment_config(
        shard_start,
        SegmentCompression::Zstd {
            use_dict: false,
            max_dict_size: ZSTD_DICT_MAX_SIZE,
        },
        existing.map_or(0, |s| s.tx_meta.config().max_row_size),
    );
    let receipts_cfg = create_segment_config(
        shard_start,
        SegmentCompression::Zstd {
            use_dict: false,
            max_dict_size: ZSTD_DICT_MAX_SIZE,
        },
        existing.map_or(0, |s| s.receipts.config().max_row_size),
    );
    let sizes_cfg = create_segment_config(
        shard_start,
        SegmentCompression::None,
        existing.map_or(0, |s| s.sizes.config().max_row_size),
    );

    Ok(ShardRawWriters {
        headers: SegmentRawWriter::create(&temp_dir.join("headers"), headers_cfg)?,
        tx_hashes: SegmentRawWriter::create(&temp_dir.join("tx_hashes"), tx_hashes_cfg)?,
        tx_meta: SegmentRawWriter::create(&temp_dir.join("tx_meta"), tx_meta_cfg)?,
        receipts: SegmentRawWriter::create(&temp_dir.join("receipts"), receipts_cfg)?,
        sizes: SegmentRawWriter::create(&temp_dir.join("block_sizes"), sizes_cfg)?,
    })
}

/// Aggregate statistics across all shards, computed from in-memory shard metadata.
#[derive(Debug, Clone, Default)]
pub struct StorageAggregateStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_receipts: u64,
    pub total_logs: u64,
    pub total_shards: u64,
    pub compacted_shards: u64,
    pub disk_bytes_headers: u64,
    pub disk_bytes_transactions: u64,
    pub disk_bytes_receipts: u64,
    pub disk_bytes_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaState {
    schema_version: u64,
    chain_id: u64,
    storage_key: StorageConfigKey,
    shard_size: u64,
    max_present_block: Option<u64>,
    head_seen: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShardMeta {
    shard_start: u64,
    shard_size: u64,
    present_count: u32,
    complete: bool,
    sorted: bool,
    sealed: bool,
    tail_block: Option<u64>,
    content_hash: Option<String>,
    #[serde(default = "default_hash_algo")]
    content_hash_algo: String,
    /// Tracks compaction progress for crash recovery.
    /// Values: None (idle), "writing", "swapping", "cleanup"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    compaction_phase: Option<String>,
    /// Total transaction count across all blocks in this shard.
    #[serde(default)]
    total_transactions: u64,
    /// Total receipt count across all blocks in this shard.
    #[serde(default)]
    total_receipts: u64,
    /// Total log count across all blocks in this shard.
    /// Accumulated during follow-mode writes; preserved (not recomputed) during compaction
    /// because receipts are zstd-compressed and decoding just to count logs is expensive.
    #[serde(default)]
    total_logs: u64,
    /// On-disk bytes for headers segment files.
    #[serde(default)]
    disk_bytes_headers: u64,
    /// On-disk bytes for transaction segment files (tx_hashes + tx_meta).
    #[serde(default)]
    disk_bytes_transactions: u64,
    /// On-disk bytes for receipts segment files.
    #[serde(default)]
    disk_bytes_receipts: u64,
    /// Total on-disk bytes for all segment files in this shard.
    #[serde(default)]
    disk_bytes_total: u64,
}

struct ShardState {
    dir: PathBuf,
    meta: ShardMeta,
    bitset: Bitset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalBundleRecord {
    number: u64,
    header: Vec<u8>,
    tx_hashes: Vec<u8>,
    tx_meta: Vec<u8>,
    tx_meta_uncompressed_len: u32,
    receipts: Vec<u8>,
    receipts_uncompressed_len: u32,
    size: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SegmentHeader {
    start_block: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentCompression {
    None,
    Zstd {
        use_dict: bool,
        max_dict_size: usize,
    },
}

const fn segment_compressor(compression: SegmentCompression) -> Option<Compressors> {
    match compression {
        SegmentCompression::None => None,
        SegmentCompression::Zstd {
            use_dict,
            max_dict_size,
        } => Some(Compressors::Zstd(Zstd::new(use_dict, max_dict_size, 1))),
    }
}

struct SegmentState {
    writer: NippyJarWriter<SegmentHeader>,
    next_block: u64,
}

struct SegmentWriter {
    path: PathBuf,
    start_block: u64,
    state: Mutex<SegmentState>,
}

impl SegmentWriter {
    fn open(path: PathBuf, start_block: u64, compression: SegmentCompression) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).wrap_err("failed to create segment dir")?;
        }
        let jar = if segment_exists(&path) {
            let jar =
                NippyJar::<SegmentHeader>::load(&path).wrap_err("failed to load static segment")?;
            let header = jar.user_header();
            if header.start_block != start_block {
                return Err(eyre!(
                    "segment start mismatch for {} (expected {}, got {})",
                    path.display(),
                    start_block,
                    header.start_block
                ));
            }
            let expected_compress = matches!(compression, SegmentCompression::Zstd { .. });
            if jar.compressor().is_some() != expected_compress {
                return Err(eyre!("segment compression mismatch for {}", path.display()));
            }
            jar
        } else {
            let mut jar = NippyJar::new(1, &path, SegmentHeader { start_block });
            match compression {
                SegmentCompression::None => {}
                SegmentCompression::Zstd {
                    use_dict,
                    max_dict_size,
                } => {
                    jar = jar.with_zstd(use_dict, max_dict_size);
                }
            }
            jar
        };

        let next_block = start_block + jar.rows() as u64;
        let writer = NippyJarWriter::new(jar).wrap_err("failed to open static segment writer")?;
        Ok(Self {
            path,
            start_block,
            state: Mutex::new(SegmentState { writer, next_block }),
        })
    }

    fn append_rows_inner(&self, start_block: u64, rows: &[Vec<u8>]) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        let mut state = self.state.lock();
        if start_block < state.next_block {
            return Err(eyre!(
                "segment {} expected block {}, got {}",
                self.path.display(),
                state.next_block,
                start_block
            ));
        }
        let gap = start_block.saturating_sub(state.next_block) as usize;
        // IMPORTANT: avoid cloning potentially huge row payloads.
        //
        // We intentionally feed the writer an iterator of borrowed slices. This keeps memory usage
        // bounded by the caller-owned `rows` buffer, and prevents `append_rows` from duplicating
        // shard-sized data during compaction.
        let gap_iter = std::iter::repeat_with(|| {
            Ok::<&[u8], Box<dyn std::error::Error + Send + Sync>>(&[][..])
        })
        .take(gap);
        let rows_iter = rows
            .iter()
            .map(|bytes| Ok::<&[u8], Box<dyn std::error::Error + Send + Sync>>(bytes.as_slice()));
        let iter = gap_iter.chain(rows_iter);
        let total_rows = gap.saturating_add(rows.len()) as u64;
        state
            .writer
            .append_rows(vec![iter], total_rows)
            .wrap_err("failed to append rows")?;
        state
            .writer
            .commit()
            .wrap_err("failed to commit static segment")?;
        state.next_block = state.next_block.saturating_add(total_rows);
        Ok(())
    }

    fn append_rows(&self, start_block: u64, rows: &[Vec<u8>]) -> Result<()> {
        self.append_rows_inner(start_block, rows)
    }

    fn prune_to(&self, ancestor_number: u64) -> Result<()> {
        let mut state = self.state.lock();
        if ancestor_number < self.start_block {
            let prune_rows = state.next_block.saturating_sub(self.start_block);
            if prune_rows > 0 {
                state
                    .writer
                    .prune_rows(prune_rows as usize)
                    .wrap_err("failed to prune segment")?;
                state
                    .writer
                    .commit()
                    .wrap_err("failed to commit segment prune")?;
            }
            state.next_block = self.start_block;
            return Ok(());
        }
        let target_next = ancestor_number.saturating_add(1);
        if target_next >= state.next_block {
            return Ok(());
        }
        let prune_rows = state.next_block.saturating_sub(target_next);
        state
            .writer
            .prune_rows(prune_rows as usize)
            .wrap_err("failed to prune segment")?;
        state
            .writer
            .commit()
            .wrap_err("failed to commit segment prune")?;
        state.next_block = target_next;
        Ok(())
    }

}

pub struct Storage {
    data_dir: PathBuf,
    peer_cache_dir: PathBuf,
    meta: Mutex<MetaState>,
    shards: Mutex<HashMap<u64, Arc<Mutex<ShardState>>>>,
    peer_cache: Mutex<HashMap<String, StoredPeer>>,
    /// LRU cache for read-only segment handles, keyed by shard_start.
    /// Avoids reopening 5 segment files on every block read.
    segment_cache: Mutex<LruCache<u64, Arc<ShardSegmentReaders>>>,
}

#[derive(Debug, Clone)]
pub struct DirtyShardInfo {
    pub shard_start: u64,
    pub wal_bytes: u64,
}

#[expect(clippy::missing_fields_in_debug, reason = "intentionally show only data_dir")]
impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("data_dir", &self.data_dir)
            .finish()
    }
}

impl Storage {
    #[expect(clippy::cognitive_complexity, reason = "storage open with config validation and recovery")]
    pub fn open(config: &NodeConfig) -> Result<Self> {
        fs::create_dir_all(&config.data_dir).wrap_err("failed to create data dir")?;
        let peer_cache_dir = config
            .peer_cache_dir
            .clone()
            .unwrap_or_else(|| config.data_dir.clone());
        fs::create_dir_all(&peer_cache_dir).wrap_err("failed to create peer cache dir")?;

        let meta_path = meta_path(&config.data_dir);
        let mut meta = if meta_path.exists() {
            load_meta(&meta_path)?
        } else {
            MetaState {
                schema_version: SCHEMA_VERSION,
                chain_id: config.chain_id,
                storage_key: StorageConfigKey::from(config),
                shard_size: config.shard_size,
                max_present_block: None,
                head_seen: None,
            }
        };
        validate_meta(config, &meta)?;
        if meta.schema_version != SCHEMA_VERSION {
            return Err(eyre!(
                "schema version mismatch: expected {}, got {} (delete data dir to rebuild)",
                SCHEMA_VERSION,
                meta.schema_version
            ));
        }
        if !meta_path.exists() {
            persist_meta(&meta_path, &meta)?;
        }

        let static_dir = static_dir(&config.data_dir);
        fs::create_dir_all(&static_dir).wrap_err("failed to create static dir")?;
        let shards_root = shards_dir(&config.data_dir);
        fs::create_dir_all(&shards_root).wrap_err("failed to create shards dir")?;

        let mut shards: HashMap<u64, Arc<Mutex<ShardState>>> = HashMap::new();
        for entry in fs::read_dir(&shards_root)? {
            let entry = entry?;
            if !entry.metadata()?.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(shard_start) = name.parse::<u64>() else {
                continue;
            };
            let shard_dir = entry.path();

            // Load shard metadata
            let mut shard_meta = load_shard_meta(&shard_dir)?;

            // Perform phase-aware recovery if needed
            let recovery_result = recover_shard(&shard_dir, &mut shard_meta)?;
            match &recovery_result {
                ShardRecoveryResult::Clean => {}
                ShardRecoveryResult::CleanedOrphans => {
                    tracing::info!(shard = shard_start, "cleaned orphan files");
                }
                result => {
                    tracing::info!(shard = shard_start, recovery = ?result, "recovered shard from interrupted compaction");
                    // Persist the updated metadata after recovery
                    persist_shard_meta(&shard_dir, &shard_meta)?;
                }
            }

            // Load bitset (may have been restored by recovery)
            let bitset = Bitset::load(&bitset_path(&shard_dir), meta.shard_size as usize)?;
            let mut state = ShardState {
                dir: shard_dir,
                meta: shard_meta,
                bitset,
            };

            // Mark as needing compaction if WAL exists
            repair_shard_from_wal(&mut state)?;
            reconcile_shard_meta(&mut state);

            // Recompute disk sizes from segment files on startup
            let (dh, dt, dr, dtotal) = compute_shard_disk_bytes(&state.dir);
            state.meta.disk_bytes_headers = dh;
            state.meta.disk_bytes_transactions = dt;
            state.meta.disk_bytes_receipts = dr;
            state.meta.disk_bytes_total = dtotal;

            let max_present = max_present_in_bitset(&state, meta.shard_size);
            if let Some(max_block) = max_present {
                meta.max_present_block = Some(meta.max_present_block.unwrap_or(0).max(max_block));
            }
            shards.insert(shard_start, Arc::new(Mutex::new(state)));
        }
        persist_meta(&meta_path, &meta)?;

        let peer_cache = load_peer_cache(&peer_cache_dir)?;
        let segment_cache = LruCache::new(
            NonZeroUsize::new(SEGMENT_CACHE_SIZE)
                .unwrap_or_else(|| NonZeroUsize::new(1).expect("1 is non-zero")),
        );
        Ok(Self {
            data_dir: config.data_dir.clone(),
            peer_cache_dir,
            meta: Mutex::new(meta),
            shards: Mutex::new(shards),
            peer_cache: Mutex::new(peer_cache),
            segment_cache: Mutex::new(segment_cache),
        })
    }

    /// Run storage repair without starting the full node.
    /// Scans all shards for interrupted compactions and orphan files,
    /// performs recovery, and returns a detailed report.
    pub fn repair(config: &NodeConfig) -> Result<RepairReport> {
        // Ensure directories exist
        if !config.data_dir.exists() {
            return Ok(RepairReport { shards: vec![] });
        }

        let shards_root = shards_dir(&config.data_dir);
        if !shards_root.exists() {
            return Ok(RepairReport { shards: vec![] });
        }

        let meta_path = meta_path(&config.data_dir);
        let _meta = if meta_path.exists() {
            load_meta(&meta_path)?
        } else {
            // No meta file means empty storage
            return Ok(RepairReport { shards: vec![] });
        };

        let mut results = Vec::new();

        // Scan all shard directories
        let mut shard_dirs: Vec<_> = fs::read_dir(&shards_root)?
            .filter_map(Result::ok)
            .filter(|e| e.metadata().map(|m| m.is_dir()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.parse::<u64>().ok().map(|start| (start, e.path()))
            })
            .collect();

        // Sort by shard start for consistent output
        shard_dirs.sort_by_key(|(start, _)| *start);

        for (shard_start, shard_dir) in shard_dirs {
            // Load shard metadata
            let mut shard_meta = load_shard_meta(&shard_dir)?;
            let original_phase = shard_meta.compaction_phase.clone();

            // Perform recovery
            let result = recover_shard(&shard_dir, &mut shard_meta)?;

            // Persist updated metadata if recovery was performed
            if result.needs_recovery() {
                persist_shard_meta(&shard_dir, &shard_meta)?;

                // Also check for WAL and mark as needing compaction if present
                let wal = wal_path(&shard_dir);
                if wal.exists() {
                    shard_meta.sorted = false;
                    shard_meta.sealed = false;
                    shard_meta.content_hash = None;
                    persist_shard_meta(&shard_dir, &shard_meta)?;
                }
            }

            results.push(ShardRepairInfo {
                shard_start,
                result,
                original_phase,
            });
        }

        Ok(RepairReport { shards: results })
    }

    pub fn disk_usage(&self) -> Result<StorageDiskStats> {
        Self::disk_usage_at(&self.data_dir)
    }

    /// Aggregate stats across all shards from in-memory shard metadata.
    /// This is cheap — no disk I/O, just iterates locked shard meta.
    pub fn aggregate_stats(&self) -> StorageAggregateStats {
        let shards = self.shards.lock();
        let mut stats = StorageAggregateStats::default();
        stats.total_shards = shards.len() as u64;
        for shard in shards.values() {
            let state = shard.lock();
            stats.total_blocks += u64::from(state.meta.present_count);
            stats.total_transactions += state.meta.total_transactions;
            stats.total_receipts += state.meta.total_receipts;
            stats.total_logs += state.meta.total_logs;
            if state.meta.sorted {
                stats.compacted_shards += 1;
            }
            stats.disk_bytes_headers += state.meta.disk_bytes_headers;
            stats.disk_bytes_transactions += state.meta.disk_bytes_transactions;
            stats.disk_bytes_receipts += state.meta.disk_bytes_receipts;
            stats.disk_bytes_total += state.meta.disk_bytes_total;
        }
        stats
    }

    pub fn disk_usage_at(data_dir: &Path) -> Result<StorageDiskStats> {
        let static_dir = static_dir(data_dir);
        let meta_bytes = file_size(&meta_path(data_dir))?;
        let peers_bytes = file_size(&peer_path(data_dir))?;
        let static_total_bytes = dir_size(&static_dir)?;
        let total_bytes = dir_size(data_dir)?;
        let segments = shard_segment_stats(data_dir)?;

        Ok(StorageDiskStats {
            total_bytes,
            meta_bytes,
            peers_bytes,
            static_total_bytes,
            segments,
        })
    }

    pub fn shard_size(&self) -> u64 {
        let meta = self.meta.lock();
        meta.shard_size
    }

    pub fn last_indexed_block(&self) -> Result<Option<u64>> {
        let meta = self.meta.lock();
        Ok(meta.max_present_block)
    }

    #[cfg(test)]
    pub fn set_last_indexed_block(&self, value: u64) -> Result<()> {
        let mut meta = self.meta.lock();
        meta.max_present_block = Some(value);
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

    pub fn head_seen(&self) -> Result<Option<u64>> {
        let meta = self.meta.lock();
        Ok(meta.head_seen)
    }

    pub fn set_head_seen(&self, value: u64) -> Result<()> {
        let mut meta = self.meta.lock();
        meta.head_seen = Some(value);
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

    pub fn has_block(&self, number: u64) -> Result<bool> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(false);
        };
        let state = shard.lock();
        let offset = (number - shard_start) as usize;
        Ok(state.bitset.is_set(offset))
    }

    pub fn missing_blocks_in_range(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<u64>> {
        let start = *range.start();
        let end = *range.end();
        if end < start {
            return Ok(Vec::new());
        }

        // Hot-path: this gets called for large ranges (e.g. 1M blocks). Avoid per-block shard map
        // lookups and locks; scan shard bitsets in larger chunks instead.
        let shard_size = self.shard_size().max(1);
        let first_shard = shard_start(start, shard_size);
        let last_shard = shard_start(end, shard_size);

        let mut out = Vec::new();
        let mut shard_start = first_shard;
        while shard_start <= last_shard {
            let shard_end = shard_start.saturating_add(shard_size).saturating_sub(1);
            let local_start = start.max(shard_start);
            let local_end = end.min(shard_end);
            let off_start = (local_start - shard_start) as usize;
            let off_end = (local_end - shard_start) as usize;

            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                out.extend(local_start..=local_end);
                shard_start = shard_start.saturating_add(shard_size);
                continue;
            };
            let state = shard.lock();
            if state.meta.complete {
                shard_start = shard_start.saturating_add(shard_size);
                continue;
            }

            let bytes = state.bitset.bytes();
            let start_byte = off_start / 8;
            let end_byte = off_end / 8;
            for byte_idx in start_byte..=end_byte {
                let mut mask = 0xFFu8;
                if byte_idx == start_byte {
                    mask &= 0xFFu8 << (off_start % 8);
                }
                if byte_idx == end_byte {
                    let end_bit = off_end % 8;
                    // Shifting by 8 would panic on `u8`; `end_bit == 7` means keep all bits.
                    mask &= if end_bit == 7 {
                        0xFF
                    } else {
                        (1u8 << (end_bit + 1)) - 1
                    };
                }

                let byte = bytes.get(byte_idx).copied().unwrap_or_default();
                let mut missing = (!byte) & mask;
                while missing != 0 {
                    let bit = missing.trailing_zeros() as usize;
                    let offset = byte_idx * 8 + bit;
                    out.push(shard_start + offset as u64);
                    missing &= missing - 1;
                }
            }

            shard_start = shard_start.saturating_add(shard_size);
        }

        Ok(out)
    }

    pub fn missing_ranges_in_range(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<(Vec<std::ops::RangeInclusive<u64>>, u64)> {
        let start = *range.start();
        let end = *range.end();
        if end < start {
            return Ok((Vec::new(), 0));
        }

        let shard_size = self.shard_size().max(1);
        let first_shard = shard_start(start, shard_size);
        let last_shard = shard_start(end, shard_size);

        let mut ranges = Vec::new();
        let mut missing = 0u64;
        let mut shard_start = first_shard;
        while shard_start <= last_shard {
            let shard_end = shard_start.saturating_add(shard_size).saturating_sub(1);
            let local_start = start.max(shard_start);
            let local_end = end.min(shard_end);
            let off_start = (local_start - shard_start) as usize;
            let off_end = (local_end - shard_start) as usize;

            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                ranges.push(local_start..=local_end);
                missing =
                    missing.saturating_add(local_end.saturating_sub(local_start).saturating_add(1));
                shard_start = shard_start.saturating_add(shard_size);
                continue;
            };
            let state = shard.lock();
            if state.meta.complete {
                shard_start = shard_start.saturating_add(shard_size);
                continue;
            }

            let mut current_start: Option<u64> = None;
            for offset in off_start..=off_end {
                let present = state.bitset.is_set(offset);
                if !present {
                    missing = missing.saturating_add(1);
                    if current_start.is_none() {
                        current_start = Some(shard_start + offset as u64);
                    }
                } else if let Some(start_block) = current_start.take() {
                    let end_block = shard_start + offset as u64 - 1;
                    ranges.push(start_block..=end_block);
                }
            }
            if let Some(start_block) = current_start {
                ranges.push(start_block..=local_end);
            }

            shard_start = shard_start.saturating_add(shard_size);
        }

        Ok((ranges, missing))
    }

    pub fn write_block_bundles_wal(&self, bundles: &[BlockBundle]) -> Result<Vec<u64>> {
        if bundles.is_empty() {
            return Ok(Vec::new());
        }
        let mut zstd = Compressor::new(0).wrap_err("failed to init zstd compressor")?;
        let mut per_shard: HashMap<u64, Vec<WalRecord>> = HashMap::new();
        let mut block_log_counts: HashMap<u64, u64> = HashMap::new();
        for bundle in bundles {
            let shard_start = shard_start(bundle.number, self.shard_size());
            // Remember log count per block while we still have decoded receipts
            block_log_counts.insert(
                bundle.number,
                bundle.receipts.receipts.iter().map(|r| r.logs.len() as u64).sum::<u64>(),
            );
            let tx_meta_uncompressed = encode_bincode_value(&bundle.transactions)?;
            let tx_meta_uncompressed_len = tx_meta_uncompressed.len() as u32;
            let receipts_uncompressed = encode_bincode_compat_value(&bundle.receipts)?;
            let receipts_uncompressed_len = receipts_uncompressed.len() as u32;
            let record = WalBundleRecord {
                number: bundle.number,
                header: encode_bincode_compat_value(&bundle.header)?,
                tx_hashes: encode_bincode_value(&bundle.tx_hashes)?,
                tx_meta: zstd
                    .compress(&tx_meta_uncompressed)
                    .wrap_err("failed to compress tx_meta")?,
                tx_meta_uncompressed_len,
                receipts: zstd
                    .compress(&receipts_uncompressed)
                    .wrap_err("failed to compress receipts")?,
                receipts_uncompressed_len,
                size: encode_u64_value(bundle.size.size),
            };
            let payload = encode_bincode_value(&record)?;
            per_shard.entry(shard_start).or_default().push(WalRecord {
                block_number: bundle.number,
                payload,
            });
        }

        let mut written_blocks = Vec::new();
        for (shard_start, records) in per_shard {
            let shard = self.get_or_create_shard(shard_start)?;
            let mut state = shard.lock();
            let mut to_append: Vec<WalRecord> = Vec::new();
            let mut seen_offsets: HashSet<usize> = HashSet::new();

            for record in records {
                let offset = (record.block_number - shard_start) as usize;
                if state.bitset.is_set(offset) || !seen_offsets.insert(offset) {
                    continue;
                }
                to_append.push(record);
            }

            if to_append.is_empty() {
                continue;
            }

            append_records(&wal_path(&state.dir), &to_append)?;

            // Track written blocks and mark shard as needing compaction
            // NOTE: We do NOT update the bitset here - it only reflects what's in the sorted segment.
            // The bitset will be rebuilt from actual data during compaction.
            let mut max_written: Option<u64> = None;
            for record in &to_append {
                written_blocks.push(record.block_number);
                max_written = Some(
                    max_written
                        .unwrap_or(record.block_number)
                        .max(record.block_number),
                );
            }

            // Accumulate log counts for actually appended blocks
            let logs_delta: u64 = to_append
                .iter()
                .map(|r| block_log_counts.get(&r.block_number).copied().unwrap_or(0))
                .sum();
            state.meta.total_logs += logs_delta;

            // Mark shard as not sorted (has pending WAL data) and persist
            if !to_append.is_empty() && state.meta.sorted {
                state.meta.sorted = false;
                state.meta.sealed = false;
                state.meta.content_hash = None;
                persist_shard_meta(&state.dir, &state.meta)?;
            } else if logs_delta > 0 {
                persist_shard_meta(&state.dir, &state.meta)?;
            }

            if let Some(max_written) = max_written {
                self.bump_max_present(max_written)?;
            }
        }
        Ok(written_blocks)
    }

    pub fn write_block_bundle_follow(&self, bundle: &BlockBundle) -> Result<()> {
        let shard_start = shard_start(bundle.number, self.shard_size());
        let shard = self.get_or_create_shard(shard_start)?;
        let mut state = shard.lock();
        let offset = (bundle.number - shard_start) as usize;
        if state.bitset.is_set(offset) {
            return Ok(());
        }
        let sorted_dir = sorted_dir(&state.dir);
        fs::create_dir_all(&sorted_dir).wrap_err("failed to create sorted dir")?;

        // Invalidate cache since we're writing to sorted segments
        self.invalidate_segment_cache(shard_start);

        let segments = shard_segment_writers(&sorted_dir, shard_start)?;
        segments.headers.append_rows(
            bundle.number,
            &[encode_bincode_compat_value(&bundle.header)?],
        )?;
        segments
            .tx_hashes
            .append_rows(bundle.number, &[encode_bincode_value(&bundle.tx_hashes)?])?;
        segments
            .tx_meta
            .append_rows(bundle.number, &[encode_bincode_value(&bundle.transactions)?])?;
        segments.receipts.append_rows(
            bundle.number,
            &[encode_bincode_compat_value(&bundle.receipts)?],
        )?;
        segments
            .sizes
            .append_rows(bundle.number, &[encode_u64_value(bundle.size.size)])?;

        if state.bitset.set(offset) {
            state.meta.present_count = state.meta.present_count.saturating_add(1);
            state.meta.total_transactions += bundle.tx_hashes.hashes.len() as u64;
            state.meta.total_receipts += bundle.receipts.receipts.len() as u64;
            state.meta.total_logs += bundle.receipts.receipts.iter().map(|r| r.logs.len() as u64).sum::<u64>();
            state.meta.complete = u64::from(state.meta.present_count) >= state.meta.shard_size;
        }
        state.meta.sorted = true;
        state.meta.sealed = false;
        state.meta.content_hash = None;
        state.meta.tail_block = Some(
            state
                .meta
                .tail_block
                .unwrap_or(shard_start)
                .max(bundle.number),
        );
        // Update disk sizes from sorted segment files
        let (dh, dt, dr, dtotal) = compute_shard_disk_bytes(&state.dir);
        state.meta.disk_bytes_headers = dh;
        state.meta.disk_bytes_transactions = dt;
        state.meta.disk_bytes_receipts = dr;
        state.meta.disk_bytes_total = dtotal;
        persist_shard_meta(&state.dir, &state.meta)?;
        state.bitset.flush(&bitset_path(&state.dir))?;
        self.bump_max_present(bundle.number)?;
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        clippy::cognitive_complexity,
        reason = "compaction has atomic phases (write/swap/cleanup) that must stay in sync"
    )]
    pub fn compact_shard(&self, shard_start: u64) -> Result<()> {
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(());
        };
        let mut state = shard.lock();
        let wal_p = wal_path(&state.dir);
        let wal_slices = build_slice_index(&wal_p, shard_start, state.meta.shard_size as usize)?;
        let has_wal = wal_slices.is_some();

        // If no WAL and already sorted, nothing to do
        if !has_wal && state.meta.sorted {
            return Ok(());
        }

        let sorted_dir = sorted_dir(&state.dir);
        let backup_dir = state.dir.join("sorted.old");
        let temp_dir = state.dir.join("sorted.tmp");
        let bitset_tmp = bitset_tmp_path(&state.dir);
        let bitset_old = bitset_old_path(&state.dir);
        let bitset_live = bitset_path(&state.dir);

        // ========== PHASE: WRITING ==========
        // Set phase and persist before making any changes
        state.meta.compaction_phase = Some(PHASE_WRITING.to_string());
        persist_shard_meta(&state.dir, &state.meta)?;

        // Clean up any leftover tmp files from previous interrupted compaction
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).wrap_err("failed to clean sorted.tmp")?;
        }
        if bitset_tmp.exists() {
            fs::remove_file(&bitset_tmp).wrap_err("failed to clean bitset.tmp")?;
        }
        fs::create_dir_all(&temp_dir).wrap_err("failed to create sorted.tmp")?;

        // Open existing segment if it exists
        let existing = if sorted_dir.exists() {
            Some(ShardRawSegments {
                headers: SegmentRawSource::<SegmentHeader>::open(&sorted_dir.join("headers"))?,
                tx_hashes: SegmentRawSource::<SegmentHeader>::open(&sorted_dir.join("tx_hashes"))?,
                tx_meta: SegmentRawSource::<SegmentHeader>::open(&sorted_dir.join("tx_meta"))?,
                receipts: SegmentRawSource::<SegmentHeader>::open(&sorted_dir.join("receipts"))?,
                sizes: SegmentRawSource::<SegmentHeader>::open(&sorted_dir.join("block_sizes"))?,
            })
        } else {
            None
        };

        // Create new bitset to track what we actually write (NOT based on old bitset)
        let shard_size = state.meta.shard_size as usize;
        let mut new_bitset = Bitset::new(shard_size);
        let mut actual_tail_block: Option<u64> = None;

        // Determine the range to iterate: max of WAL entries and existing segment rows
        let wal_max_offset = wal_slices.as_ref().and_then(|w| {
            w.entries
                .iter()
                .enumerate()
                .filter_map(|(i, e)| e.map(|_| i))
                .max()
        });
        let existing_rows = existing.as_ref().map_or(0, |e| e.headers.config().rows);
        let max_offset = wal_max_offset
            .unwrap_or(0)
            .max(existing_rows.saturating_sub(1));

        // If nothing to compact, clean up and return
        if max_offset == 0 && wal_max_offset.is_none() && existing_rows == 0 {
            if wal_p.exists() {
                fs::remove_file(&wal_p).wrap_err("failed to remove staging.wal")?;
            }
            fs::remove_dir_all(&temp_dir).wrap_err("failed to remove sorted.tmp")?;
            state.meta.sorted = true;
            state.meta.compaction_phase = None;
            persist_shard_meta(&state.dir, &state.meta)?;
            return Ok(());
        }

        let total_rows = max_offset.saturating_add(1);

        // Accumulators for tx/receipt counts (recomputed authoritatively at compaction)
        let mut total_tx: u64 = 0;
        let mut total_receipt_count: u64 = 0;

        // Create segment writers
        let mut writers = create_compaction_writers(&temp_dir, shard_start, existing.as_ref())?;

        // Iterate over all rows, checking WAL and existing segment for actual data
        // DO NOT use the old bitset - it may be out of sync
        for local_offset in 0..total_rows {
            // Check if this row has data in WAL
            let wal_data = wal_slices.as_ref().and_then(|w| w.entries.get(local_offset).copied().flatten());

            // Check if this row has COMPLETE data in existing segment
            // We must verify ALL segments have data, not just headers, because
            // a previous interrupted compaction may have left segments in an inconsistent state.
            let has_existing_data = if let Some(ref ex) = existing {
                let has_header = ex.headers.row_bytes(local_offset)?.is_some();
                let has_all = has_header
                    && ex.tx_hashes.row_bytes(local_offset)?.is_some()
                    && ex.tx_meta.row_bytes(local_offset)?.is_some()
                    && ex.receipts.row_bytes(local_offset)?.is_some()
                    && ex.sizes.row_bytes(local_offset)?.is_some();

                // Warn if we have partial data (header exists but other segments don't)
                if has_header && !has_all {
                    let block_number = shard_start + local_offset as u64;
                    tracing::warn!(
                        block = block_number,
                        "existing segment has partial data, will be skipped unless WAL has complete data"
                    );
                }
                has_all
            } else {
                false
            };

            if wal_data.is_none() && !has_existing_data {
                // No data for this row - write empty
                writers.push_empty()?;
                continue;
            }

            // This row has data - set bit in new bitset and track tail
            new_bitset.set(local_offset);
            let block_number = shard_start + local_offset as u64;
            actual_tail_block = Some(actual_tail_block.unwrap_or(block_number).max(block_number));

            // Prefer WAL data over existing segment data
            if let Some(slices) = wal_data {
                // SAFETY: wal_slices is always Some when we have wal_data
                let wal = wal_slices
                    .as_ref()
                    .ok_or_else(|| eyre!("wal_slices missing when wal_data present"))?;
                let header_bytes = &wal.mmap[slices.header.as_range()];
                let tx_hash_bytes = &wal.mmap[slices.tx_hashes.as_range()];
                let tx_meta_bytes = &wal.mmap[slices.tx_meta.as_range()];
                let receipt_bytes = &wal.mmap[slices.receipts.as_range()];
                let size_bytes = &wal.mmap[slices.size.as_range()];

                // Count txs from bincode-encoded StoredTxHashes (uncompressed, Vec<B256> prefix is u64 len)
                let tx_count = bincode_vec_len(tx_hash_bytes);
                total_tx += tx_count;
                total_receipt_count += tx_count; // receipts == transactions in Ethereum

                writers.headers.push_bytes(header_bytes, slices.header.len())?;
                writers.tx_hashes.push_bytes(tx_hash_bytes, slices.tx_hashes.len())?;
                writers.tx_meta.push_bytes(tx_meta_bytes, slices.tx_meta_uncompressed_len as usize)?;
                writers.receipts.push_bytes(receipt_bytes, slices.receipts_uncompressed_len as usize)?;
                writers.sizes.push_bytes(size_bytes, slices.size.len())?;
            } else {
                // Copy from existing segment (we verified all fields exist above)
                let ex = existing
                    .as_ref()
                    .ok_or_else(|| eyre!("existing segment missing for non-WAL block"))?;
                let header_bytes = ex
                    .headers
                    .row_bytes(local_offset)?
                    .ok_or_else(|| eyre!("missing header for block {}", block_number))?;
                let tx_hash_bytes = ex.tx_hashes.row_bytes(local_offset)?
                    .ok_or_else(|| eyre!("missing tx_hashes for block {}", block_number))?;
                let tx_meta_bytes = ex.tx_meta.row_bytes(local_offset)?
                    .ok_or_else(|| eyre!("missing tx_meta for block {}", block_number))?;
                let receipt_bytes = ex.receipts.row_bytes(local_offset)?
                    .ok_or_else(|| eyre!("missing receipts for block {}", block_number))?;
                let size_bytes = ex.sizes.row_bytes(local_offset)?
                    .ok_or_else(|| eyre!("missing size for block {}", block_number))?;

                // Count txs from bincode-encoded StoredTxHashes (uncompressed)
                let tx_count = bincode_vec_len(tx_hash_bytes);
                total_tx += tx_count;
                total_receipt_count += tx_count;

                writers.headers.push_bytes(header_bytes, header_bytes.len())?;
                writers.tx_hashes.push_bytes(tx_hash_bytes, tx_hash_bytes.len())?;
                // For zstd segments, the bytes are already compressed
                writers.tx_meta.push_bytes(tx_meta_bytes, 0)?;
                writers.receipts.push_bytes(receipt_bytes, 0)?;
                writers.sizes.push_bytes(size_bytes, size_bytes.len())?;
            }
        }

        writers.finish()?;

        // Write new bitset to tmp file
        new_bitset.flush(&bitset_tmp)?;

        // If no data was found at all, clean up and return
        let Some(tail_block) = actual_tail_block else {
            fs::remove_dir_all(&temp_dir).wrap_err("failed to remove empty sorted.tmp")?;
            fs::remove_file(&bitset_tmp).wrap_err("failed to remove empty bitset.tmp")?;
            if wal_p.exists() {
                fs::remove_file(&wal_p).wrap_err("failed to remove staging.wal")?;
            }
            state.meta.sorted = true;
            state.meta.compaction_phase = None;
            persist_shard_meta(&state.dir, &state.meta)?;
            return Ok(());
        };

        // ========== PHASE: SWAPPING ==========
        state.meta.compaction_phase = Some(PHASE_SWAPPING.to_string());
        persist_shard_meta(&state.dir, &state.meta)?;

        // Atomic swap for sorted directory
        if sorted_dir.exists() {
            if backup_dir.exists() {
                fs::remove_dir_all(&backup_dir).wrap_err("failed to remove stale sorted.old")?;
            }
            fs::rename(&sorted_dir, &backup_dir).wrap_err("failed to backup old sorted")?;
        }
        fs::rename(&temp_dir, &sorted_dir).wrap_err("failed to swap sorted dir")?;

        // Atomic swap for bitset
        if bitset_live.exists() {
            if bitset_old.exists() {
                fs::remove_file(&bitset_old).wrap_err("failed to remove stale bitset.old")?;
            }
            fs::rename(&bitset_live, &bitset_old).wrap_err("failed to backup old bitset")?;
        }
        fs::rename(&bitset_tmp, &bitset_live).wrap_err("failed to swap bitset")?;

        // ========== PHASE: CLEANUP ==========
        state.meta.compaction_phase = Some(PHASE_CLEANUP.to_string());
        persist_shard_meta(&state.dir, &state.meta)?;

        // Delete backups and WAL
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).wrap_err("failed to remove sorted.old")?;
        }
        if bitset_old.exists() {
            fs::remove_file(&bitset_old).wrap_err("failed to remove bitset.old")?;
        }
        if wal_p.exists() {
            fs::remove_file(&wal_p).wrap_err("failed to remove staging.wal")?;
        }

        // Invalidate segment cache since segments were rewritten
        self.invalidate_segment_cache(shard_start);

        // Update in-memory state with the new bitset
        state.bitset = new_bitset;
        state.meta.present_count = state.bitset.count_ones();
        state.meta.complete = u64::from(state.meta.present_count) >= state.meta.shard_size;
        state.meta.sorted = true;
        state.meta.sealed = false;
        state.meta.content_hash = None;
        state.meta.tail_block = Some(tail_block);
        state.meta.compaction_phase = None;
        // Authoritative tx/receipt counts recomputed during compaction
        state.meta.total_transactions = total_tx;
        state.meta.total_receipts = total_receipt_count;
        // Note: total_logs is NOT recomputed here (receipts are zstd-compressed).
        // It is preserved from WAL writes or set to 0 for fresh shards.
        // Recompute disk sizes from the freshly written segment files
        let (dh, dt, dr, dtotal) = compute_shard_disk_bytes(&state.dir);
        state.meta.disk_bytes_headers = dh;
        state.meta.disk_bytes_transactions = dt;
        state.meta.disk_bytes_receipts = dr;
        state.meta.disk_bytes_total = dtotal;
        persist_shard_meta(&state.dir, &state.meta)?;

        if state.meta.complete && !state.meta.sealed {
            self.seal_shard_locked(&mut state)?;
        }

        Ok(())
    }

    #[expect(dead_code, reason = "convenience wrapper for cases without progress tracking")]
    pub fn compact_all_dirty(&self) -> Result<()> {
        self.compact_all_dirty_with_progress(|_| {})
    }

    /// Compact all dirty shards, calling the progress callback after each shard is compacted.
    /// The callback receives the shard_start of the just-compacted shard.
    /// Only actually dirty shards (WAL present or not sorted) are processed and reported.
    pub fn compact_all_dirty_with_progress<F>(&self, mut on_shard_done: F) -> Result<()>
    where
        F: FnMut(u64),
    {
        // Get only dirty shards (WAL present or not sorted)
        let dirty = self.dirty_shards()?;
        for info in dirty {
            self.compact_shard(info.shard_start)?;
            on_shard_done(info.shard_start);
        }
        Ok(())
    }

    /// Returns shard starts that are complete but still have WAL data and/or are not compacted.
    pub fn dirty_complete_shards(&self) -> Result<Vec<u64>> {
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock();
            shards.keys().copied().collect()
        };
        let mut out = Vec::new();
        for shard_start in shard_starts {
            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                continue;
            };
            let state = shard.lock();
            if !state.meta.complete {
                continue;
            }
            let wal_exists = wal_path(&state.dir).exists();
            if wal_exists || !state.meta.sorted {
                out.push(shard_start);
            }
        }
        out.sort_unstable();
        Ok(out)
    }

    pub fn dirty_shards(&self) -> Result<Vec<DirtyShardInfo>> {
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock();
            shards.keys().copied().collect()
        };
        let mut out = Vec::new();
        for shard_start in shard_starts {
            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                continue;
            };
            let state = shard.lock();
            let wal_path = wal_path(&state.dir);
            let wal_bytes = wal_path.metadata().map(|meta| meta.len()).unwrap_or(0);
            let is_dirty = wal_bytes > 0 || !state.meta.sorted;
            if is_dirty {
                out.push(DirtyShardInfo {
                    shard_start,
                    wal_bytes,
                });
            }
        }
        out.sort_by_key(|info| info.shard_start);
        Ok(out)
    }

    #[expect(dead_code, reason = "convenience wrapper for progress-less sealing")]
    pub fn seal_completed_shards(&self) -> Result<()> {
        self.seal_completed_shards_with_progress(|_| {})
    }

    /// Seal all completed shards, calling progress callback after each.
    pub fn seal_completed_shards_with_progress<F>(&self, mut on_sealed: F) -> Result<()>
    where
        F: FnMut(u64),
    {
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock();
            shards.keys().copied().collect()
        };
        for shard_start in shard_starts {
            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                continue;
            };
            let mut state = shard.lock();
            if state.meta.complete && state.meta.sorted && !state.meta.sealed {
                self.seal_shard_locked(&mut state)?;
                persist_shard_meta(&state.dir, &state.meta)?;
                on_sealed(shard_start);
            }
        }
        Ok(())
    }

    /// Count shards that are ready to be sealed (complete, sorted, not sealed).
    pub fn count_shards_to_seal(&self) -> Result<u64> {
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock();
            shards.keys().copied().collect()
        };
        let mut count = 0u64;
        for shard_start in shard_starts {
            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else { continue };
            let state = shard.lock();
            if state.meta.complete && state.meta.sorted && !state.meta.sealed {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn rollback_to(&self, ancestor_number: u64) -> Result<()> {
        let shard_size = self.shard_size();
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock();
            shards.keys().copied().collect()
        };
        for shard_start in shard_starts {
            let shard_end = shard_start.saturating_add(shard_size).saturating_sub(1);
            if ancestor_number >= shard_end {
                continue;
            }
            let shard = self.get_or_create_shard(shard_start)?;
            let mut state = shard.lock();
            let sorted_dir = sorted_dir(&state.dir);
            if sorted_dir.exists() {
                // Invalidate cache before pruning since segments will change
                self.invalidate_segment_cache(shard_start);
                let segments = shard_segment_writers(&sorted_dir, shard_start)?;
                segments.headers.prune_to(ancestor_number)?;
                segments.tx_hashes.prune_to(ancestor_number)?;
                segments.tx_meta.prune_to(ancestor_number)?;
                segments.receipts.prune_to(ancestor_number)?;
                segments.sizes.prune_to(ancestor_number)?;
            }
            let start_offset = if ancestor_number < shard_start {
                0
            } else {
                (ancestor_number
                    .saturating_sub(shard_start)
                    .saturating_add(1)) as usize
            };
            for offset in start_offset..state.meta.shard_size as usize {
                if state.bitset.clear(offset) {
                    state.meta.present_count = state.meta.present_count.saturating_sub(1);
                }
            }
            state.meta.complete = u64::from(state.meta.present_count) >= state.meta.shard_size;
            if ancestor_number < shard_start {
                state.meta.tail_block = None;
            } else {
                state.meta.tail_block = Some(ancestor_number);
            }
            state.meta.sealed = false;
            state.meta.content_hash = None;
            persist_shard_meta(&state.dir, &state.meta)?;
            state.bitset.flush(&bitset_path(&state.dir))?;
        }
        self.recompute_max_present()?;
        Ok(())
    }

    /// Invalidate the segment cache for a specific shard.
    /// Call this after any operation that modifies segment files.
    fn invalidate_segment_cache(&self, shard_start: u64) {
        let mut cache = self.segment_cache.lock();
        cache.pop(&shard_start);
    }

    /// Get cached segment readers for a shard, opening them if not cached.
    fn get_cached_segments(
        &self,
        shard_start: u64,
        sorted_dir: &Path,
    ) -> Result<Option<Arc<ShardSegmentReaders>>> {
        // Check cache first
        {
            let mut cache = self.segment_cache.lock();
            if let Some(segments) = cache.get(&shard_start) {
                return Ok(Some(Arc::clone(segments)));
            }
        }

        // Cache miss - open segments (read-only, no file locks)
        let segments = shard_segment_readers(sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };

        // Cache and return
        let segments = Arc::new(segments);
        {
            let mut cache = self.segment_cache.lock();
            cache.put(shard_start, Arc::clone(&segments));
        }
        Ok(Some(segments))
    }

    fn with_readers_for_present_block<T>(
        &self,
        number: u64,
        f: impl FnOnce(&ShardSegmentReaders) -> Result<Option<T>>,
    ) -> Result<Option<T>> {
        let shard_start = shard_start(number, self.shard_size());

        // Check bitset first (fast path) and get sorted_dir
        let sorted_dir = {
            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                return Ok(None);
            };
            let state = shard.lock();
            let offset = (number - shard_start) as usize;
            if !state.bitset.is_set(offset) {
                return Ok(None);
            }
            sorted_dir(&state.dir)
        }; // Release shard lock before cache access

        // Get cached segments
        let segments = self.get_cached_segments(shard_start, &sorted_dir)?;
        let Some(segments) = segments else {
            return Ok(None);
        };

        f(&segments)
    }

    pub fn block_header(&self, number: u64) -> Result<Option<Header>> {
        self.with_readers_for_present_block(number, |segments| {
            segments.headers.read_row_compat(number)
        })
    }

    pub fn block_tx_hashes(&self, number: u64) -> Result<Option<StoredTxHashes>> {
        self.with_readers_for_present_block(number, |segments| segments.tx_hashes.read_row(number))
    }

    pub fn block_size(&self, number: u64) -> Result<Option<StoredBlockSize>> {
        self.with_readers_for_present_block(number, |segments| {
            let size = segments.sizes.read_row_u64(number)?;
            Ok(size.map(|size| StoredBlockSize { size }))
        })
    }

    pub fn block_receipts(&self, number: u64) -> Result<Option<StoredReceipts>> {
        self.with_readers_for_present_block(number, |segments| {
            segments.receipts.read_row_compat(number)
        })
    }

    pub fn block_headers_range(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<(u64, Header)>> {
        let mut out = Vec::new();
        for block in range {
            if let Some(header) = self.block_header(block)? {
                out.push((block, header));
            }
        }
        Ok(out)
    }

    pub fn block_tx_hashes_range(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<(u64, StoredTxHashes)>> {
        let mut out = Vec::new();
        for block in range {
            if let Some(hashes) = self.block_tx_hashes(block)? {
                out.push((block, hashes));
            }
        }
        Ok(out)
    }

    pub fn block_receipts_range(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<(u64, StoredReceipts)>> {
        let mut out = Vec::new();
        for block in range {
            if let Some(receipts) = self.block_receipts(block)? {
                out.push((block, receipts));
            }
        }
        Ok(out)
    }

    pub fn upsert_peer(&self, mut peer: StoredPeer) -> Result<()> {
        let mut cache = self.peer_cache.lock();
        if let Some(existing) = cache.get(&peer.peer_id) {
            if peer.aimd_batch_limit.is_none() {
                peer.aimd_batch_limit = existing.aimd_batch_limit;
            }
        }
        cache.insert(peer.peer_id.clone(), peer);
        Ok(())
    }

    pub fn flush_peer_cache(&self) -> Result<()> {
        let cache = self.peer_cache.lock();
        let peers: Vec<StoredPeer> = cache.values().cloned().collect();
        persist_peer_cache(&self.peer_cache_dir, &peers)
    }

    pub fn load_peers(&self, expire_before_ms: u64, max_peers: usize) -> Result<PeerCacheLoad> {
        let mut cache = self.peer_cache.lock();
        let mut peers: Vec<StoredPeer> = Vec::new();
        let total = cache.len();
        let mut expired = 0usize;
        let corrupted = 0usize;
        let mut capped = 0usize;
        for peer in cache.values() {
            if peer.last_seen_ms < expire_before_ms {
                expired += 1;
                continue;
            }
            peers.push(peer.clone());
        }
        peers.sort_by_key(|peer| Reverse(peer.last_seen_ms));
        if peers.len() > max_peers {
            capped = peers.len().saturating_sub(max_peers);
            peers.truncate(max_peers);
        }
        cache.clear();
        for peer in &peers {
            cache.insert(peer.peer_id.clone(), peer.clone());
        }
        drop(cache);
        persist_peer_cache(&self.peer_cache_dir, &peers)?;
        Ok(PeerCacheLoad {
            peers,
            total,
            expired,
            corrupted,
            capped,
        })
    }

    pub fn peer_cache_snapshot(&self) -> Vec<StoredPeer> {
        let cache = self.peer_cache.lock();
        cache.values().cloned().collect()
    }

    pub fn update_peer_batch_limits(&self, limits: &[(String, u64)]) {
        let mut cache = self.peer_cache.lock();
        for (peer_id, limit) in limits {
            if let Some(peer) = cache.get_mut(peer_id) {
                peer.aimd_batch_limit = Some(*limit);
            }
        }
    }

    fn get_shard(&self, shard_start: u64) -> Result<Option<Arc<Mutex<ShardState>>>> {
        let shards = self.shards.lock();
        Ok(shards.get(&shard_start).cloned())
    }

    fn get_or_create_shard(&self, shard_start: u64) -> Result<Arc<Mutex<ShardState>>> {
        if let Some(shard) = self.get_shard(shard_start)? {
            return Ok(shard);
        }
        let mut shards = self.shards.lock();
        if let Some(shard) = shards.get(&shard_start) {
            return Ok(Arc::clone(shard));
        }
        let dir = shard_dir(&self.data_dir, shard_start);
        fs::create_dir_all(&dir).wrap_err("failed to create shard dir")?;
        let bitset = Bitset::new(self.shard_size() as usize);
        bitset.flush(&bitset_path(&dir))?;
        let meta = ShardMeta {
            shard_start,
            shard_size: self.shard_size(),
            present_count: 0,
            complete: false,
            sorted: true,
            sealed: false,
            tail_block: None,
            content_hash: None,
            content_hash_algo: "sha256".to_string(),
            compaction_phase: None,
            total_transactions: 0,
            total_receipts: 0,
            total_logs: 0,
            disk_bytes_headers: 0,
            disk_bytes_transactions: 0,
            disk_bytes_receipts: 0,
            disk_bytes_total: 0,
        };
        persist_shard_meta(&dir, &meta)?;
        let state = ShardState { dir, meta, bitset };
        let shard = Arc::new(Mutex::new(state));
        shards.insert(shard_start, Arc::clone(&shard));
        Ok(shard)
    }

    fn bump_max_present(&self, block: u64) -> Result<()> {
        let mut meta = self.meta.lock();
        match meta.max_present_block {
            None => {
                meta.max_present_block = Some(block);
                persist_meta(&meta_path(&self.data_dir), &meta)?;
            }
            Some(current) => {
                if block > current {
                    meta.max_present_block = Some(block);
                    persist_meta(&meta_path(&self.data_dir), &meta)?;
                }
            }
        }
        Ok(())
    }

    fn recompute_max_present(&self) -> Result<()> {
        let shard_size = self.shard_size();
        let shards = self.shards.lock();
        let mut max_present = None;
        for shard in shards.values() {
            let state = shard.lock();
            if let Some(max_block) = max_present_in_bitset(&state, shard_size) {
                max_present = Some(max_present.unwrap_or(0).max(max_block));
            }
        }
        let mut meta = self.meta.lock();
        meta.max_present_block = max_present;
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

    #[expect(clippy::unused_self, reason = "method signature for consistency")]
    fn seal_shard_locked(&self, state: &mut ShardState) -> Result<()> {
        let Some(tail_block) = state.meta.tail_block else {
            return Ok(());
        };
        let sorted_dir = sorted_dir(&state.dir);
        let hash = compute_shard_hash(
            state.meta.shard_start,
            state.meta.shard_size as u32,
            tail_block,
            state.bitset.bytes(),
            &sorted_dir,
        )?;
        state.meta.sealed = true;
        state.meta.content_hash = Some(hash);
        state.meta.content_hash_algo = "sha256".to_string();
        Ok(())
    }
}

struct ShardSegments {
    headers: SegmentWriter,
    tx_hashes: SegmentWriter,
    tx_meta: SegmentWriter,
    receipts: SegmentWriter,
    sizes: SegmentWriter,
}

/// Read-only segment handle. Unlike `SegmentWriter`, this does NOT open a
/// `NippyJarWriter` (which takes an exclusive file lock). Each read loads the
/// jar fresh – the same thing `SegmentWriter::read_row_raw` already does – so
/// multiple `SegmentReader`s can coexist with an active writer.
struct SegmentReader {
    path: PathBuf,
}

impl SegmentReader {
    fn open(path: PathBuf, start_block: u64) -> Result<Self> {
        if !segment_exists(&path) {
            return Err(eyre!("segment not found: {}", path.display()));
        }
        let jar = NippyJar::<SegmentHeader>::load(&path)
            .wrap_err("failed to load static segment")?;
        let header = jar.user_header();
        if header.start_block != start_block {
            return Err(eyre!(
                "segment start mismatch for {} (expected {}, got {})",
                path.display(),
                start_block,
                header.start_block
            ));
        }
        Ok(Self { path })
    }

    fn read_row_raw(&self, block_number: u64) -> Result<Option<Vec<u8>>> {
        let jar = NippyJar::<SegmentHeader>::load(&self.path)
            .wrap_err("failed to load static segment")?;
        let header = jar.user_header();
        if block_number < header.start_block {
            return Ok(None);
        }
        let row = block_number.saturating_sub(header.start_block) as usize;
        if row >= jar.rows() {
            return Ok(None);
        }
        let mut cursor = NippyJarCursor::new(&jar).wrap_err("failed to open segment cursor")?;
        let Some(row_vals) = cursor.row_by_number(row)? else {
            return Ok(None);
        };
        let bytes = row_vals
            .first()
            .ok_or_else(|| eyre!("missing column data"))?;
        if bytes.is_empty() {
            return Ok(None);
        }
        Ok(Some(bytes.to_vec()))
    }

    fn read_row_compat<T>(&self, block_number: u64) -> Result<Option<T>>
    where
        T: SerdeBincodeCompat,
        <T as SerdeBincodeCompat>::BincodeRepr<'static>: serde::de::DeserializeOwned,
    {
        let Some(bytes) = self.read_row_raw(block_number)? else {
            return Ok(None);
        };
        decode_bincode_compat_value(&bytes).map(Some)
    }

    fn read_row<T: serde::de::DeserializeOwned>(&self, block_number: u64) -> Result<Option<T>> {
        let Some(bytes) = self.read_row_raw(block_number)? else {
            return Ok(None);
        };
        decode_bincode(&bytes).map(Some)
    }

    fn read_row_u64(&self, block_number: u64) -> Result<Option<u64>> {
        let Some(bytes) = self.read_row_raw(block_number)? else {
            return Ok(None);
        };
        decode_u64(&bytes).map(Some)
    }
}

/// Container for read-only segment handles for a single shard.
#[expect(dead_code, reason = "tx_meta included for completeness; reads not yet wired")]
struct ShardSegmentReaders {
    headers: SegmentReader,
    tx_hashes: SegmentReader,
    tx_meta: SegmentReader,
    receipts: SegmentReader,
    sizes: SegmentReader,
}

fn shard_segment_writers(sorted_dir: &Path, shard_start: u64) -> Result<ShardSegments> {
    Ok(ShardSegments {
        headers: SegmentWriter::open(
            sorted_dir.join("headers"),
            shard_start,
            SegmentCompression::None,
        )?,
        tx_hashes: SegmentWriter::open(
            sorted_dir.join("tx_hashes"),
            shard_start,
            SegmentCompression::None,
        )?,
        tx_meta: SegmentWriter::open(
            sorted_dir.join("tx_meta"),
            shard_start,
            SegmentCompression::Zstd {
                use_dict: false,
                max_dict_size: ZSTD_DICT_MAX_SIZE,
            },
        )?,
        receipts: SegmentWriter::open(
            sorted_dir.join("receipts"),
            shard_start,
            SegmentCompression::Zstd {
                use_dict: false,
                max_dict_size: ZSTD_DICT_MAX_SIZE,
            },
        )?,
        sizes: SegmentWriter::open(
            sorted_dir.join("block_sizes"),
            shard_start,
            SegmentCompression::None,
        )?,
    })
}

fn shard_segment_readers(sorted_dir: &Path, shard_start: u64) -> Result<Option<ShardSegmentReaders>> {
    if !sorted_dir.exists() {
        return Ok(None);
    }
    let required = ["headers", "tx_hashes", "tx_meta", "receipts", "block_sizes"];
    for name in required {
        let base = sorted_dir.join(name);
        if !segment_exists(&base) {
            return Err(eyre!("missing segment {}", base.display()));
        }
    }
    Ok(Some(ShardSegmentReaders {
        headers: SegmentReader::open(sorted_dir.join("headers"), shard_start)?,
        tx_hashes: SegmentReader::open(sorted_dir.join("tx_hashes"), shard_start)?,
        tx_meta: SegmentReader::open(sorted_dir.join("tx_meta"), shard_start)?,
        receipts: SegmentReader::open(sorted_dir.join("receipts"), shard_start)?,
        sizes: SegmentReader::open(sorted_dir.join("block_sizes"), shard_start)?,
    }))
}

const fn shard_start(block_number: u64, shard_size: u64) -> u64 {
    (block_number / shard_size) * shard_size
}

fn segment_exists(path: &Path) -> bool {
    path.with_extension(CONFIG_FILE_EXTENSION).exists()
}

fn meta_path(data_dir: &Path) -> PathBuf {
    data_dir.join(META_FILE_NAME)
}

fn static_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(STATIC_DIR_NAME)
}

fn shards_dir(data_dir: &Path) -> PathBuf {
    static_dir(data_dir).join(SHARDS_DIR_NAME)
}

fn shard_dir(data_dir: &Path, shard_start: u64) -> PathBuf {
    shards_dir(data_dir).join(shard_start.to_string())
}

fn shard_meta_path(shard_dir: &Path) -> PathBuf {
    shard_dir.join(SHARD_META_NAME)
}

fn bitset_path(shard_dir: &Path) -> PathBuf {
    shard_dir.join(BITSET_NAME)
}

fn bitset_tmp_path(shard_dir: &Path) -> PathBuf {
    shard_dir.join(BITSET_TMP_NAME)
}

fn bitset_old_path(shard_dir: &Path) -> PathBuf {
    shard_dir.join(BITSET_OLD_NAME)
}

fn wal_path(shard_dir: &Path) -> PathBuf {
    shard_dir.join("state").join(WAL_NAME)
}

fn sorted_dir(shard_dir: &Path) -> PathBuf {
    shard_dir.join("sorted")
}

fn peer_path(data_dir: &Path) -> PathBuf {
    data_dir.join(PEER_FILE_NAME)
}

fn load_meta(path: &Path) -> Result<MetaState> {
    let bytes = fs::read(path).wrap_err("failed to read meta.json")?;
    serde_json::from_slice(&bytes).wrap_err("failed to decode meta.json")
}

fn persist_meta(path: &Path, meta: &MetaState) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(meta).wrap_err("failed to encode meta.json")?;
    fs::write(path, bytes).wrap_err("failed to write meta.json")
}

fn load_shard_meta(shard_dir: &Path) -> Result<ShardMeta> {
    let path = shard_meta_path(shard_dir);
    let bytes = fs::read(&path).wrap_err("failed to read shard.json")?;
    serde_json::from_slice(&bytes).wrap_err("failed to decode shard.json")
}

fn persist_shard_meta(shard_dir: &Path, meta: &ShardMeta) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(meta).wrap_err("failed to encode shard.json")?;
    fs::write(shard_meta_path(shard_dir), bytes).wrap_err("failed to write shard.json")
}

fn validate_meta(config: &NodeConfig, meta: &MetaState) -> Result<()> {
    if meta.chain_id != config.chain_id {
        return Err(eyre!(
            "chain id mismatch: expected {}, got {}",
            config.chain_id,
            meta.chain_id
        ));
    }
    if meta.storage_key != StorageConfigKey::from(config) {
        return Err(eyre!(
            "storage config mismatch: expected {:?}, got {:?}",
            StorageConfigKey::from(config),
            meta.storage_key
        ));
    }
    if meta.shard_size != config.shard_size {
        return Err(eyre!(
            "shard size mismatch: expected {}, got {}",
            config.shard_size,
            meta.shard_size
        ));
    }
    Ok(())
}

fn file_size(path: &Path) -> Result<u64> {
    match fs::metadata(path) {
        Ok(meta) => Ok(meta.len()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(err.into()),
    }
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total: u64 = 0;
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(err) => return Err(err.into()),
    };
    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            total = total.saturating_add(dir_size(&entry.path())?);
        } else {
            total = total.saturating_add(meta.len());
        }
    }
    Ok(total)
}

fn segment_disk_size(base: &Path) -> Result<u64> {
    let mut total: u64 = 0;
    total = total.saturating_add(file_size(base)?);
    total = total.saturating_add(file_size(&base.with_extension("off"))?);
    total = total.saturating_add(file_size(&base.with_extension(CONFIG_FILE_EXTENSION))?);
    total = total.saturating_add(file_size(&base.with_extension("idx"))?);
    Ok(total)
}

fn shard_segment_stats(data_dir: &Path) -> Result<Vec<SegmentDiskStats>> {
    let shards_root = shards_dir(data_dir);
    let names = ["headers", "tx_hashes", "tx_meta", "receipts", "block_sizes"];
    let mut totals: HashMap<&str, u64> = HashMap::new();
    if !shards_root.exists() {
        return Ok(Vec::new());
    }
    for entry in fs::read_dir(shards_root)? {
        let entry = entry?;
        if !entry.metadata()?.is_dir() {
            continue;
        }
        let sorted = entry.path().join("sorted");
        if !sorted.exists() {
            continue;
        }
        for name in names {
            let base = sorted.join(name);
            let bytes = segment_disk_size(&base)?;
            *totals.entry(name).or_insert(0) += bytes;
        }
    }
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        out.push(SegmentDiskStats {
            name: name.to_string(),
            bytes: *totals.get(name).unwrap_or(&0),
        });
    }
    Ok(out)
}

/// Compute per-segment and total disk bytes for a shard directory.
/// Returns `(headers, transactions, receipts, total)` byte counts.
fn compute_shard_disk_bytes(shard_dir: &Path) -> (u64, u64, u64, u64) {
    let sorted = sorted_dir(shard_dir);
    if !sorted.exists() {
        return (0, 0, 0, 0);
    }
    let headers = segment_disk_size(&sorted.join("headers")).unwrap_or(0);
    let tx_hashes = segment_disk_size(&sorted.join("tx_hashes")).unwrap_or(0);
    let tx_meta = segment_disk_size(&sorted.join("tx_meta")).unwrap_or(0);
    let receipts = segment_disk_size(&sorted.join("receipts")).unwrap_or(0);
    let block_sizes = segment_disk_size(&sorted.join("block_sizes")).unwrap_or(0);
    let transactions = tx_hashes.saturating_add(tx_meta);
    let total = headers
        .saturating_add(transactions)
        .saturating_add(receipts)
        .saturating_add(block_sizes);
    (headers, transactions, receipts, total)
}

/// Extract the vector length from a bincode-encoded `Vec<T>`.
/// Bincode encodes a Vec as a u64 length prefix (little-endian) followed by elements.
/// Returns 0 if the bytes are too short.
fn bincode_vec_len(bytes: &[u8]) -> u64 {
    if bytes.len() < 8 {
        return 0;
    }
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

fn load_peer_cache(data_dir: &Path) -> Result<HashMap<String, StoredPeer>> {
    let path = peer_path(data_dir);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let bytes = fs::read(&path).wrap_err("failed to read peers.json")?;
    let peers: Vec<StoredPeer> =
        serde_json::from_slice(&bytes).wrap_err("failed to decode peers.json")?;
    Ok(peers
        .into_iter()
        .map(|peer| (peer.peer_id.clone(), peer))
        .collect())
}

fn persist_peer_cache(data_dir: &Path, peers: &[StoredPeer]) -> Result<()> {
    let path = peer_path(data_dir);
    let bytes = serde_json::to_vec_pretty(peers).wrap_err("failed to encode peers.json")?;
    fs::write(&path, bytes).wrap_err("failed to write peers.json")
}

fn max_present_in_bitset(state: &ShardState, shard_size: u64) -> Option<u64> {
    let mut max_offset = None;
    let size = shard_size as usize;
    for offset in (0..size).rev() {
        if state.bitset.is_set(offset) {
            max_offset = Some(offset);
            break;
        }
    }
    max_offset.map(|offset| state.meta.shard_start + offset as u64)
}

fn repair_shard_from_wal(state: &mut ShardState) -> Result<()> {
    let wal_path = wal_path(&state.dir);
    if !wal_path.exists() {
        return Ok(());
    }
    // WAL exists - mark shard as needing compaction.
    // NOTE: We do NOT update the bitset here. The bitset only reflects what's in the sorted
    // segment. When compaction runs, it will rebuild the bitset from actual segment data + WAL.
    state.meta.sorted = false;
    state.meta.sealed = false;
    state.meta.content_hash = None;
    Ok(())
}

/// Result of shard recovery operation
#[derive(Debug, Clone)]
pub enum ShardRecoveryResult {
    /// No recovery needed
    Clean,
    /// Recovered from interrupted "writing" phase
    RecoveredWriting,
    /// Recovered from interrupted "swapping" phase
    RecoveredSwapping,
    /// Recovered from interrupted "cleanup" phase
    RecoveredCleanup,
    /// Cleaned up orphan files (no phase was set but tmp files existed)
    CleanedOrphans,
}

impl ShardRecoveryResult {
    /// Returns true if recovery was needed
    pub const fn needs_recovery(&self) -> bool {
        !matches!(self, Self::Clean)
    }

    /// Returns a human-readable description
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Clean => "OK",
            Self::RecoveredWriting => "recovered from interrupted write phase",
            Self::RecoveredSwapping => "recovered from interrupted swap phase",
            Self::RecoveredCleanup => "completed interrupted cleanup",
            Self::CleanedOrphans => "cleaned orphan files",
        }
    }
}

/// Info about a single shard's repair result
#[derive(Debug, Clone)]
pub struct ShardRepairInfo {
    /// Shard start block
    pub shard_start: u64,
    /// What recovery was performed
    pub result: ShardRecoveryResult,
    /// The compaction phase that was interrupted (if any)
    pub original_phase: Option<String>,
}

/// Summary report of storage repair operation
#[derive(Debug, Clone)]
pub struct RepairReport {
    /// Per-shard repair results
    pub shards: Vec<ShardRepairInfo>,
}

impl RepairReport {
    /// Count of shards that needed repair
    pub fn repaired_count(&self) -> usize {
        self.shards.iter().filter(|s| s.result.needs_recovery()).count()
    }

    /// Count of shards that were already clean
    pub fn clean_count(&self) -> usize {
        self.shards.iter().filter(|s| !s.result.needs_recovery()).count()
    }

    /// Total shard count
    pub const fn total_count(&self) -> usize {
        self.shards.len()
    }
}

/// Recover a shard from an interrupted compaction based on its phase.
/// Returns what kind of recovery was performed (for UI/logging).
#[expect(clippy::cognitive_complexity, reason = "recovery handles multiple interrupted states")]
fn recover_shard(shard_dir: &Path, meta: &mut ShardMeta) -> Result<ShardRecoveryResult> {
    let sorted = sorted_dir(shard_dir);
    let backup = shard_dir.join("sorted.old");
    let temp = shard_dir.join("sorted.tmp");
    let bitset_tmp = bitset_tmp_path(shard_dir);
    let bitset_old = bitset_old_path(shard_dir);
    let bitset_live = bitset_path(shard_dir);

    let phase = meta.compaction_phase.as_deref();

    match phase {
        Some(PHASE_WRITING) => {
            // Interrupted while writing tmp files - discard partial work
            if temp.exists() {
                fs::remove_dir_all(&temp).wrap_err("failed to remove sorted.tmp")?;
            }
            if bitset_tmp.exists() {
                fs::remove_file(&bitset_tmp).wrap_err("failed to remove bitset.tmp")?;
            }
            // Reset phase - will trigger re-compaction if WAL exists
            meta.compaction_phase = None;
            Ok(ShardRecoveryResult::RecoveredWriting)
        }

        Some(PHASE_SWAPPING) => {
            // Interrupted during atomic swaps - restore to pre-swap state
            // Step 1: Ensure sorted/ is valid
            if !sorted.exists() && backup.exists() {
                fs::rename(&backup, &sorted).wrap_err("failed to restore sorted.old")?;
            }
            // Step 2: Ensure bitset is valid
            if !bitset_live.exists() && bitset_old.exists() {
                fs::rename(&bitset_old, &bitset_live).wrap_err("failed to restore bitset.old")?;
            }
            // Step 3: Clean up all tmp/old files
            if temp.exists() {
                fs::remove_dir_all(&temp).wrap_err("failed to remove sorted.tmp")?;
            }
            if backup.exists() {
                fs::remove_dir_all(&backup).wrap_err("failed to remove sorted.old")?;
            }
            if bitset_tmp.exists() {
                fs::remove_file(&bitset_tmp).wrap_err("failed to remove bitset.tmp")?;
            }
            if bitset_old.exists() {
                fs::remove_file(&bitset_old).wrap_err("failed to remove bitset.old")?;
            }
            // Reset phase and mark as needing compaction
            meta.compaction_phase = None;
            meta.sorted = false;
            Ok(ShardRecoveryResult::RecoveredSwapping)
        }

        Some(PHASE_CLEANUP) => {
            // Interrupted during cleanup - just finish the cleanup
            if backup.exists() {
                fs::remove_dir_all(&backup).wrap_err("failed to remove sorted.old")?;
            }
            if bitset_old.exists() {
                fs::remove_file(&bitset_old).wrap_err("failed to remove bitset.old")?;
            }
            let wal = wal_path(shard_dir);
            if wal.exists() {
                fs::remove_file(&wal).wrap_err("failed to remove staging.wal")?;
            }
            // Complete the phase
            meta.compaction_phase = None;
            meta.sorted = true;
            Ok(ShardRecoveryResult::RecoveredCleanup)
        }

        None => {
            // No phase set, but check for orphan files from hard crashes
            let mut had_orphans = false;

            // Handle sorted.old - if it exists, compaction might have succeeded
            if backup.exists() && sorted.exists() {
                fs::remove_dir_all(&backup).wrap_err("failed to remove orphan sorted.old")?;
                had_orphans = true;
            } else if backup.exists() && !sorted.exists() {
                fs::rename(&backup, &sorted).wrap_err("failed to restore orphan sorted.old")?;
                had_orphans = true;
            }

            // Clean up any tmp files
            if temp.exists() {
                fs::remove_dir_all(&temp).wrap_err("failed to remove orphan sorted.tmp")?;
                had_orphans = true;
            }
            if bitset_tmp.exists() {
                fs::remove_file(&bitset_tmp).wrap_err("failed to remove orphan bitset.tmp")?;
                had_orphans = true;
            }
            if bitset_old.exists() {
                fs::remove_file(&bitset_old).wrap_err("failed to remove orphan bitset.old")?;
                had_orphans = true;
            }

            if had_orphans {
                Ok(ShardRecoveryResult::CleanedOrphans)
            } else {
                Ok(ShardRecoveryResult::Clean)
            }
        }

        Some(unknown) => {
            // Unknown phase - treat as orphan state
            tracing::warn!(shard = %shard_dir.display(), phase = %unknown, "unknown compaction phase, resetting");
            meta.compaction_phase = None;
            meta.sorted = false;
            Ok(ShardRecoveryResult::CleanedOrphans)
        }
    }
}

fn reconcile_shard_meta(state: &mut ShardState) {
    let count = state.bitset.count_ones();
    state.meta.present_count = count;
    state.meta.complete = u64::from(count) >= state.meta.shard_size;
}

fn default_hash_algo() -> String {
    "sha256".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
    use crate::storage::StoredTransactions;
    use reth_ethereum_primitives::Receipt;
    use reth_primitives_traits::Header;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> std::path::PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "stateless-history-node-sharded-test-{now}-{}-{suffix}",
            std::process::id()
        ));
        path
    }

    fn base_config(data_dir: PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir,
            peer_cache_dir: None,
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            shard_size: crate::cli::DEFAULT_SHARD_SIZE,
            end_block: None,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
            verbosity: 0,
            run_name: None,
            log: false,
            log_output_dir: PathBuf::from(crate::cli::DEFAULT_LOG_OUTPUT_DIR),
            log_trace: false,
            log_trace_filter: crate::cli::DEFAULT_LOG_TRACE_FILTER.to_string(),
            log_trace_include_args: false,
            log_trace_include_locations: false,
            log_events: false,
            log_events_verbose: false,
            log_json: false,
            log_json_filter: crate::cli::DEFAULT_LOG_JSON_FILTER.to_string(),
            log_report: false,
            log_resources: false,
            min_peers: 1,
            repair: false,
            no_tui: false,
            command: None,
            rpc_max_request_body_bytes: 0,
            rpc_max_response_body_bytes: 0,
            rpc_max_connections: 0,
            rpc_max_batch_requests: 0,
            rpc_max_blocks_per_filter: 0,
            rpc_max_logs_per_response: 0,
            fast_sync_chunk_size: 16,
            fast_sync_chunk_max: None,
            fast_sync_max_inflight: 2,
            fast_sync_max_buffered_blocks: 64,
            db_write_batch_blocks: 1,
            db_write_flush_interval_ms: None,
            defer_compaction: false,
        }
    }

    fn header_with_number(number: u64) -> Header {
        let mut header = Header::default();
        header.number = number;
        header
    }

    fn bundle_with_number(number: u64) -> BlockBundle {
        BlockBundle {
            number,
            header: header_with_number(number),
            tx_hashes: StoredTxHashes { hashes: Vec::new() },
            transactions: StoredTransactions { txs: Vec::new() },
            size: StoredBlockSize { size: 0 },
            receipts: StoredReceipts {
                receipts: Vec::<Receipt>::new(),
            },
        }
    }

    #[test]
    fn wal_replay_triggers_compaction() {
        // With the new atomic compaction model, WAL writes don't update the bitset.
        // The bitset is rebuilt during compaction from actual segment data.
        // This test verifies that after reopen + compact, the block is visible.
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");
        let bundle = bundle_with_number(5);
        storage
            .write_block_bundles_wal(&[bundle])
            .expect("write wal");
        drop(storage);

        let reopened = Storage::open(&config).expect("reopen storage");
        // Before compaction, block is not visible (only in WAL)
        assert!(!reopened.has_block(5).expect("has block before compact"));

        // After compaction, block should be visible
        reopened.compact_shard(0).expect("compact shard");
        assert!(reopened.has_block(5).expect("has block after compact"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compaction_rewrites_sorted_segments() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");
        storage
            .write_block_bundles_wal(&[bundle_with_number(2), bundle_with_number(0)])
            .expect("write wal");
        storage.compact_shard(0).expect("compact shard");

        assert!(storage.block_header(0).expect("header 0").is_some());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert!(storage.block_header(2).expect("header 2").is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn repair_on_clean_storage_reports_ok() {
        let dir = temp_dir();
        let config = base_config(dir.clone());

        // Create a storage and write + compact some data
        let storage = Storage::open(&config).expect("open storage");
        storage
            .write_block_bundles_wal(&[bundle_with_number(0), bundle_with_number(1)])
            .expect("write wal");
        storage.compact_shard(0).expect("compact shard");
        drop(storage);

        // Run repair on clean storage
        let report = Storage::repair(&config).expect("repair");
        assert_eq!(report.shards.len(), 1);
        assert!(!report.shards[0].result.needs_recovery());
        assert_eq!(report.repaired_count(), 0);
        assert_eq!(report.clean_count(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn repair_empty_storage_returns_empty_report() {
        let dir = temp_dir();
        let config = base_config(dir.clone());

        // Run repair on non-existent storage
        let report = Storage::repair(&config).expect("repair");
        assert!(report.shards.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
