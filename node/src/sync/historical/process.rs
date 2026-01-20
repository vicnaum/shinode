//! Processing stage for historical sync.

use crate::storage::{
    BlockBundle, StoredBlockSize, StoredLogs, StoredReceipts, StoredTransaction, StoredTransactions,
    StoredTxHashes, StoredWithdrawal, StoredWithdrawals, StoredLog,
};
use crate::sync::{BlockPayload};
use crate::sync::historical::stats::{IngestBenchStats, ProcessTiming};
use crate::sync::historical::types::{FetchedBlock, ProbeRecord};
use alloy_consensus::Transaction as _;
use alloy_primitives::{logs_bloom, TxKind};
use eyre::{eyre, Result};
use reth_ethereum_primitives::{Block, BlockBody};
use reth_primitives_traits::SealedHeader;
use std::time::Instant;

/// Process a fetched block in probe mode.
pub fn process_probe(block: FetchedBlock) -> ProbeRecord {
    let receipts = block.receipts.len() as u64;
    ProbeRecord {
        number: block.number,
        peer_id: block.peer_id,
        receipts,
        timing: block.timing,
    }
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
    let header_hash = SealedHeader::seal_slow(header.clone()).hash();
    let header_hash_us = header_hash_start.elapsed().as_micros() as u64;

    let tx_hashes_start = Instant::now();
    let tx_hashes = body
        .transactions
        .iter()
        .map(|tx| *tx.hash())
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
    let mut stored_transactions = Vec::with_capacity(body.transactions.len());
    for tx in &body.transactions {
        let value = tx.value();
        let signature = tx.signature().clone();
        let signing_hash = tx.signature_hash();
        let to = match tx.kind() {
            TxKind::Call(address) => Some(address),
            TxKind::Create => None,
        };
        stored_transactions.push(StoredTransaction {
            hash: *tx.hash(),
            from: None,
            to,
            value,
            nonce: tx.nonce(),
            signature: Some(signature),
            signing_hash: Some(signing_hash),
        });
    }
    let transactions_us = txs_start.elapsed().as_micros() as u64;

    let withdrawals_start = Instant::now();
    let stored_withdrawals = StoredWithdrawals {
        withdrawals: body.withdrawals.as_ref().map(|withdrawals| {
            withdrawals
                .as_ref()
                .iter()
                .map(|withdrawal| StoredWithdrawal {
                    index: withdrawal.index,
                    validator_index: withdrawal.validator_index,
                    address: withdrawal.address,
                    amount: withdrawal.amount,
                })
                .collect()
        }),
    };
    let withdrawals_us = withdrawals_start.elapsed().as_micros() as u64;

    let block_size_start = Instant::now();
    let block_size = block_rlp_size(&header, &body);
    let block_size_us = block_size_start.elapsed().as_micros() as u64;

    let bloom_start = Instant::now();
    let computed_bloom = logs_bloom(
        receipts
            .iter()
            .flat_map(|receipt| receipt.logs.iter().map(|log| log.as_ref())),
    );
    let logs_bloom_us = bloom_start.elapsed().as_micros() as u64;
    let mut stored_header = header.clone();
    stored_header.logs_bloom = computed_bloom;

    let logs_start = Instant::now();
    let mut block_logs = Vec::new();
    for (tx_index, (tx_hash, receipt)) in tx_hashes
        .iter()
        .zip(receipts.iter())
        .enumerate()
    {
        for (log_index, log) in receipt.logs.iter().cloned().enumerate() {
            let alloy_primitives::Log { address, data } = log;
            let (topics, data) = data.split();
            let stored_log = StoredLog {
                address,
                topics,
                data,
                block_number: header.number,
                block_hash: header_hash,
                transaction_hash: *tx_hash,
                transaction_index: tx_index as u64,
                log_index: log_index as u64,
                removed: false,
            };
            block_logs.push(stored_log);
        }
    }
    let logs_build_us = logs_start.elapsed().as_micros() as u64;
    let log_count = block_logs.len() as u64;

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
        logs: StoredLogs { logs: block_logs },
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
            logs_bloom_us,
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
