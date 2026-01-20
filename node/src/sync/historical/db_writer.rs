//! Batched DB writer for ingest mode.

use crate::storage::{BlockBundle, Storage};
use crate::sync::historical::stats::{DbWriteByteTotals, IngestBenchStats};
use eyre::Result;
use reth_primitives_traits::serde_bincode_compat::SerdeBincodeCompat;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy)]
pub struct DbWriteConfig {
    pub batch_blocks: usize,
    pub flush_interval: Option<Duration>,
}

impl DbWriteConfig {
    pub fn new(batch_blocks: u64, flush_interval_ms: Option<u64>) -> Self {
        let batch_blocks = batch_blocks.max(1) as usize;
        let flush_interval = flush_interval_ms.map(Duration::from_millis);
        Self {
            batch_blocks,
            flush_interval,
        }
    }
}

pub enum DbWriterMessage {
    Block(BlockBundle),
    Flush,
}

pub async fn run_db_writer(
    storage: Arc<Storage>,
    mut rx: mpsc::Receiver<DbWriterMessage>,
    config: DbWriteConfig,
    bench: Option<Arc<IngestBenchStats>>,
    start_block: u64,
) -> Result<()> {
    let mut buffer: Vec<BlockBundle> = Vec::with_capacity(config.batch_blocks);
    let mut interval = config.flush_interval.map(tokio::time::interval);
    let mut expected_next = storage
        .last_indexed_block()?
        .map(|block| block.saturating_add(1))
        .unwrap_or(start_block)
        .max(start_block);

    loop {
        tokio::select! {
            maybe_msg = rx.recv() => {
                match maybe_msg {
                    Some(DbWriterMessage::Block(block)) => {
                        buffer.push(block);
                        if buffer.len() >= config.batch_blocks {
                            flush_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                &mut expected_next,
                            )?;
                        }
                    }
                    Some(DbWriterMessage::Flush) => {
                        if !buffer.is_empty() {
                            flush_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                &mut expected_next,
                            )?;
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            flush_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                &mut expected_next,
                            )?;
                        }
                        break;
                    }
                }
            }
            _ = async {
                if let Some(interval) = interval.as_mut() {
                    interval.tick().await;
                }
            }, if interval.is_some() => {
                if !buffer.is_empty() {
                    flush_buffer(
                        &storage,
                        &mut buffer,
                        bench.as_ref(),
                        &mut expected_next,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn flush_buffer(
    storage: &Storage,
    buffer: &mut Vec<BlockBundle>,
    bench: Option<&Arc<IngestBenchStats>>,
    expected_next: &mut u64,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }

    buffer.sort_by_key(|bundle| bundle.number);
    let mut idx = 0;
    while idx < buffer.len() && buffer[idx].number < *expected_next {
        idx += 1;
    }
    let start_idx = idx;
    let mut expected = *expected_next;
    while idx < buffer.len() && buffer[idx].number == expected {
        expected = expected.saturating_add(1);
        idx += 1;
    }

    if idx > start_idx {
        let written = &buffer[start_idx..idx];
        let blocks = written.len() as u64;
        let started = Instant::now();
        storage.write_block_bundle_batch(written)?;
        if let Some(bench) = bench {
            bench.record_db_write(blocks, started.elapsed());
            let bytes = db_bytes_for_bundles(written)?;
            bench.record_db_write_bytes(bytes);
        }
        *expected_next = expected;
    }

    let remaining: Vec<_> = buffer.drain(idx..).collect();
    buffer.clear();
    buffer.extend(remaining);
    Ok(())
}

fn encoded_len<T: Serialize>(value: &T) -> Result<u64> {
    Ok(bincode::serialize(value)?.len() as u64)
}

fn encoded_len_compat<T: SerdeBincodeCompat>(value: &T) -> Result<u64> {
    Ok(bincode::serialize(&value.as_repr())?.len() as u64)
}

fn db_bytes_for_bundles(bundles: &[BlockBundle]) -> Result<DbWriteByteTotals> {
    let mut totals = DbWriteByteTotals::default();
    for bundle in bundles {
        totals.headers = totals
            .headers
            .saturating_add(encoded_len_compat(&bundle.header)?);
        totals.tx_hashes = totals
            .tx_hashes
            .saturating_add(encoded_len(&bundle.tx_hashes)?);
        totals.transactions = totals
            .transactions
            .saturating_add(encoded_len(&bundle.transactions)?);
        totals.sizes = totals.sizes.saturating_add(8);
        totals.receipts = totals
            .receipts
            .saturating_add(encoded_len_compat(&bundle.receipts)?);
    }
    Ok(totals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode};
    use crate::storage::{
        StoredBlockSize, StoredLogs, StoredReceipts, StoredTransaction, StoredTransactions,
        StoredTxHashes, StoredWithdrawals,
    };
    use alloy_primitives::{Address, B256, U256};
    use reth_ethereum_primitives::Receipt;
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
            "stateless-history-node-db-writer-test-{now}-{}-{suffix}",
            std::process::id()
        ));
        path
    }

    fn base_config(data_dir: std::path::PathBuf) -> NodeConfig {
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
            benchmark: crate::cli::BenchmarkMode::Disabled,
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

    fn header_with_number(number: u64) -> reth_primitives_traits::Header {
        let mut header = reth_primitives_traits::Header::default();
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
            size: StoredBlockSize { size: 0 },
            receipts: StoredReceipts { receipts: Vec::<Receipt>::new() },
            logs: StoredLogs { logs: Vec::new() },
        }
    }

    #[test]
    fn flush_buffer_writes_contiguous_prefix() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("open storage");

        let mut buffer = vec![bundle_with_number(2), bundle_with_number(0)];
        let mut expected_next = 0;
        flush_buffer(&storage, &mut buffer, None, &mut expected_next)
            .expect("flush buffer");

        assert!(storage.block_header(0).expect("header 0").is_some());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert!(storage.block_header(2).expect("header 2").is_none());
        assert_eq!(expected_next, 1);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].number, 2);

        buffer.push(bundle_with_number(1));
        flush_buffer(&storage, &mut buffer, None, &mut expected_next)
            .expect("flush buffer 2");

        assert!(storage.block_header(1).expect("header 1").is_some());
        assert!(storage.block_header(2).expect("header 2").is_some());
        assert_eq!(expected_next, 3);
        assert!(buffer.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
