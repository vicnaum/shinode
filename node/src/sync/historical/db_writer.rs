//! Batched DB writer for ingest mode.

use crate::storage::{BlockBundle, Storage};
use crate::sync::historical::stats::{BenchEvent, BenchEventLogger, DbWriteByteTotals, IngestBenchStats};
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
    events: Option<Arc<BenchEventLogger>>,
    start_block: u64,
) -> Result<()> {
    let mut buffer: Vec<BlockBundle> = Vec::with_capacity(config.batch_blocks);
    let mut interval = config.flush_interval.map(tokio::time::interval);
    let mut expected_next = storage
        .last_indexed_block()?
        .map(|block| block.saturating_add(1))
        .unwrap_or(start_block)
        .max(start_block);
    tracing::debug!(
        expected_next,
        batch_blocks = config.batch_blocks,
        "db writer started"
    );

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
                                events.as_ref(),
                                &mut expected_next,
                                config.batch_blocks,
                            )?;
                        }
                    }
                    Some(DbWriterMessage::Flush) => {
                        tracing::info!(
                            buffered_blocks = buffer.len(),
                            expected_next,
                            "db writer flush requested"
                        );
                        if !buffer.is_empty() {
                            flush_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                events.as_ref(),
                                &mut expected_next,
                                config.batch_blocks,
                            )?;
                        }
                    }
                    None => {
                        tracing::info!(
                            buffered_blocks = buffer.len(),
                            expected_next,
                            "db writer channel closed; final flush"
                        );
                        if !buffer.is_empty() {
                            flush_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                events.as_ref(),
                                &mut expected_next,
                                config.batch_blocks,
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
                        events.as_ref(),
                        &mut expected_next,
                        config.batch_blocks,
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
    events: Option<&Arc<BenchEventLogger>>,
    expected_next: &mut u64,
    max_write_blocks: usize,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }

    buffer.sort_by_key(|bundle| bundle.number);
    let mut idx = 0usize;
    while idx < buffer.len() && buffer[idx].number < *expected_next {
        idx += 1;
    }
    if idx > 0 {
        buffer.drain(..idx);
    }
    if buffer.is_empty() {
        return Ok(());
    }

    let mut expected = *expected_next;
    let mut contiguous = 0usize;
    while contiguous < buffer.len() && buffer[contiguous].number == expected {
        expected = expected.saturating_add(1);
        contiguous += 1;
    }
    if contiguous == 0 {
        tracing::debug!(
            expected_next = *expected_next,
            buffer_len = buffer.len(),
            first = buffer.first().map(|b| b.number),
            "db flush: no contiguous prefix ready"
        );
        return Ok(());
    }

    let chunk = max_write_blocks.max(1);
    let last_contiguous = buffer[contiguous.saturating_sub(1)].number;
    tracing::info!(
        expected_next = *expected_next,
        buffer_len = buffer.len(),
        contiguous_blocks = contiguous,
        range_start = buffer[0].number,
        range_end = last_contiguous,
        chunk_blocks = chunk,
        "db flush: flushing contiguous blocks"
    );
    let mut cursor = 0usize;
    while cursor < contiguous {
        let end = (cursor + chunk).min(contiguous);
        let written = &buffer[cursor..end];
        let blocks = written.len() as u64;
        let write_start = written.first().map(|b| b.number).unwrap_or(0);
        let write_end = written.last().map(|b| b.number).unwrap_or(write_start);
        let bytes = if bench.is_some() || events.is_some() {
            Some(db_bytes_for_bundles(written)?)
        } else {
            None
        };
        tracing::debug!(
            start = write_start,
            end = write_end,
            blocks,
            "db flush: writing block bundle batch"
        );
        if let Some(events) = events {
            let bytes_total = bytes
                .as_ref()
                .map(|totals| {
                    totals
                        .headers
                        .saturating_add(totals.tx_hashes)
                        .saturating_add(totals.transactions)
                        .saturating_add(totals.withdrawals)
                        .saturating_add(totals.sizes)
                        .saturating_add(totals.receipts)
                        .saturating_add(totals.logs)
                })
                .unwrap_or(0);
            events.record(BenchEvent::DbFlushStart {
                blocks,
                bytes_total,
            });
        }
        let _span = tracing::trace_span!("db_flush", blocks = blocks).entered();
        let started = Instant::now();
        storage.write_block_bundle_batch(written)?;
        tracing::debug!(
            start = write_start,
            end = write_end,
            blocks,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "db flush: wrote block bundle batch"
        );
        if let Some(bench) = bench {
            bench.record_db_write(blocks, started.elapsed());
            if let Some(bytes) = bytes.as_ref() {
                bench.record_db_write_bytes(*bytes);
            }
        }
        if let Some(events) = events {
            let elapsed_ms = started.elapsed().as_millis() as u64;
            let bytes_total = bytes
                .as_ref()
                .map(|totals| {
                    totals
                        .headers
                        .saturating_add(totals.tx_hashes)
                        .saturating_add(totals.transactions)
                        .saturating_add(totals.withdrawals)
                        .saturating_add(totals.sizes)
                        .saturating_add(totals.receipts)
                        .saturating_add(totals.logs)
                })
                .unwrap_or(0);
            events.record(BenchEvent::DbFlushEnd {
                blocks,
                bytes_total,
                duration_ms: elapsed_ms,
            });
        }
        cursor = end;
    }

    *expected_next = expected;
    buffer.drain(..contiguous);
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
            "stateless-history-node-db-writer-test-{now}-{}-{suffix}",
            std::process::id()
        ));
        path
    }

    fn base_config(data_dir: std::path::PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir,
            peer_cache_dir: None,
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            end_block: None,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
            verbosity: 0,
            benchmark: crate::cli::BenchmarkMode::Disabled,
            benchmark_name: None,
            benchmark_output_dir: PathBuf::from(crate::cli::DEFAULT_BENCHMARK_OUTPUT_DIR),
            benchmark_trace: false,
            benchmark_events: false,
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
        flush_buffer(&storage, &mut buffer, None, None, &mut expected_next, 1)
            .expect("flush buffer");

        assert!(storage.block_header(0).expect("header 0").is_some());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert!(storage.block_header(2).expect("header 2").is_none());
        assert_eq!(expected_next, 1);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].number, 2);

        buffer.push(bundle_with_number(1));
        flush_buffer(&storage, &mut buffer, None, None, &mut expected_next, 1)
            .expect("flush buffer 2");

        assert!(storage.block_header(1).expect("header 1").is_some());
        assert!(storage.block_header(2).expect("header 2").is_some());
        assert_eq!(expected_next, 3);
        assert!(buffer.is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
