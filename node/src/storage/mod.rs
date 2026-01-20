//! Storage backed by static files (NippyJar) + tiny JSON metadata.

use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
use alloy_primitives::{Address, B256, Bytes, Signature, U256};
use eyre::{eyre, Result, WrapErr};
use reth_ethereum_primitives::Receipt;
use reth_nippy_jar::{NippyJar, NippyJarCursor, NippyJarWriter, CONFIG_FILE_EXTENSION};
use reth_primitives_traits::{
    serde_bincode_compat::{BincodeReprFor, SerdeBincodeCompat},
    Header,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::HashMap,
    fs,
    io,
    net::SocketAddr,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::Mutex,
};

const SCHEMA_VERSION: u64 = 1;
const META_FILE_NAME: &str = "meta.json";
const PEER_FILE_NAME: &str = "peers.json";
const STATIC_DIR_NAME: &str = "static";
const ZSTD_DICT_MAX_SIZE: usize = 5000;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTxHashes {
    pub hashes: Vec<B256>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTransaction {
    pub hash: B256,
    #[serde(default)]
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub value: U256,
    pub nonce: u64,
    #[serde(default)]
    pub signature: Option<Signature>,
    #[serde(default)]
    pub signing_hash: Option<B256>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTransactions {
    pub txs: Vec<StoredTransaction>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredWithdrawal {
    pub index: u64,
    pub validator_index: u64,
    pub address: Address,
    pub amount: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredWithdrawals {
    pub withdrawals: Option<Vec<StoredWithdrawal>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredBlockSize {
    pub size: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockBundle {
    pub number: u64,
    pub header: Header,
    pub tx_hashes: StoredTxHashes,
    pub transactions: StoredTransactions,
    pub withdrawals: StoredWithdrawals,
    pub size: StoredBlockSize,
    pub receipts: StoredReceipts,
    pub logs: StoredLogs,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptBundle {
    pub number: u64,
    pub header: Header,
    pub receipts: StoredReceipts,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredReceipts {
    pub receipts: Vec<Receipt>,
}

type ReceiptBincodeRepr<'a> = BincodeReprFor<'a, Receipt>;

impl SerdeBincodeCompat for StoredReceipts {
    type BincodeRepr<'a> = Vec<ReceiptBincodeRepr<'a>>;

    fn as_repr(&self) -> Self::BincodeRepr<'_> {
        self.receipts.iter().map(|receipt| receipt.as_repr()).collect()
    }

    fn from_repr(repr: Self::BincodeRepr<'_>) -> Self {
        let receipts = repr
            .into_iter()
            .map(|repr| <Receipt as SerdeBincodeCompat>::from_repr(repr))
            .collect();
        StoredReceipts { receipts }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredLog {
    pub address: Address,
    pub topics: Vec<B256>,
    pub block_number: u64,
    pub block_hash: B256,
    pub transaction_hash: B256,
    pub transaction_index: u64,
    pub log_index: u64,
    pub removed: bool,
    pub data: Bytes,
}

#[derive(Debug, Clone, Serialize)]
pub struct SegmentDiskStats {
    pub name: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageDiskStats {
    pub total_bytes: u64,
    pub meta_bytes: u64,
    pub peers_bytes: u64,
    pub static_total_bytes: u64,
    pub segments: Vec<SegmentDiskStats>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredPeer {
    pub peer_id: String,
    pub tcp_addr: SocketAddr,
    pub udp_addr: Option<SocketAddr>,
    pub last_seen_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerCacheLoad {
    pub peers: Vec<StoredPeer>,
    pub total: usize,
    pub expired: usize,
    pub corrupted: usize,
    pub capped: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredLogs {
    pub logs: Vec<StoredLog>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogIndexEntry {
    pub block_number: u64,
    pub log_index: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
struct StorageConfigKey {
    retention_mode: RetentionMode,
    head_source: HeadSource,
    reorg_strategy: ReorgStrategy,
}

impl From<&NodeConfig> for StorageConfigKey {
    fn from(config: &NodeConfig) -> Self {
        Self {
            retention_mode: config.retention_mode,
            head_source: config.head_source,
            reorg_strategy: config.reorg_strategy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaState {
    schema_version: u64,
    chain_id: u64,
    storage_key: StorageConfigKey,
    start_block: u64,
    last_indexed_block: Option<u64>,
    head_seen: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SegmentHeader {
    start_block: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentCompression {
    None,
    Zstd { use_dict: bool, max_dict_size: usize },
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
            fs::create_dir_all(parent).wrap_err("failed to create static dir")?;
        }
        let jar = if segment_exists(&path) {
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
            let expected_compress = matches!(compression, SegmentCompression::Zstd { .. });
            if jar.compressor().is_some() != expected_compress {
                return Err(eyre!(
                    "segment compression mismatch for {}",
                    path.display()
                ));
            }
            jar
        } else {
            let mut jar = NippyJar::new(1, &path, SegmentHeader { start_block });
            match compression {
                SegmentCompression::None => {}
                SegmentCompression::Zstd { use_dict, max_dict_size } => {
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

    fn append_rows(&self, start_block: u64, rows: &[Vec<u8>]) -> Result<()> {
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
        let all_rows = if gap == 0 {
            rows.to_vec()
        } else {
            let mut filled = vec![Vec::new(); gap];
            filled.extend_from_slice(rows);
            filled
        };
        let iter = all_rows.iter().map(|bytes| Ok(bytes.as_slice()));
        state
            .writer
            .append_rows(vec![iter], all_rows.len() as u64)
            .wrap_err("failed to append rows")?;
        state
            .writer
            .commit()
            .wrap_err("failed to commit static segment")?;
        state.next_block = state
            .next_block
            .saturating_add(all_rows.len() as u64);
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
                state.writer.commit().wrap_err("failed to commit segment prune")?;
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

    fn read_row<T: DeserializeOwned>(&self, block_number: u64) -> Result<Option<T>> {
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
        let decoded = decode_bincode(bytes)?;
        Ok(Some(decoded))
    }

    fn read_row_compat<T>(&self, block_number: u64) -> Result<Option<T>>
    where
        T: SerdeBincodeCompat,
        <T as SerdeBincodeCompat>::BincodeRepr<'static>: DeserializeOwned,
    {
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
        let decoded = decode_bincode_compat_value(bytes)?;
        Ok(Some(decoded))
    }

    fn read_row_u64(&self, block_number: u64) -> Result<Option<u64>> {
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
        decode_u64(bytes).map(Some)
    }

    fn read_range<T: DeserializeOwned>(
        &self,
        range: RangeInclusive<u64>,
    ) -> Result<Vec<(u64, T)>> {
        let jar = NippyJar::<SegmentHeader>::load(&self.path)
            .wrap_err("failed to load static segment")?;
        let header = jar.user_header();
        let mut cursor = NippyJarCursor::new(&jar).wrap_err("failed to open segment cursor")?;
        let mut out = Vec::new();
        for block in range {
            if block < header.start_block {
                continue;
            }
            let row = block.saturating_sub(header.start_block) as usize;
            if row >= jar.rows() {
                break;
            }
            let Some(row_vals) = cursor.row_by_number(row)? else {
                continue;
            };
            let bytes = row_vals
                .get(0)
                .ok_or_else(|| eyre!("missing column data"))?;
            if bytes.is_empty() {
                continue;
            }
            let decoded = decode_bincode(bytes)?;
            out.push((block, decoded));
        }
        Ok(out)
    }

    fn read_range_compat<T>(&self, range: RangeInclusive<u64>) -> Result<Vec<(u64, T)>>
    where
        T: SerdeBincodeCompat,
        <T as SerdeBincodeCompat>::BincodeRepr<'static>: DeserializeOwned,
    {
        let jar = NippyJar::<SegmentHeader>::load(&self.path)
            .wrap_err("failed to load static segment")?;
        let header = jar.user_header();
        let mut cursor = NippyJarCursor::new(&jar).wrap_err("failed to open segment cursor")?;
        let mut out = Vec::new();
        for block in range {
            if block < header.start_block {
                continue;
            }
            let row = block.saturating_sub(header.start_block) as usize;
            if row >= jar.rows() {
                break;
            }
            let Some(row_vals) = cursor.row_by_number(row)? else {
                continue;
            };
            let bytes = row_vals
                .get(0)
                .ok_or_else(|| eyre!("missing column data"))?;
            if bytes.is_empty() {
                continue;
            }
            let decoded = decode_bincode_compat_value(bytes)?;
            out.push((block, decoded));
        }
        Ok(out)
    }
}

pub struct Storage {
    data_dir: PathBuf,
    meta: Mutex<MetaState>,
    segments: SegmentStore,
    peer_cache: Mutex<HashMap<String, StoredPeer>>,
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage")
            .field("data_dir", &self.data_dir)
            .finish()
    }
}

struct SegmentStore {
    headers: SegmentWriter,
    tx_hashes: SegmentWriter,
    tx_meta: SegmentWriter,
    receipts: SegmentWriter,
    sizes: SegmentWriter,
}

impl Storage {
    pub fn open(config: &NodeConfig) -> Result<Self> {
        fs::create_dir_all(&config.data_dir).wrap_err("failed to create data dir")?;
        let meta_path = meta_path(&config.data_dir);
        let meta = if meta_path.exists() {
            load_meta(&meta_path)?
        } else {
            MetaState {
                schema_version: SCHEMA_VERSION,
                chain_id: config.chain_id,
                storage_key: StorageConfigKey::from(config),
                start_block: config.start_block,
                last_indexed_block: None,
                head_seen: None,
            }
        };
        validate_meta(config, &meta)?;
        if meta.schema_version != SCHEMA_VERSION {
            return Err(eyre!(
                "schema version mismatch: expected {}, got {}",
                SCHEMA_VERSION,
                meta.schema_version
            ));
        }
        if !meta_path.exists() {
            persist_meta(&meta_path, &meta)?;
        }

        let static_dir = static_dir(&config.data_dir);
        fs::create_dir_all(&static_dir).wrap_err("failed to create static dir")?;

        let start_block = meta.start_block;
        let segments = SegmentStore {
            headers: SegmentWriter::open(static_dir.join("headers"), start_block, SegmentCompression::None)?,
            tx_hashes: SegmentWriter::open(static_dir.join("tx_hashes"), start_block, SegmentCompression::None)?,
            tx_meta: SegmentWriter::open(
                static_dir.join("tx_meta"),
                start_block,
                SegmentCompression::Zstd { use_dict: false, max_dict_size: ZSTD_DICT_MAX_SIZE },
            )?,
            receipts: SegmentWriter::open(
                static_dir.join("receipts"),
                start_block,
                SegmentCompression::Zstd { use_dict: false, max_dict_size: ZSTD_DICT_MAX_SIZE },
            )?,
            sizes: SegmentWriter::open(static_dir.join("block_sizes"), start_block, SegmentCompression::None)?,
        };

        let peer_cache = load_peer_cache(&config.data_dir)?;
        Ok(Self {
            data_dir: config.data_dir.clone(),
            meta: Mutex::new(meta),
            segments,
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
        let segments = segment_disk_stats(&static_dir)?;

        Ok(StorageDiskStats {
            total_bytes,
            meta_bytes,
            peers_bytes,
            static_total_bytes,
            segments,
        })
    }

    pub fn last_indexed_block(&self) -> Result<Option<u64>> {
        let meta = self.meta.lock().expect("meta lock");
        Ok(meta.last_indexed_block)
    }

    pub fn set_last_indexed_block(&self, value: u64) -> Result<()> {
        let mut meta = self.meta.lock().expect("meta lock");
        meta.last_indexed_block = Some(value);
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

    #[allow(dead_code)]
    pub fn write_block_header(&self, number: u64, header: Header) -> Result<()> {
        self.segments
            .headers
            .append_rows(number, &[encode_bincode_compat_value(&header)?])
    }

    #[allow(dead_code)]
    pub fn write_block_tx_hashes(&self, number: u64, hashes: StoredTxHashes) -> Result<()> {
        self.segments
            .tx_hashes
            .append_rows(number, &[encode_bincode_value(&hashes)?])
    }

    #[allow(dead_code)]
    pub fn write_block_transactions(&self, number: u64, transactions: StoredTransactions) -> Result<()> {
        self.segments
            .tx_meta
            .append_rows(number, &[encode_bincode_value(&transactions)?])
    }

    #[allow(dead_code)]
    pub fn write_block_withdrawals(&self, _number: u64, _withdrawals: StoredWithdrawals) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    pub fn write_block_size(&self, number: u64, size: StoredBlockSize) -> Result<()> {
        self.segments
            .sizes
            .append_rows(number, &[encode_u64_value(size.size)])
    }

    #[allow(dead_code)]
    pub fn write_block_receipts(&self, number: u64, receipts: StoredReceipts) -> Result<()> {
        self.segments
            .receipts
            .append_rows(number, &[encode_bincode_compat_value(&receipts)?])
    }

    #[allow(dead_code)]
    pub fn write_block_logs(&self, _number: u64, _logs: StoredLogs) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    pub fn write_log_indexes(&self, _logs: &[StoredLog]) -> Result<()> {
        Ok(())
    }

    pub fn write_block_bundle_batch(&self, bundles: &[BlockBundle]) -> Result<()> {
        if bundles.is_empty() {
            return Ok(());
        }
        for window in bundles.windows(2) {
            if window[1].number != window[0].number.saturating_add(1) {
                return Err(eyre!("bundle batch is not contiguous"));
            }
        }
        let first = bundles.first().expect("bundles non-empty").number;
        let mut headers = Vec::with_capacity(bundles.len());
        let mut tx_hashes = Vec::with_capacity(bundles.len());
        let mut tx_meta = Vec::with_capacity(bundles.len());
        let mut receipts = Vec::with_capacity(bundles.len());
        let mut sizes = Vec::with_capacity(bundles.len());

        for bundle in bundles {
            headers.push(encode_bincode_compat_value(&bundle.header)?);
            tx_hashes.push(encode_bincode_value(&bundle.tx_hashes)?);
            tx_meta.push(encode_bincode_value(&bundle.transactions)?);
            receipts.push(encode_bincode_compat_value(&bundle.receipts)?);
            sizes.push(encode_u64_value(bundle.size.size));
        }

        self.segments.headers.append_rows(first, &headers)?;
        self.segments.tx_hashes.append_rows(first, &tx_hashes)?;
        self.segments.tx_meta.append_rows(first, &tx_meta)?;
        self.segments.receipts.append_rows(first, &receipts)?;
        self.segments.sizes.append_rows(first, &sizes)?;

        let last_number = bundles.last().map(|bundle| bundle.number).unwrap_or(first);
        self.set_last_indexed_block(last_number)?;
        Ok(())
    }

    pub fn write_receipts_batch(&self, bundles: &[ReceiptBundle]) -> Result<()> {
        if bundles.is_empty() {
            return Ok(());
        }
        for window in bundles.windows(2) {
            if window[1].number != window[0].number.saturating_add(1) {
                return Err(eyre!("receipt batch is not contiguous"));
            }
        }
        let first = bundles.first().expect("bundles non-empty").number;
        let mut headers = Vec::with_capacity(bundles.len());
        let mut receipts = Vec::with_capacity(bundles.len());
        for bundle in bundles {
            headers.push(encode_bincode_compat_value(&bundle.header)?);
            receipts.push(encode_bincode_compat_value(&bundle.receipts)?);
        }
        self.segments.headers.append_rows(first, &headers)?;
        self.segments.receipts.append_rows(first, &receipts)?;

        let last_number = bundles.last().map(|bundle| bundle.number).unwrap_or(first);
        self.set_last_indexed_block(last_number)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn rollback_to(&self, ancestor_number: u64) -> Result<()> {
        self.segments.headers.prune_to(ancestor_number)?;
        self.segments.tx_hashes.prune_to(ancestor_number)?;
        self.segments.tx_meta.prune_to(ancestor_number)?;
        self.segments.receipts.prune_to(ancestor_number)?;
        self.segments.sizes.prune_to(ancestor_number)?;
        let mut meta = self.meta.lock().expect("meta lock");
        meta.last_indexed_block = if ancestor_number < meta.start_block {
            None
        } else {
            Some(ancestor_number)
        };
        persist_meta(&meta_path(&self.data_dir), &meta)
    }

    #[allow(dead_code)]
    pub fn block_header(&self, number: u64) -> Result<Option<Header>> {
        self.segments.headers.read_row_compat(number)
    }

    #[allow(dead_code)]
    pub fn block_tx_hashes(&self, number: u64) -> Result<Option<StoredTxHashes>> {
        self.segments.tx_hashes.read_row(number)
    }

    #[allow(dead_code)]
    pub fn block_transactions(&self, number: u64) -> Result<Option<StoredTransactions>> {
        self.segments.tx_meta.read_row(number)
    }

    #[allow(dead_code)]
    pub fn block_withdrawals(&self, _number: u64) -> Result<Option<StoredWithdrawals>> {
        Ok(None)
    }

    #[allow(dead_code)]
    pub fn block_size(&self, number: u64) -> Result<Option<StoredBlockSize>> {
        self.segments
            .sizes
            .read_row_u64(number)
            .map(|opt| opt.map(|size| StoredBlockSize { size }))
    }

    #[allow(dead_code)]
    pub fn block_receipts(&self, number: u64) -> Result<Option<StoredReceipts>> {
        self.segments.receipts.read_row_compat(number)
    }

    #[allow(dead_code)]
    pub fn block_logs(&self, _number: u64) -> Result<Option<StoredLogs>> {
        Ok(None)
    }

    #[allow(dead_code)]
    pub fn block_headers_range(&self, range: RangeInclusive<u64>) -> Result<Vec<(u64, Header)>> {
        self.segments.headers.read_range_compat(range)
    }

    #[allow(dead_code)]
    pub fn block_tx_hashes_range(
        &self,
        range: RangeInclusive<u64>,
    ) -> Result<Vec<(u64, StoredTxHashes)>> {
        self.segments.tx_hashes.read_range(range)
    }

    #[allow(dead_code)]
    pub fn block_receipts_range(
        &self,
        range: RangeInclusive<u64>,
    ) -> Result<Vec<(u64, StoredReceipts)>> {
        self.segments.receipts.read_range_compat(range)
    }

    #[allow(dead_code)]
    pub fn block_logs_range(&self, _range: RangeInclusive<u64>) -> Result<Vec<(u64, StoredLogs)>> {
        Ok(Vec::new())
    }

    #[allow(dead_code)]
    pub fn log_index_by_address_range(
        &self,
        _address: Address,
        _range: RangeInclusive<u64>,
    ) -> Result<Vec<LogIndexEntry>> {
        Ok(Vec::new())
    }

    #[allow(dead_code)]
    pub fn log_index_by_topic0_range(
        &self,
        _topic0: B256,
        _range: RangeInclusive<u64>,
    ) -> Result<Vec<LogIndexEntry>> {
        Ok(Vec::new())
    }

    pub fn upsert_peer(&self, peer: StoredPeer) -> Result<()> {
        let mut cache = self.peer_cache.lock().expect("peer cache lock");
        cache.insert(peer.peer_id.clone(), peer);
        Ok(())
    }

    pub fn flush_peer_cache(&self) -> Result<()> {
        let cache = self.peer_cache.lock().expect("peer cache lock");
        let peers: Vec<StoredPeer> = cache.values().cloned().collect();
        persist_peer_cache(&self.data_dir, &peers)
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
        persist_peer_cache(&self.data_dir, &peers)?;
        Ok(PeerCacheLoad {
            peers,
            total,
            expired,
            corrupted,
            capped,
        })
    }
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

fn peer_path(data_dir: &Path) -> PathBuf {
    data_dir.join(PEER_FILE_NAME)
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

fn segment_disk_stats(static_dir: &Path) -> Result<Vec<SegmentDiskStats>> {
    let segments = ["headers", "tx_hashes", "tx_meta", "receipts", "block_sizes"];
    let mut out = Vec::with_capacity(segments.len());
    for name in segments {
        let base = static_dir.join(name);
        let bytes = segment_disk_size(&base)?;
        out.push(SegmentDiskStats {
            name: name.to_string(),
            bytes,
        });
    }
    Ok(out)
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
    if meta.start_block != config.start_block {
        return Err(eyre!(
            "start block mismatch: expected {}, got {}",
            config.start_block,
            meta.start_block
        ));
    }
    Ok(())
}

fn load_meta(path: &Path) -> Result<MetaState> {
    let bytes = fs::read(path).wrap_err("failed to read meta.json")?;
    serde_json::from_slice(&bytes).wrap_err("failed to decode meta.json")
}

fn persist_meta(path: &Path, meta: &MetaState) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(meta).wrap_err("failed to encode meta.json")?;
    fs::write(path, bytes).wrap_err("failed to write meta.json")
}

fn load_peer_cache(data_dir: &Path) -> Result<HashMap<String, StoredPeer>> {
    let path = peer_path(data_dir);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let bytes = fs::read(&path).wrap_err("failed to read peers.json")?;
    let peers: Vec<StoredPeer> =
        serde_json::from_slice(&bytes).wrap_err("failed to decode peers.json")?;
    Ok(peers.into_iter().map(|peer| (peer.peer_id.clone(), peer)).collect())
}

fn persist_peer_cache(data_dir: &Path, peers: &[StoredPeer]) -> Result<()> {
    let path = peer_path(data_dir);
    let bytes = serde_json::to_vec_pretty(peers).wrap_err("failed to encode peers.json")?;
    fs::write(&path, bytes).wrap_err("failed to write peers.json")
}

fn encode_bincode_value<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).wrap_err("failed to encode bincode payload")
}

fn decode_bincode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes).wrap_err("failed to decode bincode payload")
}

fn encode_bincode_compat_value<T: SerdeBincodeCompat>(value: &T) -> Result<Vec<u8>> {
    let repr = value.as_repr();
    bincode::serialize(&repr).wrap_err("failed to encode bincode payload")
}

fn decode_bincode_compat_value<T>(bytes: &[u8]) -> Result<T>
where
    T: SerdeBincodeCompat,
    <T as SerdeBincodeCompat>::BincodeRepr<'static>: DeserializeOwned,
{
    let repr: <T as SerdeBincodeCompat>::BincodeRepr<'static> =
        bincode::deserialize(bytes).wrap_err("failed to decode bincode payload")?;
    Ok(<T as SerdeBincodeCompat>::from_repr(repr))
}

fn encode_u64_value(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

fn decode_u64(bytes: &[u8]) -> Result<u64> {
    if bytes.len() != 8 {
        return Err(eyre!("invalid u64 encoding"));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::BenchmarkMode;
    use alloy_primitives::{Address, B256, Bytes, Log, Signature, U256};
    use reth_ethereum_primitives::TxType;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "stateless-history-node-test-{now}-{}-{suffix}",
            std::process::id()
        ));
        path
    }

    fn base_config(data_dir: PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir,
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            end_block: None,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
            verbosity: 0,
            benchmark: BenchmarkMode::Disabled,
            command: None,
            rpc_max_request_body_bytes: 0,
            rpc_max_response_body_bytes: 0,
            rpc_max_connections: 0,
            rpc_max_batch_requests: 0,
            rpc_max_blocks_per_filter: 0,
            rpc_max_logs_per_response: 0,
            fast_sync_chunk_size: 16,
            fast_sync_max_inflight: 2,
            fast_sync_max_buffered_blocks: 64,
            db_write_batch_blocks: 1,
            db_write_flush_interval_ms: None,
        }
    }

    fn receipt_with_logs(logs: Vec<Log>) -> Receipt {
        Receipt {
            tx_type: TxType::Legacy,
            success: true,
            cumulative_gas_used: 0,
            logs,
        }
    }

    fn header_with_number(number: u64) -> Header {
        let mut header = Header::default();
        header.number = number;
        header
    }

    #[test]
    fn storage_bootstrap_and_config_validation() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");
        assert_eq!(storage.last_indexed_block().unwrap(), None);
        assert_eq!(storage.head_seen().unwrap(), None);
        storage.set_last_indexed_block(10).expect("set last indexed");
        storage.set_head_seen(12).expect("set head seen");
        assert_eq!(storage.last_indexed_block().unwrap(), Some(10));
        assert_eq!(storage.head_seen().unwrap(), Some(12));

        let mut mismatched = config.clone();
        mismatched.chain_id = 2;
        let err = Storage::open(&mismatched).expect_err("chain mismatch");
        assert!(format!("{err}").contains("chain id mismatch"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_range_reads_roundtrip() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let header0 = header_with_number(0);
        let header1 = header_with_number(1);
        let receipts0 = vec![receipt_with_logs(vec![Log::new_unchecked(
            Address::from([0x01u8; 20]),
            vec![B256::from([0x10u8; 32])],
            Bytes::from(vec![0x01]),
        )])];
        let receipts1 = vec![receipt_with_logs(vec![Log::new_unchecked(
            Address::from([0x02u8; 20]),
            vec![B256::from([0x20u8; 32])],
            Bytes::from(vec![0x02]),
        )])];

        storage
            .write_block_header(0, header0.clone())
            .expect("write header 0");
        storage
            .write_block_header(1, header1.clone())
            .expect("write header 1");
        storage
            .write_block_tx_hashes(0, StoredTxHashes { hashes: vec![B256::ZERO] })
            .expect("write tx hashes 0");
        storage
            .write_block_tx_hashes(1, StoredTxHashes { hashes: vec![B256::ZERO] })
            .expect("write tx hashes 1");
        storage
            .write_block_receipts(0, StoredReceipts { receipts: receipts0.clone() })
            .expect("write receipts 0");
        storage
            .write_block_receipts(1, StoredReceipts { receipts: receipts1.clone() })
            .expect("write receipts 1");

        let headers = storage.block_headers_range(0..=1).expect("headers range");
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, 0);
        assert_eq!(headers[1].0, 1);

        let hashes = storage
            .block_tx_hashes_range(0..=1)
            .expect("hashes range");
        assert_eq!(hashes.len(), 2);

        let receipts = storage
            .block_receipts_range(0..=1)
            .expect("receipts range");
        assert_eq!(receipts.len(), 2);

        let logs = storage.block_logs_range(0..=1).expect("logs range");
        assert!(logs.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_block_metadata_roundtrip() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let sig0 = Signature::new(U256::from(1), U256::from(2), false);
        let sig1 = Signature::new(U256::from(3), U256::from(4), true);
        let sig_hash0 = B256::from([0x44u8; 32]);
        let sig_hash1 = B256::from([0x55u8; 32]);
        let tx0 = StoredTransaction {
            hash: B256::from([0x11u8; 32]),
            from: Some(Address::from([0x01u8; 20])),
            to: Some(Address::from([0x02u8; 20])),
            value: U256::from(42),
            nonce: 7,
            signature: Some(sig0),
            signing_hash: Some(sig_hash0),
        };
        let tx1 = StoredTransaction {
            hash: B256::from([0x22u8; 32]),
            from: Some(Address::from([0x03u8; 20])),
            to: None,
            value: U256::from(7),
            nonce: 9,
            signature: Some(sig1),
            signing_hash: Some(sig_hash1),
        };
        storage
            .write_block_transactions(0, StoredTransactions { txs: vec![tx0.clone()] })
            .expect("write txs 0");
        storage
            .write_block_transactions(1, StoredTransactions { txs: vec![tx1.clone()] })
            .expect("write txs 1");

        assert_eq!(
            storage
                .block_transactions(0)
                .expect("tx lookup")
                .expect("txs exist"),
            StoredTransactions { txs: vec![tx0] }
        );
        assert_eq!(
            storage
                .block_transactions(1)
                .expect("tx lookup")
                .expect("txs exist"),
            StoredTransactions { txs: vec![tx1] }
        );

        storage
            .write_block_size(0, StoredBlockSize { size: 1234 })
            .expect("write size");
        assert_eq!(
            storage
                .block_size(0)
                .expect("read size")
                .expect("size exists"),
            StoredBlockSize { size: 1234 }
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_block_bundle_batch_roundtrip() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let header0 = header_with_number(0);
        let header1 = header_with_number(1);
        let tx_hash0 = B256::from([0x11u8; 32]);
        let tx_hash1 = B256::from([0x22u8; 32]);

        let log0 = Log::new_unchecked(
            Address::from([0x01u8; 20]),
            vec![B256::from([0x10u8; 32])],
            Bytes::from(vec![0x01]),
        );
        let log1 = Log::new_unchecked(
            Address::from([0x02u8; 20]),
            vec![B256::from([0x20u8; 32])],
            Bytes::from(vec![0x02]),
        );

        let bundle0 = BlockBundle {
            number: 0,
            header: header0,
            tx_hashes: StoredTxHashes { hashes: vec![tx_hash0] },
            transactions: StoredTransactions {
                txs: vec![StoredTransaction {
                    hash: tx_hash0,
                    from: Some(Address::from([0x0au8; 20])),
                    to: Some(Address::from([0x0bu8; 20])),
                    value: U256::from(1),
                    nonce: 0,
                    signature: None,
                    signing_hash: None,
                }],
            },
            withdrawals: StoredWithdrawals { withdrawals: None },
            size: StoredBlockSize { size: 111 },
            receipts: StoredReceipts {
                receipts: vec![receipt_with_logs(vec![log0])],
            },
            logs: StoredLogs { logs: Vec::new() },
        };

        let bundle1 = BlockBundle {
            number: 1,
            header: header1,
            tx_hashes: StoredTxHashes { hashes: vec![tx_hash1] },
            transactions: StoredTransactions {
                txs: vec![StoredTransaction {
                    hash: tx_hash1,
                    from: Some(Address::from([0x0cu8; 20])),
                    to: None,
                    value: U256::from(2),
                    nonce: 1,
                    signature: None,
                    signing_hash: None,
                }],
            },
            withdrawals: StoredWithdrawals { withdrawals: None },
            size: StoredBlockSize { size: 222 },
            receipts: StoredReceipts {
                receipts: vec![receipt_with_logs(vec![log1])],
            },
            logs: StoredLogs { logs: Vec::new() },
        };

        storage
            .write_block_bundle_batch(&[bundle0, bundle1])
            .expect("write bundle batch");

        assert_eq!(storage.last_indexed_block().unwrap(), Some(1));
        assert!(storage.block_header(0).expect("header lookup").is_some());
        assert!(storage.block_header(1).expect("header lookup").is_some());
        assert_eq!(
            storage
                .block_transactions(0)
                .expect("tx lookup")
                .expect("txs exist")
                .txs
                .len(),
            1
        );
        assert!(storage.block_logs(1).expect("log lookup").is_none());
        let addr_index = storage
            .log_index_by_address_range(Address::from([0x01u8; 20]), 0..=0)
            .expect("address index lookup");
        assert!(addr_index.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_rollback_prunes_segments() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let make_bundle = |number: u64, hash: B256| BlockBundle {
            number,
            header: header_with_number(number),
            tx_hashes: StoredTxHashes { hashes: vec![hash] },
            transactions: StoredTransactions {
                txs: vec![StoredTransaction {
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
            size: StoredBlockSize { size: number + 100 },
            receipts: StoredReceipts {
                receipts: vec![receipt_with_logs(Vec::new())],
            },
            logs: StoredLogs { logs: Vec::new() },
        };

        let bundles = vec![
            make_bundle(0, B256::from([0x10u8; 32])),
            make_bundle(1, B256::from([0x11u8; 32])),
            make_bundle(2, B256::from([0x12u8; 32])),
        ];

        storage
            .write_block_bundle_batch(&bundles)
            .expect("write bundle batch");
        assert!(storage.block_header(2).expect("header 2").is_some());

        storage.rollback_to(0).expect("rollback");
        assert_eq!(storage.last_indexed_block().unwrap(), Some(0));
        assert!(storage.block_header(0).expect("header 0").is_some());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert!(storage.block_tx_hashes(1).expect("hashes 1").is_none());
        assert!(storage.block_transactions(1).expect("txs 1").is_none());
        assert!(storage.block_receipts(1).expect("receipts 1").is_none());
        assert!(storage.block_size(1).expect("size 1").is_none());

        let headers = storage.block_headers_range(0..=2).expect("headers range");
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_gap_fill_skips_missing_rows() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let header2 = header_with_number(2);
        storage
            .write_block_header(2, header2.clone())
            .expect("write header 2");

        assert!(storage.block_header(0).expect("header 0").is_none());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert_eq!(
            storage
                .block_header(2)
                .expect("header 2")
                .expect("header exists"),
            header2
        );

        let headers = storage.block_headers_range(0..=2).expect("headers range");
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, 2);
        assert_eq!(headers[0].1, header2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_log_index_queries() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let topic0 = B256::from([0x11u8; 32]);
        let log = StoredLog {
            address: Address::ZERO,
            topics: vec![topic0],
            block_number: 5,
            block_hash: B256::ZERO,
            transaction_hash: B256::ZERO,
            transaction_index: 0,
            log_index: 2,
            removed: false,
            data: Bytes::from(vec![0x01]),
        };

        storage
            .write_log_indexes(&[log.clone()])
            .expect("write log indexes");

        let by_address = storage
            .log_index_by_address_range(Address::ZERO, 0..=10)
            .expect("address index query");
        assert!(by_address.is_empty());

        let by_topic = storage
            .log_index_by_topic0_range(topic0, 0..=10)
            .expect("topic index query");
        assert!(by_topic.is_empty());

        let empty = storage
            .log_index_by_address_range(Address::ZERO, 6..=10)
            .expect("address range query");
        assert!(empty.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_peer_cache_prunes_and_caps() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_millis() as u64;
        let day_ms = Duration::from_secs(24 * 60 * 60).as_millis() as u64;
        let expire_before = now_ms.saturating_sub(day_ms.saturating_mul(7));

        let expired = StoredPeer {
            peer_id: "peer-expired".to_string(),
            tcp_addr: "127.0.0.1:30303".parse().expect("valid addr"),
            udp_addr: None,
            last_seen_ms: now_ms.saturating_sub(day_ms.saturating_mul(8)),
        };
        let recent1 = StoredPeer {
            peer_id: "peer-recent-1".to_string(),
            tcp_addr: "127.0.0.2:30303".parse().expect("valid addr"),
            udp_addr: Some("127.0.0.2:30303".parse().expect("valid addr")),
            last_seen_ms: now_ms.saturating_sub(day_ms.saturating_mul(2)),
        };
        let recent2 = StoredPeer {
            peer_id: "peer-recent-2".to_string(),
            tcp_addr: "127.0.0.3:30303".parse().expect("valid addr"),
            udp_addr: None,
            last_seen_ms: now_ms.saturating_sub(day_ms),
        };
        let recent3 = StoredPeer {
            peer_id: "peer-recent-3".to_string(),
            tcp_addr: "127.0.0.4:30303".parse().expect("valid addr"),
            udp_addr: None,
            last_seen_ms: now_ms,
        };

        storage.upsert_peer(expired).expect("insert expired");
        storage.upsert_peer(recent1).expect("insert recent1");
        storage.upsert_peer(recent2).expect("insert recent2");
        storage.upsert_peer(recent3).expect("insert recent3");
        storage.flush_peer_cache().expect("flush peers");

        let load = storage
            .load_peers(expire_before, 2)
            .expect("load peers");
        assert_eq!(load.total, 4);
        assert_eq!(load.expired, 1);
        assert_eq!(load.capped, 1);
        assert_eq!(load.peers.len(), 2);
        assert_eq!(load.peers[0].peer_id, "peer-recent-3");
        assert_eq!(load.peers[1].peer_id, "peer-recent-2");

        let peers_after = storage.load_peers(0, 10).expect("load peers");
        assert_eq!(peers_after.total, 2);
        assert_eq!(peers_after.peers.len(), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn storage_log_index_accepts_unsorted_logs() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");
        let log1 = StoredLog {
            address: Address::ZERO,
            topics: vec![],
            block_number: 3,
            block_hash: B256::ZERO,
            transaction_hash: B256::ZERO,
            transaction_index: 0,
            log_index: 1,
            removed: false,
            data: Bytes::from(vec![0x01]),
        };
        let log0 = StoredLog { block_number: 2, log_index: 0, ..log1.clone() };
        let log2 = StoredLog { block_number: 3, log_index: 2, ..log1.clone() };
        storage
            .write_log_indexes(&[log1.clone(), log0.clone(), log2.clone()])
            .expect("write indexes");

        let by_address = storage
            .log_index_by_address_range(Address::ZERO, 0..=10)
            .expect("address index query");
        assert!(by_address.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
