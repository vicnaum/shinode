//! Processing stage for historical sync.

use crate::chain::{BaseChain, EthereumChain};
use crate::storage::{
    BlockBundle, StoredBlockSize, StoredReceipts, StoredTransaction, StoredTransactions,
    StoredTxHashes,
};
use crate::sync::historical::stats::{IngestBenchStats, ProcessTiming};
use crate::sync::BlockPayload;
use alloy_consensus::{Block, BlockBody};
use alloy_consensus::{SignableTransaction, Transaction as _, TxReceipt};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Keccak256, B256};
use bytes::buf::UninitSlice;
use eyre::{eyre, Result};
use reth_ethereum_primitives::TransactionSigned;
use reth_primitives_traits::SealedHeader;
use std::mem::MaybeUninit;
use std::time::Instant;

/// Converts chain-specific transactions and receipts into storage representations.
pub trait IngestChain: crate::chain::ChainTypes {
    fn tx_hashes(txs: &[Self::SignedTx]) -> Vec<B256>;
    fn transactions(
        txs: &[Self::SignedTx],
        receipts: &[Self::Receipt],
        hashes: &[B256],
    ) -> Result<StoredTransactions>;
    fn store_receipts(receipts: Vec<Self::Receipt>) -> StoredReceipts;

    fn process_ingest(
        payload: BlockPayload<Self>,
        bench: Option<&IngestBenchStats>,
    ) -> Result<(BlockBundle, u64)> {
        process_ingest(payload, bench)
    }
}

impl IngestChain for EthereumChain {
    fn tx_hashes(txs: &[Self::SignedTx]) -> Vec<B256> {
        let mut buf = KeccakBuf::new();
        txs.iter().map(|tx| tx_hash_fast(&mut buf, tx)).collect()
    }

    fn transactions(
        txs: &[Self::SignedTx],
        _receipts: &[Self::Receipt],
        hashes: &[B256],
    ) -> Result<StoredTransactions> {
        let mut buf = KeccakBuf::new();
        Ok(StoredTransactions {
            txs: txs
                .iter()
                .zip(hashes)
                .map(|(tx, hash)| StoredTransaction {
                    hash: *hash,
                    from: None,
                    to: tx.to(),
                    value: tx.value(),
                    nonce: Some(tx.nonce()),
                    signature: Some(*tx.signature()),
                    signing_hash: Some(signing_hash_fast(&mut buf, tx)),
                })
                .collect(),
        })
    }

    fn store_receipts(receipts: Vec<Self::Receipt>) -> StoredReceipts {
        StoredReceipts::Ethereum(receipts)
    }
}

impl IngestChain for BaseChain {
    fn tx_hashes(txs: &[Self::SignedTx]) -> Vec<B256> {
        // Hash the canonical typed envelope, including the 0x7e deposit prefix.
        txs.iter()
            .map(|tx| alloy_primitives::keccak256(tx.encoded_2718()))
            .collect()
    }

    fn transactions(
        txs: &[Self::SignedTx],
        receipts: &[Self::Receipt],
        hashes: &[B256],
    ) -> Result<StoredTransactions> {
        use op_alloy_consensus::OpTxEnvelope;
        use reth_optimism_primitives::OpReceipt;
        let mut stored = Vec::with_capacity(txs.len());
        for ((tx, receipt), hash) in txs.iter().zip(receipts).zip(hashes) {
            eyre::ensure!(
                u8::from(tx.tx_type()) == u8::from(receipt.tx_type()),
                "transaction/receipt type mismatch"
            );
            let (from, nonce, signing_hash) = match tx {
                OpTxEnvelope::Deposit(deposit) => {
                    let OpReceipt::Deposit(receipt) = receipt else {
                        return Err(eyre!("deposit transaction requires a deposit receipt"));
                    };
                    (Some(deposit.from), receipt.deposit_nonce, None)
                }
                OpTxEnvelope::Legacy(tx) => {
                    (None, Some(tx.tx().nonce()), Some(tx.signature_hash()))
                }
                OpTxEnvelope::Eip2930(tx) => {
                    (None, Some(tx.tx().nonce()), Some(tx.signature_hash()))
                }
                OpTxEnvelope::Eip1559(tx) => {
                    (None, Some(tx.tx().nonce()), Some(tx.signature_hash()))
                }
                OpTxEnvelope::Eip7702(tx) => {
                    (None, Some(tx.tx().nonce()), Some(tx.signature_hash()))
                }
            };
            stored.push(StoredTransaction {
                hash: *hash,
                from,
                to: tx.to(),
                value: tx.value(),
                nonce,
                signature: tx.signature().copied(),
                signing_hash,
            });
        }
        Ok(StoredTransactions { txs: stored })
    }

    fn store_receipts(receipts: Vec<Self::Receipt>) -> StoredReceipts {
        StoredReceipts::Op(receipts)
    }
}

const KECCAK_SCRATCH_LEN: usize = 4096;

struct KeccakBuf {
    hasher: Keccak256,
    scratch: [MaybeUninit<u8>; KECCAK_SCRATCH_LEN],
}

impl KeccakBuf {
    fn new() -> Self {
        Self {
            hasher: Keccak256::new(),
            scratch: [MaybeUninit::uninit(); KECCAK_SCRATCH_LEN],
        }
    }

    fn reset(&mut self) {
        self.hasher = Keccak256::new();
    }

    fn finalize_reset(&mut self) -> B256 {
        let hasher = std::mem::replace(&mut self.hasher, Keccak256::new());
        hasher.finalize()
    }
}

unsafe impl bytes::BufMut for KeccakBuf {
    fn remaining_mut(&self) -> usize {
        usize::MAX
    }

    fn chunk_mut(&mut self) -> &mut UninitSlice {
        UninitSlice::uninit(&mut self.scratch)
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        if cnt == 0 {
            return;
        }
        debug_assert!(cnt <= self.scratch.len());
        let slice = std::slice::from_raw_parts(self.scratch.as_ptr().cast::<u8>(), cnt);
        self.hasher.update(slice);
    }
}

fn tx_hash_fast(buf: &mut KeccakBuf, tx: &TransactionSigned) -> B256 {
    buf.reset();
    match tx {
        TransactionSigned::Legacy(signed) => signed.eip2718_encode(buf),
        TransactionSigned::Eip2930(signed) => signed.eip2718_encode(buf),
        TransactionSigned::Eip1559(signed) => signed.eip2718_encode(buf),
        TransactionSigned::Eip4844(signed) => signed.eip2718_encode(buf),
        TransactionSigned::Eip7702(signed) => signed.eip2718_encode(buf),
    }
    buf.finalize_reset()
}

fn signing_hash_fast(buf: &mut KeccakBuf, tx: &TransactionSigned) -> B256 {
    buf.reset();
    match tx {
        TransactionSigned::Legacy(signed) => signed.tx().encode_for_signing(buf),
        TransactionSigned::Eip2930(signed) => signed.tx().encode_for_signing(buf),
        TransactionSigned::Eip1559(signed) => signed.tx().encode_for_signing(buf),
        TransactionSigned::Eip4844(signed) => signed.tx().encode_for_signing(buf),
        TransactionSigned::Eip7702(signed) => signed.tx().encode_for_signing(buf),
    }
    buf.finalize_reset()
}

/// Process a full payload into a storage bundle (ingest mode).
pub fn process_ingest<C: IngestChain>(
    payload: BlockPayload<C>,
    bench: Option<&IngestBenchStats>,
) -> Result<(BlockBundle, u64)> {
    let BlockPayload {
        header,
        body,
        receipts,
    } = payload;

    let total_start = Instant::now();

    let header_hash_start = Instant::now();
    let _header_hash = SealedHeader::seal_slow(header.clone()).hash();
    let header_hash_us = header_hash_start.elapsed().as_micros() as u64;

    let tx_hashes_start = Instant::now();
    let tx_hashes = C::tx_hashes(&body.transactions);
    let tx_hashes_us = tx_hashes_start.elapsed().as_micros() as u64;
    if tx_hashes.len() != receipts.len() {
        return Err(eyre!(
            "tx hash count {} does not match receipts count {} for block {}",
            tx_hashes.len(),
            receipts.len(),
            header.number
        ));
    }

    let txs_start = Instant::now();
    let stored_transactions = C::transactions(&body.transactions, &receipts, &tx_hashes)?;
    let transactions_us = txs_start.elapsed().as_micros() as u64;

    let withdrawals_us = 0;

    let block_size_start = Instant::now();
    let block_size = block_rlp_size(&header, &body);
    let block_size_us = block_size_start.elapsed().as_micros() as u64;

    let stored_header = header.clone();

    let logs_start = Instant::now();
    let mut log_count = 0u64;
    for receipt in &receipts {
        log_count = log_count.saturating_add(receipt.logs().len() as u64);
    }
    let logs_build_us = logs_start.elapsed().as_micros() as u64;

    let bundle = BlockBundle {
        number: header.number,
        header: stored_header,
        tx_hashes: StoredTxHashes { hashes: tx_hashes },
        transactions: stored_transactions,
        size: StoredBlockSize { size: block_size },
        receipts: C::store_receipts(receipts),
    };

    if let Some(bench) = bench {
        let total_us = total_start.elapsed().as_micros() as u64;
        bench.record_process(ProcessTiming {
            total_us,
            header_hash_us,
            tx_hashes_us,
            transactions_us,
            withdrawals_us,
            block_size_us,
            logs_build_us,
        });
    }

    Ok((bundle, log_count))
}

fn block_rlp_size<T: Clone + alloy_rlp::Encodable>(
    header: &reth_primitives_traits::Header,
    body: &BlockBody<T>,
) -> u64 {
    let block = Block {
        header: header.clone(),
        body: body.clone(),
    };
    alloy_rlp::encode(&block).len() as u64
}
