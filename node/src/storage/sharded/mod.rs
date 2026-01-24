//! Sharded storage backend (schema v2).

mod bitset;
mod hash;
mod nippy_raw;
mod wal;

use crate::cli::NodeConfig;
use crate::storage::{
    decode_bincode, decode_bincode_compat_value, decode_u64, encode_bincode_compat_value,
    encode_bincode_value, encode_u64_value, BlockBundle, LogIndexEntry, PeerCacheLoad,
    SegmentDiskStats, StorageConfigKey, StorageDiskStats, StoredBlockSize, StoredLogs, StoredPeer,
    StoredReceipts, StoredTransactions, StoredTxHashes, StoredWithdrawals, ZSTD_DICT_MAX_SIZE,
};
use bitset::Bitset;
use crc32fast::Hasher;
use eyre::{eyre, Result, WrapErr};
use hash::compute_shard_hash;
use nippy_raw::{NippyJarConfig, SegmentRawSource, SegmentRawWriter};
use reth_nippy_jar::{
    compression::{Compressors, Zstd},
    NippyJar, NippyJarCursor, NippyJarWriter, CONFIG_FILE_EXTENSION,
};
use reth_primitives_traits::{serde_bincode_compat::SerdeBincodeCompat, Header};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use wal::{append_records, build_index as build_wal_index, build_slice_index, WalRecord};
use zstd::bulk::Compressor;

const SCHEMA_VERSION: u64 = 2;
const META_FILE_NAME: &str = "meta.json";
const PEER_FILE_NAME: &str = "peers.json";
const STATIC_DIR_NAME: &str = "static";
const SHARDS_DIR_NAME: &str = "shards";
const SHARD_META_NAME: &str = "shard.json";
const BITSET_NAME: &str = "present.bitset";
const WAL_NAME: &str = "staging.wal";

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

fn segment_compressor(compression: SegmentCompression) -> Option<Compressors> {
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
    #[allow(dead_code)]
    compression: SegmentCompression,
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
            compression,
            state: Mutex::new(SegmentState { writer, next_block }),
        })
    }

    fn append_rows_inner(&self, start_block: u64, rows: &[Vec<u8>], commit: bool) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        let mut state = self.state.lock().expect("segment lock");
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
        if commit {
            state
                .writer
                .commit()
                .wrap_err("failed to commit static segment")?;
        }
        state.next_block = state.next_block.saturating_add(total_rows);
        Ok(())
    }

    fn append_rows(&self, start_block: u64, rows: &[Vec<u8>]) -> Result<()> {
        self.append_rows_inner(start_block, rows, true)
    }

    // Reserved for future optimizations where we want to amortize NippyJar commits across
    // multiple appends (e.g. compaction writing in larger batches).
    #[allow(dead_code)]
    fn append_rows_no_commit(&self, start_block: u64, rows: &[Vec<u8>]) -> Result<()> {
        self.append_rows_inner(start_block, rows, false)
    }

    // Reserved for future use alongside `append_rows_no_commit`.
    #[allow(dead_code)]
    fn commit(&self) -> Result<()> {
        let mut state = self.state.lock().expect("segment lock");
        state
            .writer
            .commit()
            .wrap_err("failed to commit static segment")?;
        Ok(())
    }

    fn prune_to(&self, ancestor_number: u64) -> Result<()> {
        let mut state = self.state.lock().expect("segment lock");
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
            .get(0)
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

pub struct Storage {
    data_dir: PathBuf,
    peer_cache_dir: PathBuf,
    meta: Mutex<MetaState>,
    shards: Mutex<HashMap<u64, Arc<Mutex<ShardState>>>>,
    peer_cache: Mutex<HashMap<String, StoredPeer>>,
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("data_dir", &self.data_dir)
            .finish()
    }
}

impl Storage {
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
            let shard_start = match name.parse::<u64>() {
                Ok(value) => value,
                Err(_) => continue,
            };
            let shard_dir = entry.path();
            let shard_meta = load_shard_meta(&shard_dir)?;
            let bitset = Bitset::load(&bitset_path(&shard_dir), meta.shard_size as usize)?;
            let mut state = ShardState {
                dir: shard_dir,
                meta: shard_meta,
                bitset,
            };
            repair_shard_from_wal(&mut state)?;
            reconcile_shard_meta(&mut state);
            let max_present = max_present_in_bitset(&state, meta.shard_size);
            if let Some(max_block) = max_present {
                meta.max_present_block = Some(meta.max_present_block.unwrap_or(0).max(max_block));
            }
            shards.insert(shard_start, Arc::new(Mutex::new(state)));
        }
        persist_meta(&meta_path, &meta)?;

        let peer_cache = load_peer_cache(&peer_cache_dir)?;
        Ok(Self {
            data_dir: config.data_dir.clone(),
            peer_cache_dir,
            meta: Mutex::new(meta),
            shards: Mutex::new(shards),
            peer_cache: Mutex::new(peer_cache),
        })
    }

    pub fn open_existing(config: &NodeConfig) -> Result<Option<Self>> {
        let meta = meta_path(&config.data_dir);
        if !meta.exists() {
            return Ok(None);
        }
        Ok(Some(Self::open(config)?))
    }

    pub fn disk_usage(&self) -> Result<StorageDiskStats> {
        Self::disk_usage_at(&self.data_dir)
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
        let meta = self.meta.lock().expect("meta lock");
        meta.shard_size
    }

    pub fn last_indexed_block(&self) -> Result<Option<u64>> {
        let meta = self.meta.lock().expect("meta lock");
        Ok(meta.max_present_block)
    }

    #[allow(dead_code)]
    pub fn set_last_indexed_block(&self, value: u64) -> Result<()> {
        let mut meta = self.meta.lock().expect("meta lock");
        meta.max_present_block = Some(value);
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

    pub fn head_seen(&self) -> Result<Option<u64>> {
        let meta = self.meta.lock().expect("meta lock");
        Ok(meta.head_seen)
    }

    pub fn set_head_seen(&self, value: u64) -> Result<()> {
        let mut meta = self.meta.lock().expect("meta lock");
        meta.head_seen = Some(value);
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

    pub fn has_block(&self, number: u64) -> Result<bool> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(false);
        };
        let state = shard.lock().expect("shard lock");
        let offset = (number - shard_start) as usize;
        Ok(state.bitset.is_set(offset))
    }

    pub fn missing_blocks_in_range(
        &self,
        range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<u64>> {
        let mut out = Vec::new();
        for block in range {
            if !self.has_block(block)? {
                out.push(block);
            }
        }
        Ok(out)
    }

    pub fn write_block_bundles_wal(&self, bundles: &[BlockBundle]) -> Result<Vec<u64>> {
        if bundles.is_empty() {
            return Ok(Vec::new());
        }
        let mut zstd = Compressor::new(0).wrap_err("failed to init zstd compressor")?;
        let mut per_shard: HashMap<u64, Vec<WalRecord>> = HashMap::new();
        for bundle in bundles {
            let shard_start = shard_start(bundle.number, self.shard_size());
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
        for (shard_start, mut records) in per_shard {
            let shard = self.get_or_create_shard(shard_start)?;
            let mut state = shard.lock().expect("shard lock");
            let mut to_append: Vec<WalRecord> = Vec::new();
            let mut seen_offsets: HashSet<usize> = HashSet::new();

            for record in records.drain(..) {
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

            let mut max_written: Option<u64> = None;
            let mut meta_completed = false;
            for record in &to_append {
                let offset = (record.block_number - shard_start) as usize;
                if state.bitset.set(offset) {
                    state.meta.present_count = state.meta.present_count.saturating_add(1);
                    state.meta.complete = state.meta.present_count as u64 >= state.meta.shard_size;
                    state.meta.sorted = false;
                    state.meta.sealed = false;
                    state.meta.content_hash = None;
                    if state.meta.complete {
                        meta_completed = true;
                    }
                    written_blocks.push(record.block_number);
                    max_written = Some(
                        max_written
                            .unwrap_or(record.block_number)
                            .max(record.block_number),
                    );
                }
            }

            if meta_completed {
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
        let mut state = shard.lock().expect("shard lock");
        let offset = (bundle.number - shard_start) as usize;
        if state.bitset.is_set(offset) {
            return Ok(());
        }
        let sorted_dir = sorted_dir(&state.dir);
        fs::create_dir_all(&sorted_dir).wrap_err("failed to create sorted dir")?;

        let segments = shard_segment_writers(&sorted_dir, shard_start)?;
        segments.headers.append_rows(
            bundle.number,
            &[encode_bincode_compat_value(&bundle.header)?],
        )?;
        segments
            .tx_hashes
            .append_rows(bundle.number, &[encode_bincode_value(&bundle.tx_hashes)?])?;
        segments.tx_meta.append_rows(
            bundle.number,
            &[encode_bincode_value(&bundle.transactions)?],
        )?;
        segments.receipts.append_rows(
            bundle.number,
            &[encode_bincode_compat_value(&bundle.receipts)?],
        )?;
        segments
            .sizes
            .append_rows(bundle.number, &[encode_u64_value(bundle.size.size)])?;

        if state.bitset.set(offset) {
            state.meta.present_count = state.meta.present_count.saturating_add(1);
            state.meta.complete = state.meta.present_count as u64 >= state.meta.shard_size;
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
        persist_shard_meta(&state.dir, &state.meta)?;
        state.bitset.flush(&bitset_path(&state.dir))?;
        self.bump_max_present(bundle.number)?;
        Ok(())
    }

    pub fn compact_shard(&self, shard_start: u64) -> Result<()> {
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(());
        };
        let mut state = shard.lock().expect("shard lock");
        let wal_path = wal_path(&state.dir);
        let wal_slices = build_slice_index(&wal_path, shard_start, state.meta.shard_size as usize)?;
        let has_wal = wal_slices.is_some();
        if !has_wal && state.meta.sorted {
            return Ok(());
        }

        let max_present = max_present_in_bitset(&state, state.meta.shard_size);
        let mut tail_block = state.meta.tail_block;
        if let Some(max_present) = max_present {
            tail_block = Some(tail_block.unwrap_or(max_present).max(max_present));
        }
        let Some(tail_block) = tail_block else {
            if wal_path.exists() {
                fs::remove_file(&wal_path).wrap_err("failed to remove staging.wal")?;
            }
            state.meta.sorted = true;
            persist_shard_meta(&state.dir, &state.meta)?;
            return Ok(());
        };

        let sorted_dir = sorted_dir(&state.dir);
        let temp_dir = state.dir.join("sorted.tmp");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).wrap_err("failed to clean sorted.tmp")?;
        }
        fs::create_dir_all(&temp_dir).wrap_err("failed to create sorted.tmp")?;

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

        let headers_cfg = NippyJarConfig {
            version: 1,
            user_header: SegmentHeader {
                start_block: shard_start,
            },
            columns: 1,
            rows: 0,
            compressor: segment_compressor(SegmentCompression::None),
            max_row_size: existing
                .as_ref()
                .map(|s| s.headers.config().max_row_size)
                .unwrap_or(0),
        };
        let tx_hashes_cfg = NippyJarConfig {
            version: 1,
            user_header: SegmentHeader {
                start_block: shard_start,
            },
            columns: 1,
            rows: 0,
            compressor: segment_compressor(SegmentCompression::None),
            max_row_size: existing
                .as_ref()
                .map(|s| s.tx_hashes.config().max_row_size)
                .unwrap_or(0),
        };
        let tx_meta_cfg = NippyJarConfig {
            version: 1,
            user_header: SegmentHeader {
                start_block: shard_start,
            },
            columns: 1,
            rows: 0,
            compressor: segment_compressor(SegmentCompression::Zstd {
                use_dict: false,
                max_dict_size: ZSTD_DICT_MAX_SIZE,
            }),
            max_row_size: existing
                .as_ref()
                .map(|s| s.tx_meta.config().max_row_size)
                .unwrap_or(0),
        };
        let receipts_cfg = NippyJarConfig {
            version: 1,
            user_header: SegmentHeader {
                start_block: shard_start,
            },
            columns: 1,
            rows: 0,
            compressor: segment_compressor(SegmentCompression::Zstd {
                use_dict: false,
                max_dict_size: ZSTD_DICT_MAX_SIZE,
            }),
            max_row_size: existing
                .as_ref()
                .map(|s| s.receipts.config().max_row_size)
                .unwrap_or(0),
        };
        let sizes_cfg = NippyJarConfig {
            version: 1,
            user_header: SegmentHeader {
                start_block: shard_start,
            },
            columns: 1,
            rows: 0,
            compressor: segment_compressor(SegmentCompression::None),
            max_row_size: existing
                .as_ref()
                .map(|s| s.sizes.config().max_row_size)
                .unwrap_or(0),
        };

        let mut headers_out = SegmentRawWriter::create(&temp_dir.join("headers"), headers_cfg)?;
        let mut tx_hashes_out =
            SegmentRawWriter::create(&temp_dir.join("tx_hashes"), tx_hashes_cfg)?;
        let mut tx_meta_out = SegmentRawWriter::create(&temp_dir.join("tx_meta"), tx_meta_cfg)?;
        let mut receipts_out = SegmentRawWriter::create(&temp_dir.join("receipts"), receipts_cfg)?;
        let mut sizes_out = SegmentRawWriter::create(&temp_dir.join("block_sizes"), sizes_cfg)?;

        let end_offset = (tail_block - shard_start) as usize;
        let total_rows = end_offset.saturating_add(1);
        for local_offset in 0..total_rows {
            if !state.bitset.is_set(local_offset) {
                headers_out.push_empty()?;
                tx_hashes_out.push_empty()?;
                tx_meta_out.push_empty()?;
                receipts_out.push_empty()?;
                sizes_out.push_empty()?;
                continue;
            }

            let block_number = shard_start + local_offset as u64;
            if let Some(wal) = wal_slices.as_ref() {
                if let Some(slices) = wal.entries[local_offset] {
                    let header_bytes = &wal.mmap[slices.header.as_range()];
                    let tx_hash_bytes = &wal.mmap[slices.tx_hashes.as_range()];
                    let tx_meta_bytes = &wal.mmap[slices.tx_meta.as_range()];
                    let receipt_bytes = &wal.mmap[slices.receipts.as_range()];
                    let size_bytes = &wal.mmap[slices.size.as_range()];

                    headers_out.push_bytes(header_bytes, slices.header.len())?;
                    tx_hashes_out.push_bytes(tx_hash_bytes, slices.tx_hashes.len())?;
                    tx_meta_out
                        .push_bytes(tx_meta_bytes, slices.tx_meta_uncompressed_len as usize)?;
                    receipts_out
                        .push_bytes(receipt_bytes, slices.receipts_uncompressed_len as usize)?;
                    sizes_out.push_bytes(size_bytes, slices.size.len())?;
                    continue;
                }
            }

            let Some(existing) = existing.as_ref() else {
                return Err(eyre!(
                    "missing block {} in WAL and sorted segment",
                    block_number
                ));
            };

            let header_bytes = existing
                .headers
                .row_bytes(local_offset)?
                .ok_or_else(|| eyre!("missing header for block {}", block_number))?;
            let tx_hash_bytes = existing
                .tx_hashes
                .row_bytes(local_offset)?
                .ok_or_else(|| eyre!("missing tx_hashes for block {}", block_number))?;
            let tx_meta_bytes = existing
                .tx_meta
                .row_bytes(local_offset)?
                .ok_or_else(|| eyre!("missing tx_meta for block {}", block_number))?;
            let receipt_bytes = existing
                .receipts
                .row_bytes(local_offset)?
                .ok_or_else(|| eyre!("missing receipts for block {}", block_number))?;
            let size_bytes = existing
                .sizes
                .row_bytes(local_offset)?
                .ok_or_else(|| eyre!("missing size for block {}", block_number))?;

            headers_out.push_bytes(header_bytes, header_bytes.len())?;
            tx_hashes_out.push_bytes(tx_hash_bytes, tx_hash_bytes.len())?;
            // For zstd segments, the bytes are already compressed. We rely on the carried over
            // `max_row_size` for existing rows, and only bump it based on WAL rows.
            tx_meta_out.push_bytes(tx_meta_bytes, 0)?;
            receipts_out.push_bytes(receipt_bytes, 0)?;
            sizes_out.push_bytes(size_bytes, size_bytes.len())?;
        }

        headers_out.finish()?;
        tx_hashes_out.finish()?;
        tx_meta_out.finish()?;
        receipts_out.finish()?;
        sizes_out.finish()?;

        if sorted_dir.exists() {
            fs::remove_dir_all(&sorted_dir).wrap_err("failed to remove old sorted")?;
        }
        fs::rename(&temp_dir, &sorted_dir).wrap_err("failed to swap sorted dir")?;
        if wal_path.exists() {
            fs::remove_file(&wal_path).wrap_err("failed to remove staging.wal")?;
        }

        state.meta.sorted = true;
        state.meta.tail_block = Some(tail_block);
        persist_shard_meta(&state.dir, &state.meta)?;
        if state.meta.complete && !state.meta.sealed {
            self.seal_shard_locked(&mut state)?;
        }
        state.bitset.flush(&bitset_path(&state.dir))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn compact_all_dirty(&self) -> Result<()> {
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock().expect("shards lock");
            shards.keys().copied().collect()
        };
        for shard_start in shard_starts {
            self.compact_shard(shard_start)?;
        }
        Ok(())
    }

    pub fn seal_completed_shards(&self) -> Result<()> {
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock().expect("shards lock");
            shards.keys().copied().collect()
        };
        for shard_start in shard_starts {
            let shard = self.get_shard(shard_start)?;
            let Some(shard) = shard else {
                continue;
            };
            let mut state = shard.lock().expect("shard lock");
            if state.meta.complete && state.meta.sorted && !state.meta.sealed {
                self.seal_shard_locked(&mut state)?;
                persist_shard_meta(&state.dir, &state.meta)?;
            }
        }
        Ok(())
    }

    pub fn rollback_to(&self, ancestor_number: u64) -> Result<()> {
        let shard_size = self.shard_size();
        let shard_starts: Vec<u64> = {
            let shards = self.shards.lock().expect("shards lock");
            shards.keys().copied().collect()
        };
        for shard_start in shard_starts {
            let shard_end = shard_start.saturating_add(shard_size).saturating_sub(1);
            if ancestor_number >= shard_end {
                continue;
            }
            let shard = self.get_or_create_shard(shard_start)?;
            let mut state = shard.lock().expect("shard lock");
            let sorted_dir = sorted_dir(&state.dir);
            if sorted_dir.exists() {
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
            state.meta.complete = state.meta.present_count as u64 >= state.meta.shard_size;
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

    pub fn block_header(&self, number: u64) -> Result<Option<Header>> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(None);
        };
        let state = shard.lock().expect("shard lock");
        let offset = (number - shard_start) as usize;
        if !state.bitset.is_set(offset) {
            return Ok(None);
        }
        let sorted_dir = sorted_dir(&state.dir);
        let segments = shard_segment_readers(&sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };
        segments.headers.read_row_compat(number)
    }

    pub fn block_tx_hashes(&self, number: u64) -> Result<Option<StoredTxHashes>> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(None);
        };
        let state = shard.lock().expect("shard lock");
        let offset = (number - shard_start) as usize;
        if !state.bitset.is_set(offset) {
            return Ok(None);
        }
        let sorted_dir = sorted_dir(&state.dir);
        let segments = shard_segment_readers(&sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };
        segments.tx_hashes.read_row(number)
    }

    #[allow(dead_code)]
    pub fn block_transactions(&self, number: u64) -> Result<Option<StoredTransactions>> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(None);
        };
        let state = shard.lock().expect("shard lock");
        let offset = (number - shard_start) as usize;
        if !state.bitset.is_set(offset) {
            return Ok(None);
        }
        let sorted_dir = sorted_dir(&state.dir);
        let segments = shard_segment_readers(&sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };
        segments.tx_meta.read_row(number)
    }

    #[allow(dead_code)]
    pub fn block_withdrawals(&self, _number: u64) -> Result<Option<StoredWithdrawals>> {
        Ok(None)
    }

    pub fn block_size(&self, number: u64) -> Result<Option<StoredBlockSize>> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(None);
        };
        let state = shard.lock().expect("shard lock");
        let offset = (number - shard_start) as usize;
        if !state.bitset.is_set(offset) {
            return Ok(None);
        }
        let sorted_dir = sorted_dir(&state.dir);
        let segments = shard_segment_readers(&sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };
        segments
            .sizes
            .read_row_u64(number)
            .map(|opt| opt.map(|size| StoredBlockSize { size }))
    }

    pub fn block_receipts(&self, number: u64) -> Result<Option<StoredReceipts>> {
        let shard_start = shard_start(number, self.shard_size());
        let shard = self.get_shard(shard_start)?;
        let Some(shard) = shard else {
            return Ok(None);
        };
        let state = shard.lock().expect("shard lock");
        let offset = (number - shard_start) as usize;
        if !state.bitset.is_set(offset) {
            return Ok(None);
        }
        let sorted_dir = sorted_dir(&state.dir);
        let segments = shard_segment_readers(&sorted_dir, shard_start)?;
        let Some(segments) = segments else {
            return Ok(None);
        };
        segments.receipts.read_row_compat(number)
    }

    #[allow(dead_code)]
    pub fn block_logs(&self, _number: u64) -> Result<Option<StoredLogs>> {
        Ok(None)
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

    #[allow(dead_code)]
    pub fn block_logs_range(
        &self,
        _range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<(u64, StoredLogs)>> {
        Ok(Vec::new())
    }

    #[allow(dead_code)]
    pub fn log_index_by_address_range(
        &self,
        _address: alloy_primitives::Address,
        _range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<LogIndexEntry>> {
        Ok(Vec::new())
    }

    #[allow(dead_code)]
    pub fn log_index_by_topic0_range(
        &self,
        _topic0: alloy_primitives::B256,
        _range: std::ops::RangeInclusive<u64>,
    ) -> Result<Vec<LogIndexEntry>> {
        Ok(Vec::new())
    }

    pub fn upsert_peer(&self, mut peer: StoredPeer) -> Result<()> {
        let mut cache = self.peer_cache.lock().expect("peer cache lock");
        if let Some(existing) = cache.get(&peer.peer_id) {
            if peer.aimd_batch_limit.is_none() {
                peer.aimd_batch_limit = existing.aimd_batch_limit;
            }
        }
        cache.insert(peer.peer_id.clone(), peer);
        Ok(())
    }

    pub fn flush_peer_cache(&self) -> Result<()> {
        let cache = self.peer_cache.lock().expect("peer cache lock");
        let peers: Vec<StoredPeer> = cache.values().cloned().collect();
        persist_peer_cache(&self.peer_cache_dir, &peers)
    }

    pub fn load_peers(&self, expire_before_ms: u64, max_peers: usize) -> Result<PeerCacheLoad> {
        let mut cache = self.peer_cache.lock().expect("peer cache lock");
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
        for peer in peers.iter() {
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
        let cache = self.peer_cache.lock().expect("peer cache lock");
        cache.values().cloned().collect()
    }

    pub fn update_peer_batch_limits(&self, limits: &[(String, u64)]) {
        let mut cache = self.peer_cache.lock().expect("peer cache lock");
        for (peer_id, limit) in limits {
            if let Some(peer) = cache.get_mut(peer_id) {
                peer.aimd_batch_limit = Some(*limit);
            }
        }
    }

    fn get_shard(&self, shard_start: u64) -> Result<Option<Arc<Mutex<ShardState>>>> {
        let shards = self.shards.lock().expect("shards lock");
        Ok(shards.get(&shard_start).cloned())
    }

    fn get_or_create_shard(&self, shard_start: u64) -> Result<Arc<Mutex<ShardState>>> {
        if let Some(shard) = self.get_shard(shard_start)? {
            return Ok(shard);
        }
        let mut shards = self.shards.lock().expect("shards lock");
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
        };
        persist_shard_meta(&dir, &meta)?;
        let state = ShardState { dir, meta, bitset };
        let shard = Arc::new(Mutex::new(state));
        shards.insert(shard_start, Arc::clone(&shard));
        Ok(shard)
    }

    fn bump_max_present(&self, block: u64) -> Result<()> {
        let mut meta = self.meta.lock().expect("meta lock");
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
        let shards = self.shards.lock().expect("shards lock");
        let mut max_present = None;
        for shard in shards.values() {
            let state = shard.lock().expect("shard lock");
            if let Some(max_block) = max_present_in_bitset(&state, shard_size) {
                max_present = Some(max_present.unwrap_or(0).max(max_block));
            }
        }
        let mut meta = self.meta.lock().expect("meta lock");
        meta.max_present_block = max_present;
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

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

struct ShardRawSegments {
    headers: SegmentRawSource<SegmentHeader>,
    tx_hashes: SegmentRawSource<SegmentHeader>,
    tx_meta: SegmentRawSource<SegmentHeader>,
    receipts: SegmentRawSource<SegmentHeader>,
    sizes: SegmentRawSource<SegmentHeader>,
}

struct ShardSegments {
    headers: SegmentWriter,
    tx_hashes: SegmentWriter,
    tx_meta: SegmentWriter,
    receipts: SegmentWriter,
    sizes: SegmentWriter,
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

fn shard_segment_readers(sorted_dir: &Path, shard_start: u64) -> Result<Option<ShardSegments>> {
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
    Ok(Some(shard_segment_writers(sorted_dir, shard_start)?))
}

fn shard_start(block_number: u64, shard_size: u64) -> u64 {
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

#[allow(dead_code)]
fn records_to_map(
    records: Vec<WalRecord>,
    shard_start: u64,
    shard_size: u64,
) -> Result<HashMap<usize, WalBundleRecord>> {
    let mut map = HashMap::new();
    for record in records {
        let wal_bundle: WalBundleRecord = decode_bincode(&record.payload)?;
        if wal_bundle.number < shard_start {
            continue;
        }
        let offset = (wal_bundle.number - shard_start) as usize;
        if offset >= shard_size as usize {
            continue;
        }
        map.insert(offset, wal_bundle);
    }
    Ok(map)
}

// Helper for optional WAL CRC validation (currently unused; retained for future verification
// tooling / debugging).
#[allow(dead_code)]
struct WalPayloadCrcReader<'a> {
    file: &'a mut fs::File,
    hasher: Hasher,
    remaining: u64,
}

#[allow(dead_code)]
impl<'a> WalPayloadCrcReader<'a> {
    fn new(file: &'a mut fs::File, block_number: u64, payload_len: u32) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&block_number.to_le_bytes());
        hasher.update(&payload_len.to_le_bytes());
        Self {
            file,
            hasher,
            remaining: payload_len as u64,
        }
    }

    fn finish(mut self) -> Result<Hasher> {
        let mut buf = [0u8; 64 * 1024];
        while self.remaining > 0 {
            let to_read = (self.remaining as usize).min(buf.len());
            self.file.read_exact(&mut buf[..to_read])?;
            self.hasher.update(&buf[..to_read]);
            self.remaining = self.remaining.saturating_sub(to_read as u64);
        }
        Ok(self.hasher)
    }
}

#[allow(dead_code)]
impl<'a> Read for WalPayloadCrcReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Ok(0);
        }
        let to_read = buf.len().min(self.remaining as usize);
        let n = self.file.read(&mut buf[..to_read])?;
        self.hasher.update(&buf[..n]);
        self.remaining = self.remaining.saturating_sub(n as u64);
        Ok(n)
    }
}

#[allow(dead_code)]
fn read_wal_bundle_record_at(
    file: &mut fs::File,
    record_offset: u64,
    expected_payload_len: u32,
) -> Result<WalBundleRecord> {
    file.seek(SeekFrom::Start(record_offset))?;
    let mut num_buf = [0u8; 8];
    file.read_exact(&mut num_buf)?;
    let block_number = u64::from_le_bytes(num_buf);

    let mut len_buf = [0u8; 4];
    file.read_exact(&mut len_buf)?;
    let payload_len = u32::from_le_bytes(len_buf);
    if payload_len != expected_payload_len {
        return Err(eyre!(
            "wal payload length mismatch at offset {} (block {}): expected {}, got {}",
            record_offset,
            block_number,
            expected_payload_len,
            payload_len
        ));
    }

    let mut reader = WalPayloadCrcReader::new(file, block_number, payload_len);
    let bundle: WalBundleRecord =
        bincode::deserialize_from(&mut reader).wrap_err("failed to decode WAL bundle record")?;
    let hasher = reader.finish()?;

    let mut crc_buf = [0u8; 4];
    file.read_exact(&mut crc_buf)?;
    let crc_expected = u32::from_le_bytes(crc_buf);
    let crc_actual = hasher.finalize();
    if crc_actual != crc_expected {
        return Err(eyre!(
            "wal crc mismatch at offset {} (block {}): expected {}, got {}",
            record_offset,
            block_number,
            crc_expected,
            crc_actual
        ));
    }

    Ok(bundle)
}

fn repair_shard_from_wal(state: &mut ShardState) -> Result<()> {
    let wal_path = wal_path(&state.dir);
    if !wal_path.exists() {
        return Ok(());
    }
    // Avoid loading the entire WAL into memory on startup; we only need block numbers.
    let entries = build_wal_index(&wal_path)?;
    for entry in entries {
        let block = entry.block_number;
        if block < state.meta.shard_start {
            continue;
        }
        let offset = (block - state.meta.shard_start) as usize;
        if state.bitset.set(offset) {
            state.meta.present_count = state.meta.present_count.saturating_add(1);
        }
    }
    state.meta.sorted = false;
    state.meta.sealed = false;
    state.meta.content_hash = None;
    Ok(())
}

fn reconcile_shard_meta(state: &mut ShardState) {
    let count = state.bitset.count_ones();
    state.meta.present_count = count;
    state.meta.complete = count as u64 >= state.meta.shard_size;
}

fn default_hash_algo() -> String {
    "sha256".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{BenchmarkMode, HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
    use alloy_primitives::{Address, B256, U256};
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
            benchmark: BenchmarkMode::Disabled,
            benchmark_name: None,
            benchmark_output_dir: PathBuf::from(crate::cli::DEFAULT_BENCHMARK_OUTPUT_DIR),
            benchmark_trace: false,
            benchmark_trace_filter: crate::cli::DEFAULT_BENCHMARK_TRACE_FILTER.to_string(),
            benchmark_trace_include_args: false,
            benchmark_trace_include_locations: false,
            benchmark_events: false,
            benchmark_min_peers: None,
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
            fast_sync_batch_timeout_ms: crate::cli::DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS,
            fast_sync_max_buffered_blocks: 64,
            fast_sync_max_lookahead_blocks: crate::cli::DEFAULT_FAST_SYNC_MAX_LOOKAHEAD_BLOCKS,
            db_write_batch_blocks: 1,
            db_write_flush_interval_ms: None,
        }
    }

    fn header_with_number(number: u64) -> Header {
        let mut header = Header::default();
        header.number = number;
        header
    }

    fn bundle_with_number(number: u64) -> BlockBundle {
        let hash = B256::from([number as u8; 32]);
        BlockBundle {
            number,
            header: header_with_number(number),
            tx_hashes: StoredTxHashes { hashes: Vec::new() },
            transactions: StoredTransactions {
                txs: vec![crate::storage::StoredTransaction {
                    hash,
                    from: Some(Address::from([0x0au8; 20])),
                    to: None,
                    value: U256::from(number),
                    nonce: number,
                    signature: None,
                    signing_hash: None,
                }],
            },
            withdrawals: StoredWithdrawals { withdrawals: None },
            size: StoredBlockSize { size: 0 },
            receipts: StoredReceipts {
                receipts: Vec::<Receipt>::new(),
            },
            logs: StoredLogs { logs: Vec::new() },
        }
    }

    #[test]
    fn wal_replay_sets_bitset() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");
        let bundle = bundle_with_number(5);
        storage
            .write_block_bundles_wal(&[bundle])
            .expect("write wal");
        drop(storage);

        let reopened = Storage::open(&config).expect("reopen storage");
        assert!(reopened.has_block(5).expect("has block"));

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
}
