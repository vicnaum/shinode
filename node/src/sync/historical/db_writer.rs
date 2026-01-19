//! Batched DB writer for ingest mode.

use crate::storage::{BlockBundle, Storage};
use eyre::Result;
use std::sync::Arc;
use std::time::Duration;
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
                            flush_buffer(&storage, &mut buffer)?;
                        }
                    }
                    Some(DbWriterMessage::Flush) => {
                        if !buffer.is_empty() {
                            flush_buffer(&storage, &mut buffer)?;
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            flush_buffer(&storage, &mut buffer)?;
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
                    flush_buffer(&storage, &mut buffer)?;
                }
            }
        }
    }

    Ok(())
}

fn flush_buffer(storage: &Storage, buffer: &mut Vec<BlockBundle>) -> Result<()> {
    storage.write_block_bundle_batch(buffer)?;
    buffer.clear();
    Ok(())
}
