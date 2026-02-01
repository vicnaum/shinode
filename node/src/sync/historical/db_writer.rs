//! Batched DB writer for ingest mode.

use crate::storage::{BlockBundle, Storage};
use crate::sync::historical::stats::{
    BenchEvent, BenchEventLogger, DbWriteByteTotals, IngestBenchStats,
};
use crate::sync::{FinalizePhase, SyncProgressStats};
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

/// Tracks remaining blocks per shard for compaction scheduling.
pub type ShardRemainingTracker = Arc<Mutex<HashMap<u64, usize>>>;

/// Full configuration for the DB writer task.
pub struct DbWriterParams {
    pub config: DbWriteConfig,
    pub mode: DbWriteMode,
    pub bench: Option<Arc<IngestBenchStats>>,
    pub events: Option<Arc<BenchEventLogger>>,
    pub progress_stats: Option<Arc<SyncProgressStats>>,
    pub remaining_per_shard: Option<ShardRemainingTracker>,
    pub follow_expected_start: Option<u64>,
}

/// Shared context for fast-sync flush operations (immutable parts).
struct FlushContext<'a> {
    storage: &'a Arc<Storage>,
    bench: Option<&'a Arc<IngestBenchStats>>,
    events: Option<&'a Arc<BenchEventLogger>>,
    remaining_per_shard: Option<&'a ShardRemainingTracker>,
    progress_stats: Option<&'a Arc<SyncProgressStats>>,
    semaphore: &'a Arc<Semaphore>,
}

async fn flush_fast_sync_buffer(
    ctx: &FlushContext<'_>,
    buffer: &mut Vec<BlockBundle>,
    compactions: &mut Vec<JoinHandle<Result<()>>>,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }

    let bytes = if ctx.bench.is_some() || ctx.events.is_some() {
        Some(db_bytes_for_bundles(buffer)?)
    } else {
        None
    };
    if let Some(events) = ctx.events.as_ref() {
        let bytes_total = bytes.as_ref().map_or(0, super::stats::DbWriteByteTotals::total);
        events.record(BenchEvent::DbFlushStart {
            blocks: buffer.len() as u64,
            bytes_total,
        });
    }

    let started = Instant::now();
    let written_blocks = ctx.storage.write_block_bundles_wal(buffer)?;
    if let Some(bench) = ctx.bench.as_ref() {
        bench.record_db_write(buffer.len() as u64, started.elapsed());
        if let Some(bytes) = bytes.as_ref() {
            bench.record_db_write_bytes(*bytes);
        }
    }
    if let Some(events) = ctx.events.as_ref() {
        let elapsed_ms = started.elapsed().as_millis() as u64;
        let bytes_total = bytes.as_ref().map_or(0, super::stats::DbWriteByteTotals::total);
        events.record(BenchEvent::DbFlushEnd {
            blocks: buffer.len() as u64,
            bytes_total,
            duration_ms: elapsed_ms,
        });
    }

    if let Some(remaining) = ctx.remaining_per_shard.as_ref() {
        let mut shards_to_compact = Vec::new();
        let mut guard = remaining.lock().await;
        for number in written_blocks {
            let shard_start = (number / ctx.storage.shard_size()) * ctx.storage.shard_size();
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
            let storage = Arc::clone(ctx.storage);
            let semaphore = Arc::clone(ctx.semaphore);
            let events = ctx.events.cloned();
            let progress_stats = ctx.progress_stats.cloned();
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
                // Note: We don't increment compactions_done here because sync-phase
                // compactions are background work without a known total. The finalize
                // phase tracks its own compactions with proper total/done counters.
                // Refresh storage byte stats after inline compaction
                // (DB counters are updated incrementally at processing time)
                if result.is_ok() {
                    if let Some(stats) = progress_stats.as_ref() {
                        let agg = storage.aggregate_stats();
                        stats.set_db_shards(agg.total_shards, agg.compacted_shards);
                        stats.set_storage_bytes(
                            agg.disk_bytes_headers,
                            agg.disk_bytes_transactions,
                            agg.disk_bytes_receipts,
                            agg.disk_bytes_total,
                        );
                    }
                }
                result
            }));
        }
    }

    buffer.clear();
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

#[expect(clippy::large_enum_variant, reason = "Block variant is the hot path, boxing adds overhead")]
pub enum DbWriterMessage {
    Block(BlockBundle),
    Finalize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbWriteMode {
    FastSync,
    Follow,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DbWriterFinalizeStats {
    pub compactions_wait_ms: u64,
    pub compact_all_dirty_ms: u64,
    pub seal_completed_ms: u64,
}

#[expect(
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    reason = "async select loop with mode branching and finalization logic"
)]
pub async fn run_db_writer(
    storage: Arc<Storage>,
    rx: mpsc::Receiver<DbWriterMessage>,
    params: DbWriterParams,
) -> Result<DbWriterFinalizeStats> {
    let DbWriterParams {
        config,
        mode,
        bench,
        events,
        progress_stats,
        remaining_per_shard,
        follow_expected_start,
    } = params;
    let mut rx = rx;
    let mut interval = config.flush_interval.map(tokio::time::interval);
    // Compaction is a large, bursty workload (reads WAL + writes new segment files).
    // Serializing it avoids compaction fan-out doubling peak memory/IO.
    let semaphore = Arc::new(Semaphore::new(1));
    let mut compactions = Vec::new();
    let mut buffer: Vec<BlockBundle> = Vec::new();
    let mut gauge_tick = tokio::time::interval(Duration::from_secs(10));
    let mut follow_expected = if mode == DbWriteMode::Follow {
        follow_expected_start
            .or_else(|| storage.last_indexed_block().ok().flatten().map(|v| v + 1))
            .unwrap_or(0)
    } else {
        0
    };
    let mut follow_buffer: std::collections::BTreeMap<u64, BlockBundle> =
        std::collections::BTreeMap::new();

    tracing::debug!(
        mode = ?mode,
        batch_blocks = config.batch_blocks,
        "db writer started"
    );

    let write_follow_bundle = |bundle: BlockBundle| -> Result<()> {
        let bytes = if bench.is_some() || events.is_some() {
            Some(db_bytes_for_bundles(std::slice::from_ref(&bundle))?)
        } else {
            None
        };
        if let Some(events) = events.as_ref() {
            let bytes_total = bytes.as_ref().map_or(0, super::stats::DbWriteByteTotals::total);
            events.record(BenchEvent::DbFlushStart {
                blocks: 1,
                bytes_total,
            });
        }
        let started = Instant::now();
        storage.write_block_bundle_follow(&bundle)?;
        if let Some(bench) = bench.as_ref() {
            bench.record_db_write(1, started.elapsed());
            if let Some(bytes) = bytes.as_ref() {
                bench.record_db_write_bytes(*bytes);
            }
        }
        if let Some(events) = events.as_ref() {
            let elapsed_ms = started.elapsed().as_millis() as u64;
            let bytes_total = bytes.as_ref().map_or(0, super::stats::DbWriteByteTotals::total);
            events.record(BenchEvent::DbFlushEnd {
                blocks: 1,
                bytes_total,
                duration_ms: elapsed_ms,
            });
        }
        // Refresh storage byte + shard stats after follow-mode write
        // (DB counters are updated incrementally at processing time)
        if let Some(stats) = progress_stats.as_ref() {
            let agg = storage.aggregate_stats();
            stats.set_db_shards(agg.total_shards, agg.compacted_shards);
            stats.set_storage_bytes(
                agg.disk_bytes_headers,
                agg.disk_bytes_transactions,
                agg.disk_bytes_receipts,
                agg.disk_bytes_total,
            );
        }
        Ok(())
    };

    let flush_ctx = FlushContext {
        storage: &storage,
        bench: bench.as_ref(),
        events: events.as_ref(),
        remaining_per_shard: remaining_per_shard.as_ref(),
        progress_stats: progress_stats.as_ref(),
        semaphore: &semaphore,
    };

    loop {
        tokio::select! {
            maybe_msg = rx.recv() => {
                match maybe_msg {
                    Some(DbWriterMessage::Block(block)) => {
                        if mode == DbWriteMode::FastSync {
                            buffer.push(block);
                            if buffer.len() >= config.batch_blocks {
                                flush_fast_sync_buffer(&flush_ctx, &mut buffer, &mut compactions).await?;
                            }
                        } else {
                            let number = block.number;
                            if number < follow_expected {
                                continue;
                            }
                            follow_buffer.insert(number, block);
                            while let Some(bundle) = follow_buffer.remove(&follow_expected) {
                                write_follow_bundle(bundle)?;
                                follow_expected = follow_expected.saturating_add(1);
                            }
                        }
                    }
                    Some(DbWriterMessage::Finalize) | None => {
                        if mode == DbWriteMode::FastSync {
                            flush_fast_sync_buffer(&flush_ctx, &mut buffer, &mut compactions).await?;
                        } else {
                            while let Some(bundle) = follow_buffer.remove(&follow_expected) {
                                write_follow_bundle(bundle)?;
                                follow_expected = follow_expected.saturating_add(1);
                            }
                        }
                        break;
                    }
                }
            }
            () = async {
                if let Some(interval) = interval.as_mut() {
                    interval.tick().await;
                }
            }, if interval.is_some() => {
                if mode == DbWriteMode::FastSync {
                    flush_fast_sync_buffer(&flush_ctx, &mut buffer, &mut compactions).await?;
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
                // Periodically refresh shard counts for TUI
                // (covers deferred-compaction mode where inline compaction never fires)
                if let Some(stats) = progress_stats.as_ref() {
                    let agg = storage.aggregate_stats();
                    stats.set_db_shards(agg.total_shards, agg.compacted_shards);
                }
                if mode == DbWriteMode::Follow && !follow_buffer.is_empty() {
                    let min_key = follow_buffer.keys().next().copied().unwrap_or(0);
                    let max_key = follow_buffer.keys().next_back().copied().unwrap_or(0);
                    let max_gap = max_key.saturating_sub(follow_expected);
                    tracing::debug!(
                        expected = follow_expected,
                        buffer_len = follow_buffer.len(),
                        min_key,
                        max_key,
                        max_gap,
                        "follow reorder buffer"
                    );
                }
            }
        }
    }

    if mode == DbWriteMode::Follow && !follow_buffer.is_empty() {
        tracing::warn!(
            buffer_len = follow_buffer.len(),
            expected = follow_expected,
            "follow finalize: reorder buffer not empty after flush"
        );
    }

    // Wait for in-flight compactions
    let compaction_started = Instant::now();
    let mut finalize_stats = DbWriterFinalizeStats::default();
    for handle in compactions {
        handle.await??;
    }
    finalize_stats.compactions_wait_ms = compaction_started.elapsed().as_millis() as u64;

    // Finalize fast-sync: compact dirty shards and seal completed ones
    if mode == DbWriteMode::FastSync {
        finalize_fast_sync(
            &storage,
            events.as_ref(),
            progress_stats.as_ref(),
            &mut finalize_stats,
        )?;
    }

    Ok(finalize_stats)
}

/// Finalize fast-sync mode: compact all dirty shards and seal completed ones.
#[expect(clippy::cognitive_complexity, reason = "finalization with compaction and sealing phases")]
fn finalize_fast_sync(
    storage: &Storage,
    events: Option<&Arc<BenchEventLogger>>,
    progress_stats: Option<&Arc<SyncProgressStats>>,
    finalize_stats: &mut DbWriterFinalizeStats,
) -> Result<()> {
    if let Some(events) = events {
        events.record(BenchEvent::CompactAllDirtyStart);
    }

    // Reset compaction progress counters for finalize phase
    if let Some(stats) = progress_stats {
        stats.set_compactions_done(0);
        stats.set_compactions_total(0);
        stats.set_finalize_phase(FinalizePhase::Compacting);
    }

    if let Ok(mut dirty) = storage.dirty_shards() {
        if !dirty.is_empty() {
            let total_wal_bytes: u64 = dirty.iter().map(|info| info.wal_bytes).sum();
            dirty.sort_by_key(|info| std::cmp::Reverse(info.wal_bytes));
            let top = dirty.iter().take(5).collect::<Vec<_>>();
            tracing::info!(
                dirty_shards = dirty.len(),
                total_wal_mib = (total_wal_bytes as f64 / (1024.0 * 1024.0)).round(),
                top_shards = ?top,
                "finalizing: compacting dirty shards (WAL present or unsorted)"
            );
            if let Some(stats) = progress_stats {
                stats.set_compactions_total(dirty.len() as u64);
            }
        }
    }

    let compact_started = Instant::now();
    let stats_for_callback = progress_stats.cloned();
    storage.compact_all_dirty_with_progress(|_shard_start| {
        if let Some(stats) = stats_for_callback.as_ref() {
            stats.inc_compactions_done(1);
        }
    })?;
    finalize_stats.compact_all_dirty_ms = compact_started.elapsed().as_millis() as u64;

    // Refresh storage byte + shard stats after compaction
    // (DB counters are updated incrementally at processing time)
    if let Some(stats) = progress_stats {
        let agg = storage.aggregate_stats();
        stats.set_db_shards(agg.total_shards, agg.compacted_shards);
        stats.set_storage_bytes(
            agg.disk_bytes_headers,
            agg.disk_bytes_transactions,
            agg.disk_bytes_receipts,
            agg.disk_bytes_total,
        );
    }

    if let Some(events) = events {
        events.record(BenchEvent::CompactAllDirtyEnd {
            duration_ms: compact_started.elapsed().as_millis() as u64,
        });
    }

    // Sealing phase - uses separate counters from compaction
    if let Some(events) = events {
        events.record(BenchEvent::SealCompletedStart);
    }
    let seal_started = Instant::now();

    if let Some(stats) = progress_stats {
        stats.set_sealings_done(0);
        let to_seal = storage.count_shards_to_seal().unwrap_or(0);
        stats.set_sealings_total(to_seal);
        stats.set_finalize_phase(FinalizePhase::Sealing);
    }

    let mut sealed_count = 0u64;
    let stats_for_seal = progress_stats.cloned();
    storage.seal_completed_shards_with_progress(|_shard_start| {
        sealed_count += 1;
        if let Some(stats) = stats_for_seal.as_ref() {
            stats.inc_sealings_done(1);
        }
    })?;
    if sealed_count > 0 {
        tracing::info!(sealed = sealed_count, "finalizing: sealed shards");
    }

    if let Some(events) = events {
        events.record(BenchEvent::SealCompletedEnd {
            duration_ms: seal_started.elapsed().as_millis() as u64,
        });
    }
    finalize_stats.seal_completed_ms = seal_started.elapsed().as_millis() as u64;

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
    use crate::storage::{StoredBlockSize, StoredReceipts, StoredTransactions, StoredTxHashes};
    use crate::test_utils::{base_config, temp_dir};
    use reth_ethereum_primitives::Receipt;

    fn header_with_number(number: u64) -> reth_primitives_traits::Header {
        let mut header = reth_primitives_traits::Header::default();
        header.number = number;
        header
    }

    fn bundle_with_number(number: u64) -> BlockBundle {
        BlockBundle {
            number,
            header: header_with_number(number),
            tx_hashes: StoredTxHashes { hashes: Vec::new() },
            transactions: StoredTransactions { txs: Vec::new() },
            size: StoredBlockSize { size: 0 },
            receipts: StoredReceipts {
                receipts: Vec::<Receipt>::new(),
            },
        }
    }

    #[tokio::test]
    async fn db_writer_writes_out_of_order_blocks() {
        let dir = temp_dir("db-writer");
        let config = base_config(dir.clone());
        let storage = Arc::new(Storage::open(&config).expect("open storage"));

        let (tx, rx) = mpsc::channel(8);
        let handle = tokio::spawn(run_db_writer(
            Arc::clone(&storage),
            rx,
            DbWriterParams {
                config: DbWriteConfig::new(1, None),
                mode: DbWriteMode::FastSync,
                bench: None,
                events: None,
                progress_stats: None,
                remaining_per_shard: None,
                follow_expected_start: None,
            },
        ));

        tx.send(DbWriterMessage::Block(bundle_with_number(2)))
            .await
            .expect("send block 2");
        tx.send(DbWriterMessage::Block(bundle_with_number(0)))
            .await
            .expect("send block 0");
        tx.send(DbWriterMessage::Finalize).await.expect("finalize");
        drop(tx);

        handle.await.expect("db writer").expect("db writer result");

        assert!(storage.block_header(0).expect("header 0").is_some());
        assert!(storage.block_header(1).expect("header 1").is_none());
        assert!(storage.block_header(2).expect("header 2").is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
