//! Batched DB writer for ingest mode.

use crate::storage::{BlockBundle, Storage};
use crate::sync::historical::stats::{
    BenchEvent, BenchEventLogger, DbWriteByteTotals, IngestBenchStats,
};
use eyre::Result;
use reth_primitives_traits::serde_bincode_compat::SerdeBincodeCompat;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

async fn flush_fast_sync_buffer(
    storage: &Arc<Storage>,
    buffer: &mut Vec<BlockBundle>,
    bench: Option<&Arc<IngestBenchStats>>,
    events: Option<&Arc<BenchEventLogger>>,
    remaining_per_shard: Option<&Arc<Mutex<HashMap<u64, usize>>>>,
    compactions: &mut Vec<JoinHandle<Result<()>>>,
    semaphore: &Arc<Semaphore>,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }

    let bytes = if bench.is_some() || events.is_some() {
        Some(db_bytes_for_bundles(buffer)?)
    } else {
        None
    };
    if let Some(events) = events.as_ref() {
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
            blocks: buffer.len() as u64,
            bytes_total,
        });
    }

    let started = Instant::now();
    let written_blocks = storage.write_block_bundles_wal(buffer)?;
    if let Some(bench) = bench.as_ref() {
        bench.record_db_write(buffer.len() as u64, started.elapsed());
        if let Some(bytes) = bytes.as_ref() {
            bench.record_db_write_bytes(*bytes);
        }
    }
    if let Some(events) = events.as_ref() {
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
            blocks: buffer.len() as u64,
            bytes_total,
            duration_ms: elapsed_ms,
        });
    }

    if let Some(remaining) = remaining_per_shard.as_ref() {
        let mut shards_to_compact = Vec::new();
        let mut guard = remaining.lock().await;
        for number in written_blocks {
            let shard_start = (number / storage.shard_size()) * storage.shard_size();
            if let Some(count) = guard.get_mut(&shard_start) {
                if *count > 0 {
                    *count -= 1;
                    if *count == 0 {
                        shards_to_compact.push(shard_start);
                    }
                }
            }
        }
        drop(guard);

        for shard_start in shards_to_compact {
            let storage = Arc::clone(storage);
            let semaphore = Arc::clone(semaphore);
            let events = events.cloned();
            compactions.push(tokio::spawn(async move {
                let _permit = semaphore.acquire_owned().await?;
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::CompactionStart { shard_start });
                }
                let started = Instant::now();
                let result = storage.compact_shard(shard_start);
                if let Some(events) = events.as_ref() {
                    events.record(BenchEvent::CompactionEnd {
                        shard_start,
                        duration_ms: started.elapsed().as_millis() as u64,
                    });
                }
                result
            }));
        }
    }

    buffer.clear();
    Ok(())
}

fn compact_and_seal(
    storage: &Storage,
    events: Option<&Arc<BenchEventLogger>>,
) -> Result<()> {
    if let Some(events) = events.as_ref() {
        events.record(BenchEvent::CompactAllDirtyStart);
    }
    let started = Instant::now();
    storage.compact_all_dirty()?;
    if let Some(events) = events.as_ref() {
        events.record(BenchEvent::CompactAllDirtyEnd {
            duration_ms: started.elapsed().as_millis() as u64,
        });
    }

    if let Some(events) = events.as_ref() {
        events.record(BenchEvent::SealCompletedStart);
    }
    let started = Instant::now();
    storage.seal_completed_shards()?;
    if let Some(events) = events.as_ref() {
        events.record(BenchEvent::SealCompletedEnd {
            duration_ms: started.elapsed().as_millis() as u64,
        });
    }

    Ok(())
}

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
    #[allow(dead_code)]
    Flush,
    Finalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbWriteMode {
    FastSync,
    Follow,
}

pub async fn run_db_writer(
    storage: Arc<Storage>,
    mut rx: mpsc::Receiver<DbWriterMessage>,
    config: DbWriteConfig,
    bench: Option<Arc<IngestBenchStats>>,
    events: Option<Arc<BenchEventLogger>>,
    mode: DbWriteMode,
    remaining_per_shard: Option<Arc<Mutex<HashMap<u64, usize>>>>,
) -> Result<()> {
    let mut interval = config.flush_interval.map(tokio::time::interval);
    // Compaction is a large, bursty workload (reads WAL + writes new segment files).
    // Serializing it avoids compaction fan-out doubling peak memory/IO.
    let semaphore = Arc::new(Semaphore::new(1));
    let mut compactions = Vec::new();
    let mut buffer: Vec<BlockBundle> = Vec::new();
    let mut gauge_tick = tokio::time::interval(Duration::from_secs(10));

    tracing::debug!(
        mode = ?mode,
        batch_blocks = config.batch_blocks,
        "db writer started"
    );

    loop {
        tokio::select! {
            maybe_msg = rx.recv() => {
                match maybe_msg {
                    Some(DbWriterMessage::Block(block)) => {
                        if mode == DbWriteMode::FastSync {
                            buffer.push(block);
                            if buffer.len() >= config.batch_blocks {
                                flush_fast_sync_buffer(
                                    &storage,
                                    &mut buffer,
                                    bench.as_ref(),
                                    events.as_ref(),
                                    remaining_per_shard.as_ref(),
                                    &mut compactions,
                                    &semaphore,
                                )
                                .await?;
                            }
                        } else {
                            let bytes = if bench.is_some() || events.is_some() {
                                Some(db_bytes_for_bundles(&[block.clone()])?)
                            } else {
                                None
                            };
                            if let Some(events) = events.as_ref() {
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
                                    blocks: 1,
                                    bytes_total,
                                });
                            }
                            let started = Instant::now();
                            storage.write_block_bundle_follow(&block)?;
                            if let Some(bench) = bench.as_ref() {
                                bench.record_db_write(1, started.elapsed());
                                if let Some(bytes) = bytes.as_ref() {
                                    bench.record_db_write_bytes(*bytes);
                                }
                            }
                            if let Some(events) = events.as_ref() {
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
                                    blocks: 1,
                                    bytes_total,
                                    duration_ms: elapsed_ms,
                                });
                            }
                        }
                    }
                    Some(DbWriterMessage::Flush) => {
                        if mode == DbWriteMode::FastSync {
                            flush_fast_sync_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                events.as_ref(),
                                remaining_per_shard.as_ref(),
                                &mut compactions,
                                &semaphore,
                            )
                            .await?;
                            compact_and_seal(&storage, events.as_ref())?;
                        }
                    }
                    Some(DbWriterMessage::Finalize) => {
                        if mode == DbWriteMode::FastSync {
                            flush_fast_sync_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                events.as_ref(),
                                remaining_per_shard.as_ref(),
                                &mut compactions,
                                &semaphore,
                            )
                            .await?;
                            compact_and_seal(&storage, events.as_ref())?;
                        }
                        break;
                    }
                    None => {
                        if mode == DbWriteMode::FastSync {
                            flush_fast_sync_buffer(
                                &storage,
                                &mut buffer,
                                bench.as_ref(),
                                events.as_ref(),
                                remaining_per_shard.as_ref(),
                                &mut compactions,
                                &semaphore,
                            )
                            .await?;
                            compact_and_seal(&storage, events.as_ref())?;
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
                if mode == DbWriteMode::FastSync {
                    flush_fast_sync_buffer(
                        &storage,
                        &mut buffer,
                        bench.as_ref(),
                        events.as_ref(),
                        remaining_per_shard.as_ref(),
                        &mut compactions,
                        &semaphore,
                    )
                    .await?;
                    compact_and_seal(&storage, events.as_ref())?;
                }
            }
            _ = gauge_tick.tick() => {
                if let Some(events) = events.as_ref() {
                    let compactions_inflight = compactions
                        .iter()
                        .filter(|handle| !handle.is_finished())
                        .count() as u64;
                    events.record(BenchEvent::DbWriterGaugeSample {
                        buffer_len: buffer.len() as u64,
                        compactions_total: compactions.len() as u64,
                        compactions_inflight,
                    });
                }
            }
        }
    }

    for handle in compactions {
        handle.await??;
    }

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
            shard_size: crate::cli::DEFAULT_SHARD_SIZE,
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

    #[tokio::test]
    async fn db_writer_writes_out_of_order_blocks() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Arc::new(Storage::open(&config).expect("open storage"));

        let (tx, rx) = mpsc::channel(8);
        let handle = tokio::spawn(run_db_writer(
            Arc::clone(&storage),
            rx,
            DbWriteConfig::new(1, None),
            None,
            None,
            DbWriteMode::FastSync,
            None,
        ));

        tx.send(DbWriterMessage::Block(bundle_with_number(2)))
            .await
            .expect("send block 2");
        tx.send(DbWriterMessage::Block(bundle_with_number(0)))
            .await
            .expect("send block 0");
        tx.send(DbWriterMessage::Finalize)
            .await
            .expect("finalize");
        drop(tx);

        handle.await.expect("db writer").expect("db writer result");

        assert!(storage.block_header(0).expect("header 0").is_some());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert!(storage.block_header(2).expect("header 2").is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
