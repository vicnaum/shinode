//! Chain-selected segment codecs. Ethereum retains its original bincode layout.

use super::{
    decode_bincode, encode_bincode_value, StoredReceipts, StoredTransaction, StoredTransactions,
};
use alloy_primitives::{Address, Signature, B256, U256};
use eyre::{ensure, Result};
use reth_primitives_traits::serde_bincode_compat::{BincodeReprFor, SerdeBincodeCompat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageCodec {
    EthereumV1,
    OpV1,
}

impl StorageCodec {
    pub const fn for_chain(chain: crate::chain::ChainKind) -> Self {
        match chain {
            crate::chain::ChainKind::Mainnet => Self::EthereumV1,
            crate::chain::ChainKind::Base => Self::OpV1,
        }
    }

    pub fn encode_receipts(self, receipts: &StoredReceipts) -> Result<Vec<u8>> {
        match (self, receipts) {
            (Self::EthereumV1, StoredReceipts::Ethereum(receipts)) => encode_bincode_value(
                &receipts
                    .iter()
                    .map(SerdeBincodeCompat::as_repr)
                    .collect::<Vec<_>>(),
            ),
            (Self::OpV1, StoredReceipts::Op(receipts)) => encode_bincode_value(
                &receipts
                    .iter()
                    .map(SerdeBincodeCompat::as_repr)
                    .collect::<Vec<_>>(),
            ),
            _ => eyre::bail!("receipt payload does not match storage codec"),
        }
    }

    pub fn decode_receipts(self, bytes: &[u8]) -> Result<StoredReceipts> {
        match self {
            Self::EthereumV1 => {
                type Receipt = reth_ethereum_primitives::Receipt;
                let repr: Vec<BincodeReprFor<'static, Receipt>> = decode_bincode(bytes)?;
                Ok(StoredReceipts::Ethereum(
                    repr.into_iter().map(Receipt::from_repr).collect(),
                ))
            }
            Self::OpV1 => {
                type Receipt = reth_optimism_primitives::OpReceipt;
                let repr: Vec<BincodeReprFor<'static, Receipt>> = decode_bincode(bytes)?;
                Ok(StoredReceipts::Op(
                    repr.into_iter().map(Receipt::from_repr).collect(),
                ))
            }
        }
    }

    pub fn encode_transactions(self, transactions: &StoredTransactions) -> Result<Vec<u8>> {
        match self {
            Self::EthereumV1 => {
                let legacy = transactions
                    .txs
                    .iter()
                    .map(LegacyTransaction::try_from)
                    .collect::<Result<Vec<_>>>()?;
                encode_bincode_value(&legacy)
            }
            Self::OpV1 => encode_bincode_value(transactions),
        }
    }

    pub fn decode_transactions(self, bytes: &[u8]) -> Result<StoredTransactions> {
        match self {
            Self::EthereumV1 => {
                let legacy: Vec<LegacyTransaction> = decode_bincode(bytes)?;
                Ok(StoredTransactions {
                    txs: legacy.into_iter().map(StoredTransaction::from).collect(),
                })
            }
            Self::OpV1 => decode_bincode(bytes),
        }
    }
}

// Exact pre-Base field order and types. A one-field StoredTransactions wrapper
// has the same bincode representation as its Vec. Do not serialize the in-memory
// optional nonce into an Ethereum segment.
#[derive(Serialize, Deserialize)]
struct LegacyTransaction {
    hash: B256,
    from: Option<Address>,
    to: Option<Address>,
    value: U256,
    nonce: u64,
    signature: Option<Signature>,
    signing_hash: Option<B256>,
}

impl TryFrom<&StoredTransaction> for LegacyTransaction {
    type Error = eyre::Report;

    fn try_from(tx: &StoredTransaction) -> Result<Self> {
        ensure!(tx.nonce.is_some(), "Ethereum transaction nonce is missing");
        Ok(Self {
            hash: tx.hash,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            nonce: tx.nonce.unwrap_or_default(),
            signature: tx.signature,
            signing_hash: tx.signing_hash,
        })
    }
}

impl From<LegacyTransaction> for StoredTransaction {
    fn from(tx: LegacyTransaction) -> Self {
        Self {
            hash: tx.hash,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            nonce: Some(tx.nonce),
            signature: tx.signature,
            signing_hash: tx.signing_hash,
        }
    }
}
