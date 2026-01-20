//! Processing stage for historical sync.

use crate::storage::{
    BlockBundle, StoredBlockSize, StoredLogs, StoredReceipts, StoredTransaction, StoredTransactions,
    StoredTxHashes, StoredWithdrawals,
};
use crate::sync::{BlockPayload};
use crate::sync::historical::stats::{IngestBenchStats, ProcessTiming};
use crate::sync::historical::types::{FetchedBlock, ProbeRecord};
use alloy_consensus::{SignableTransaction, Transaction as _};
use alloy_primitives::{B256, Keccak256, TxKind};
use bytes::buf::UninitSlice;
use eyre::{eyre, Result};
use reth_ethereum_primitives::{Block, BlockBody, TransactionSigned};
use reth_primitives_traits::SealedHeader;
use std::mem::MaybeUninit;
use std::time::Instant;

/// Process a fetched block in probe mode.
#[allow(dead_code)]
pub fn process_probe(block: FetchedBlock) -> ProbeRecord {
    let receipts = block.receipts.len() as u64;
    ProbeRecord {
        number: block.number,
        peer_id: block.peer_id,
        receipts,
        timing: block.timing,
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
pub fn process_ingest(
    payload: BlockPayload,
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
    let mut tx_hasher = KeccakBuf::new();
    let tx_hashes = body
        .transactions
        .iter()
        .map(|tx| tx_hash_fast(&mut tx_hasher, tx))
        .collect::<Vec<_>>();
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
    let mut signing_hasher = KeccakBuf::new();
    let mut stored_transactions = Vec::with_capacity(body.transactions.len());
    for (tx, tx_hash) in body.transactions.iter().zip(tx_hashes.iter()) {
        let value = tx.value();
        let signature = tx.signature().clone();
        let signing_hash = signing_hash_fast(&mut signing_hasher, tx);
        let to = match tx.kind() {
            TxKind::Call(address) => Some(address),
            TxKind::Create => None,
        };
        stored_transactions.push(StoredTransaction {
            hash: *tx_hash,
            from: None,
            to,
            value,
            nonce: tx.nonce(),
            signature: Some(signature),
            signing_hash: Some(signing_hash),
        });
    }
    let transactions_us = txs_start.elapsed().as_micros() as u64;

    let stored_withdrawals = StoredWithdrawals { withdrawals: None };
    let withdrawals_us = 0;

    let block_size_start = Instant::now();
    let block_size = block_rlp_size(&header, &body);
    let block_size_us = block_size_start.elapsed().as_micros() as u64;

    let stored_header = header.clone();

    let logs_start = Instant::now();
    let mut log_count = 0u64;
    for receipt in receipts.iter() {
        log_count = log_count.saturating_add(receipt.logs.len() as u64);
    }
    let logs_build_us = logs_start.elapsed().as_micros() as u64;

    let bundle = BlockBundle {
        number: header.number,
        header: stored_header,
        tx_hashes: StoredTxHashes { hashes: tx_hashes },
        transactions: StoredTransactions {
            txs: stored_transactions,
        },
        withdrawals: stored_withdrawals,
        size: StoredBlockSize { size: block_size },
        receipts: StoredReceipts { receipts },
        logs: StoredLogs { logs: Vec::new() },
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

fn block_rlp_size(header: &reth_primitives_traits::Header, body: &BlockBody) -> u64 {
    let block = Block {
        header: header.clone(),
        body: body.clone(),
    };
    alloy_rlp::encode(&block).len() as u64
}
