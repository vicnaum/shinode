//! Batched DB writer for ingest mode.

use crate::storage::{BlockBundle, Storage};
use crate::sync::historical::stats::{DbWriteByteTotals, IngestBenchStats};
use bytes::buf::UninitSlice;
use reth_db_api::table::{Compress, Encode};
use eyre::Result;
use std::mem::MaybeUninit;
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
) -> Result<()> {
    let mut buffer: Vec<BlockBundle> = Vec::with_capacity(config.batch_blocks);
    let mut interval = config.flush_interval.map(tokio::time::interval);

    loop {
        tokio::select! {
            maybe_msg = rx.recv() => {
                match maybe_msg {
                    Some(DbWriterMessage::Block(block)) => {
                        buffer.push(block);
                        if buffer.len() >= config.batch_blocks {
                            flush_buffer(&storage, &mut buffer, bench.as_ref())?;
                        }
                    }
                    Some(DbWriterMessage::Flush) => {
                        if !buffer.is_empty() {
                            flush_buffer(&storage, &mut buffer, bench.as_ref())?;
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            flush_buffer(&storage, &mut buffer, bench.as_ref())?;
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
                    flush_buffer(&storage, &mut buffer, bench.as_ref())?;
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
) -> Result<()> {
    let blocks = buffer.len() as u64;
    let started = Instant::now();
    storage.write_block_bundle_batch(buffer)?;
    if let Some(bench) = bench {
        bench.record_db_write(blocks, started.elapsed());
        let bytes = db_bytes_for_bundles(buffer);
        bench.record_db_write_bytes(bytes);
    }
    buffer.clear();
    Ok(())
}

const SIZE_COUNTER_SCRATCH: usize = 4096;

#[derive(Debug)]
struct SizeCounter {
    len: usize,
    scratch: [MaybeUninit<u8>; SIZE_COUNTER_SCRATCH],
}

impl Default for SizeCounter {
    fn default() -> Self {
        Self {
            len: 0,
            scratch: [MaybeUninit::uninit(); SIZE_COUNTER_SCRATCH],
        }
    }
}

impl SizeCounter {
    fn len(&self) -> u64 {
        self.len as u64
    }
}

impl AsMut<[u8]> for SizeCounter {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.scratch.as_mut_ptr() as *mut u8, self.scratch.len())
        }
    }
}

unsafe impl bytes::BufMut for SizeCounter {
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
        self.len = self.len.saturating_add(cnt);
    }
}

fn compressed_len<V: Compress>(value: &V) -> u64 {
    if let Some(bytes) = value.uncompressable_ref() {
        return bytes.len() as u64;
    }
    let mut counter = SizeCounter::default();
    value.compress_to_buf(&mut counter);
    counter.len()
}

fn encoded_len<K: Encode>(value: K) -> u64 {
    value.encode().as_ref().len() as u64
}

fn db_bytes_for_bundles(bundles: &[BlockBundle]) -> DbWriteByteTotals {
    let mut totals = DbWriteByteTotals::default();
    for bundle in bundles {
        let key_len = encoded_len(bundle.number);
        totals.headers = totals.headers.saturating_add(key_len + compressed_len(&bundle.header));
        totals.tx_hashes =
            totals.tx_hashes.saturating_add(key_len + compressed_len(&bundle.tx_hashes));
        totals.transactions = totals
            .transactions
            .saturating_add(key_len + compressed_len(&bundle.transactions));
        totals.withdrawals = totals
            .withdrawals
            .saturating_add(key_len + compressed_len(&bundle.withdrawals));
        totals.sizes = totals.sizes.saturating_add(key_len + compressed_len(&bundle.size));
        totals.receipts =
            totals.receipts.saturating_add(key_len + compressed_len(&bundle.receipts));
        totals.logs = totals.logs.saturating_add(key_len + compressed_len(&bundle.logs));
    }
    totals
}
