//! Storage types and helpers. The sharded backend lives in `sharded`.

use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
use alloy_primitives::{Address, Bytes, Signature, B256, U256};
use eyre::{Result, WrapErr};
use reth_ethereum_primitives::Receipt;
use reth_primitives_traits::{
    serde_bincode_compat::{BincodeReprFor, SerdeBincodeCompat},
    Header,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub(crate) const ZSTD_DICT_MAX_SIZE: usize = 5000;

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredReceipts {
    pub receipts: Vec<Receipt>,
}

type ReceiptBincodeRepr<'a> = BincodeReprFor<'a, Receipt>;

impl SerdeBincodeCompat for StoredReceipts {
    type BincodeRepr<'a> = Vec<ReceiptBincodeRepr<'a>>;

    fn as_repr(&self) -> Self::BincodeRepr<'_> {
        self.receipts
            .iter()
            .map(|receipt| receipt.as_repr())
            .collect()
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
    pub tcp_addr: std::net::SocketAddr,
    pub udp_addr: Option<std::net::SocketAddr>,
    pub last_seen_ms: u64,
    #[serde(default)]
    pub aimd_batch_limit: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerCacheLoad {
    pub peers: Vec<StoredPeer>,
    pub total: usize,
    pub expired: usize,
    pub corrupted: usize,
    pub capped: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StorageConfigKey {
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

mod sharded;

pub use sharded::Storage;

pub(crate) fn encode_bincode_value<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).wrap_err("failed to encode bincode payload")
}

pub(crate) fn decode_bincode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes).wrap_err("failed to decode bincode payload")
}

pub(crate) fn encode_bincode_compat_value<T: SerdeBincodeCompat>(value: &T) -> Result<Vec<u8>> {
    let repr = value.as_repr();
    bincode::serialize(&repr).wrap_err("failed to encode bincode payload")
}

pub(crate) fn decode_bincode_compat_value<T>(bytes: &[u8]) -> Result<T>
where
    T: SerdeBincodeCompat,
    <T as SerdeBincodeCompat>::BincodeRepr<'static>: DeserializeOwned,
{
    let repr: <T as SerdeBincodeCompat>::BincodeRepr<'static> =
        bincode::deserialize(bytes).wrap_err("failed to decode bincode payload")?;
    Ok(<T as SerdeBincodeCompat>::from_repr(repr))
}

pub(crate) fn encode_u64_value(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub(crate) fn decode_u64(bytes: &[u8]) -> Result<u64> {
    if bytes.len() != 8 {
        return Err(eyre::eyre!("invalid u64 encoding"));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(buf))
}
