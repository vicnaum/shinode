//! Storage types and helpers. The sharded backend lives in `sharded`.

use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
use alloy_consensus::TxReceipt;
use alloy_primitives::{Address, Signature, B256, U256};
use eyre::{Result, WrapErr};
use reth_ethereum_primitives::Receipt;
use reth_optimism_primitives::OpReceipt;

use reth_primitives_traits::{serde_bincode_compat::SerdeBincodeCompat, Header};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

mod codec;
pub mod durable;
mod lock;
pub use lock::DataDirLock;
pub use codec::StorageCodec;

pub const ZSTD_DICT_MAX_SIZE: usize = 5000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTxHashes {
    pub hashes: Vec<B256>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTransaction {
    pub hash: B256,
    #[serde(default)]
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub value: U256,
    /// Ordinary transaction nonce, or deposit receipt nonce when available.
    pub nonce: Option<u64>,
    #[serde(default)]
    pub signature: Option<Signature>,
    #[serde(default)]
    pub signing_hash: Option<B256>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredTransactions {
    pub txs: Vec<StoredTransaction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredBlockSize {
    pub size: u64,
}

/// Tagged in memory; disk encoding is selected by database metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredReceipts {
    Ethereum(Vec<Receipt>),
    Op(Vec<OpReceipt>),
}

impl StoredReceipts {
    pub const fn codec(&self) -> StorageCodec {
        match self {
            Self::Ethereum(_) => StorageCodec::EthereumV1,
            Self::Op(_) => StorageCodec::OpV1,
        }
    }

    pub const fn len(&self) -> usize {
        match self {
            Self::Ethereum(r) => r.len(),
            Self::Op(r) => r.len(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = StoredReceiptRef<'_>> {
        (0..self.len()).map(move |i| match self {
            Self::Ethereum(r) => StoredReceiptRef::Ethereum(&r[i]),
            Self::Op(r) => StoredReceiptRef::Op(&r[i]),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredReceiptRef<'a> {
    Ethereum(&'a Receipt),
    Op(&'a OpReceipt),
}

impl TxReceipt for StoredReceiptRef<'_> {
    type Log = alloy_primitives::Log;

    fn status_or_post_state(&self) -> alloy_consensus::Eip658Value {
        match self {
            Self::Ethereum(r) => r.status_or_post_state(),
            Self::Op(r) => r.status_or_post_state(),
        }
    }
    fn status(&self) -> bool {
        match self {
            Self::Ethereum(r) => r.status(),
            Self::Op(r) => r.status(),
        }
    }
    fn bloom(&self) -> alloy_primitives::Bloom {
        match self {
            Self::Ethereum(r) => r.bloom(),
            Self::Op(r) => r.bloom(),
        }
    }
    fn cumulative_gas_used(&self) -> u64 {
        match self {
            Self::Ethereum(r) => r.cumulative_gas_used(),
            Self::Op(r) => r.cumulative_gas_used(),
        }
    }
    fn logs(&self) -> &[Self::Log] {
        match self {
            Self::Ethereum(r) => r.logs(),
            Self::Op(r) => r.logs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockBundle {
    pub number: u64,
    pub header: Header,
    pub tx_hashes: StoredTxHashes,
    pub transactions: StoredTransactions,
    pub size: StoredBlockSize,
    pub receipts: StoredReceipts,
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
pub struct StorageConfigKey {
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

pub use sharded::{Storage, StorageAggregateStats};

pub fn encode_bincode_value<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).wrap_err("failed to encode bincode payload")
}

pub fn decode_bincode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes).wrap_err("failed to decode bincode payload")
}

pub fn encode_bincode_compat_value<T: SerdeBincodeCompat>(value: &T) -> Result<Vec<u8>> {
    let repr = value.as_repr();
    bincode::serialize(&repr).wrap_err("failed to encode bincode payload")
}

pub fn decode_bincode_compat_value<T>(bytes: &[u8]) -> Result<T>
where
    T: SerdeBincodeCompat,
    <T as SerdeBincodeCompat>::BincodeRepr<'static>: DeserializeOwned,
{
    let repr: <T as SerdeBincodeCompat>::BincodeRepr<'static> =
        bincode::deserialize(bytes).wrap_err("failed to decode bincode payload")?;
    Ok(<T as SerdeBincodeCompat>::from_repr(repr))
}

pub fn encode_u64_value(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub fn decode_u64(bytes: &[u8]) -> Result<u64> {
    if bytes.len() != 8 {
        return Err(eyre::eyre!("invalid u64 encoding"));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Ok(u64::from_le_bytes(buf))
}

#[cfg(test)]
mod chain_tests;
