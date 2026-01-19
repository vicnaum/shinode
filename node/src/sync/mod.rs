//! Sync and ingest orchestration.

use crate::{
    chain::{ChainError, ChainTracker, ChainUpdate, HeadTracker, HeaderStub},
    storage::{Storage, StoredLog, StoredLogs, StoredReceipts, StoredTxHashes},
};
use async_trait::async_trait;
use alloy_primitives::{Address, B256, Bytes, Log};
use eyre::Result;
use reth_ethereum_primitives::Receipt;
use reth_primitives_traits::{Header, SealedHeader};
use std::ops::RangeInclusive;
use tokio::task::JoinSet;
use tokio::sync::Semaphore;

/// Plans contiguous ranges to fetch from `start_block..=head`.
#[derive(Debug, Clone)]
pub struct RangeSyncPlanner {
    next: u64,
    head: u64,
    batch_size: u64,
}

impl RangeSyncPlanner {
    /// Create a new planner for the inclusive range `start_block..=head`.
    pub fn new(start_block: u64, head: u64, batch_size: u64) -> Self {
        Self {
            next: start_block,
            head,
            batch_size: batch_size.max(1),
        }
    }

    /// Returns the next inclusive batch to process, or `None` when complete.
    pub fn next_batch(&mut self) -> Option<std::ops::RangeInclusive<u64>> {
        if self.next > self.head {
            return None;
        }

        let batch_end = self
            .next
            .saturating_add(self.batch_size - 1)
            .min(self.head);
        let range = self.next..=batch_end;
        self.next = batch_end.saturating_add(1);
        Some(range)
    }
}

/// Sync controller that plans ranges and advances checkpoints.
pub struct SyncController<H> {
    head_tracker: H,
    batch_size: u64,
}

impl<H> SyncController<H>
where
    H: HeadTracker,
{
    pub fn new(head_tracker: H, batch_size: u64) -> Self {
        Self {
            head_tracker,
            batch_size: batch_size.max(1),
        }
    }

    /// Plans the next range to sync based on storage checkpoints.
    pub fn next_range(&self, storage: &Storage, start_block: u64) -> Result<Option<RangeInclusive<u64>>> {
        let head = match self.head_tracker.head() {
            Some(head) => head,
            None => return Ok(None),
        };

        let next = match storage.last_indexed_block()? {
            Some(last) => last.saturating_add(1).max(start_block),
            None => start_block,
        };

        if next > head {
            return Ok(None);
        }

        let mut planner = RangeSyncPlanner::new(next, head, self.batch_size);
        Ok(planner.next_batch())
    }

    /// Marks a range as fully synced by persisting its end block.
    #[allow(dead_code)]
    pub fn mark_range_complete(&self, storage: &Storage, range: RangeInclusive<u64>) -> Result<()> {
        if let Some(end) = range.last() {
            storage.set_last_indexed_block(end)?;
        }
        Ok(())
    }
}

/// Source of block headers for sync.
#[async_trait]
pub trait BlockSource: Send + Sync {
    async fn head(&self) -> Result<u64>;
    async fn headers_by_number(&self, range: RangeInclusive<u64>) -> Result<Vec<HeaderStub>>;
}

/// Result of a sync runner step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    UpToDate { head: u64 },
    RangeApplied { range: RangeInclusive<u64> },
    Reorg { ancestor_number: u64 },
}

/// Sync runner that applies headers to the canonical chain and updates checkpoints.
#[allow(dead_code)]
pub struct SyncRunner<S> {
    source: S,
    chain: ChainTracker,
    batch_size: u64,
}

#[allow(dead_code)]
impl<S> SyncRunner<S>
where
    S: BlockSource,
{
    pub fn new(source: S, batch_size: u64) -> Self {
        Self {
            source,
            chain: ChainTracker::new(),
            batch_size: batch_size.max(1),
        }
    }

    pub async fn run_once(&mut self, storage: &Storage, start_block: u64) -> Result<SyncOutcome> {
        let head = self.source.head().await?;
        storage.set_head_seen(head)?;

        let controller = SyncController::new(crate::chain::MemoryHeadTracker::new(Some(head)), self.batch_size);
        let range = match controller.next_range(storage, start_block)? {
            Some(range) => range,
            None => return Ok(SyncOutcome::UpToDate { head }),
        };

        let headers = self.source.headers_by_number(range.clone()).await?;
        if headers.is_empty() {
            return Ok(SyncOutcome::UpToDate { head });
        }

        for header in headers {
            match self.chain.insert_header(header) {
                Ok(ChainUpdate::Reorg { ancestor_number, .. }) => {
                    storage.set_last_indexed_block(ancestor_number)?;
                    return Ok(SyncOutcome::Reorg { ancestor_number });
                }
                Ok(_) => {}
                Err(ChainError::UnknownParent(parent)) => {
                    return Err(eyre::eyre!("unknown parent {parent}"));
                }
                Err(ChainError::NonContiguousNumber { expected, got }) => {
                    return Err(eyre::eyre!(
                        "non-contiguous header number: expected {expected}, got {got}"
                    ));
                }
            }
        }

        controller.mark_range_complete(storage, range.clone())?;
        Ok(SyncOutcome::RangeApplied { range })
    }
}

/// Full payload for a block: header, tx hashes, receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPayload {
    pub header: Header,
    pub tx_hashes: Vec<B256>,
    pub receipts: Vec<Receipt>,
}

/// Log derived from receipts with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedLog {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
    pub block_number: u64,
    pub block_hash: B256,
    pub transaction_hash: B256,
    pub transaction_index: u64,
    pub log_index: u64,
    pub removed: bool,
}

/// Source of block payloads for ingestion.
#[async_trait]
pub trait BlockPayloadSource: Send + Sync {
    async fn head(&self) -> Result<u64>;
    async fn blocks_by_number(&self, range: RangeInclusive<u64>) -> Result<Vec<BlockPayload>>;
}

/// Outcome from a single ingest step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngestOutcome {
    UpToDate { head: u64 },
    RangeApplied { range: RangeInclusive<u64>, logs: Vec<DerivedLog> },
    Reorg { ancestor_number: u64 },
}

/// Ingest runner that derives logs and advances checkpoints.
#[allow(dead_code)]
pub struct IngestRunner<S> {
    source: S,
    chain: ChainTracker,
    batch_size: u64,
}

#[allow(dead_code)]
impl<S> IngestRunner<S>
where
    S: BlockPayloadSource,
{
    pub fn new(source: S, batch_size: u64) -> Self {
        Self {
            source,
            chain: ChainTracker::new(),
            batch_size: batch_size.max(1),
        }
    }

    pub async fn run_once(&mut self, storage: &Storage, start_block: u64) -> Result<IngestOutcome> {
        let head = self.source.head().await?;
        storage.set_head_seen(head)?;

        let controller = SyncController::new(crate::chain::MemoryHeadTracker::new(Some(head)), self.batch_size);
        let range = match controller.next_range(storage, start_block)? {
            Some(range) => range,
            None => return Ok(IngestOutcome::UpToDate { head }),
        };

        let mut payloads = self.source.blocks_by_number(range.clone()).await?;
        if payloads.is_empty() {
            return Ok(IngestOutcome::UpToDate { head });
        }
        payloads.sort_by_key(|payload| payload.header.number);

        let mut derived_logs = Vec::new();

        for payload in payloads {
            let BlockPayload {
                header,
                tx_hashes,
                receipts,
            } = payload;
            let header_hash = SealedHeader::seal_slow(header.clone()).hash();
            let header_stub = HeaderStub {
                number: header.number,
                hash: header_hash,
                parent_hash: header.parent_hash,
            };
            match self.chain.insert_header(header_stub) {
                Ok(ChainUpdate::Reorg { ancestor_number, .. }) => {
                    storage.set_last_indexed_block(ancestor_number)?;
                    return Ok(IngestOutcome::Reorg { ancestor_number });
                }
                Ok(_) => {}
                Err(ChainError::UnknownParent(parent)) => {
                    return Err(eyre::eyre!("unknown parent {parent}"));
                }
                Err(ChainError::NonContiguousNumber { expected, got }) => {
                    return Err(eyre::eyre!(
                        "non-contiguous header number: expected {expected}, got {got}"
                    ));
                }
            }

            if tx_hashes.len() != receipts.len() {
                return Err(eyre::eyre!(
                    "tx hash count {} does not match receipts count {} for block {}",
                    tx_hashes.len(),
                    receipts.len(),
                    header.number
                ));
            }

            storage.write_block_header(header.number, header.clone())?;

            let mut block_logs = Vec::new();
            for (tx_index, (tx_hash, receipt)) in tx_hashes
                .iter()
                .zip(receipts.iter())
                .enumerate()
            {
                for (log_index, log) in receipt.logs.iter().cloned().enumerate() {
                    let Log { address, data } = log;
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
                    derived_logs.push(DerivedLog {
                        address: stored_log.address,
                        topics: stored_log.topics.clone(),
                        data: stored_log.data.clone(),
                        block_number: stored_log.block_number,
                        block_hash: stored_log.block_hash,
                        transaction_hash: stored_log.transaction_hash,
                        transaction_index: stored_log.transaction_index,
                        log_index: stored_log.log_index,
                        removed: stored_log.removed,
                    });
                    block_logs.push(stored_log);
                }
            }

            storage.write_block_tx_hashes(
                header.number,
                StoredTxHashes {
                    hashes: tx_hashes.clone(),
                },
            )?;
            storage.write_block_receipts(
                header.number,
                StoredReceipts {
                    receipts: receipts,
                },
            )?;
            storage.write_block_logs(header.number, StoredLogs { logs: block_logs })?;
            storage.set_last_indexed_block(header.number)?;
        }

        Ok(IngestOutcome::RangeApplied { range, logs: derived_logs })
    }
}

/// Process ranges concurrently with a max concurrency limit.
#[allow(dead_code)]
pub async fn process_ranges_concurrently<F, Fut>(
    ranges: Vec<RangeInclusive<u64>>,
    max_concurrency: usize,
    handler: F,
) -> Result<()>
where
    F: Fn(RangeInclusive<u64>) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send + 'static,
{
    let semaphore = std::sync::Arc::new(Semaphore::new(max_concurrency.max(1)));
    let mut join_set = JoinSet::new();

    for range in ranges {
        let permit = semaphore.clone().acquire_owned().await?;
        let range = range.clone();
        let handler = handler.clone();
        join_set.spawn(async move {
            let _permit = permit;
            handler(range).await
        });
    }

    while let Some(result) = join_set.join_next().await {
        result??;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BlockPayload, BlockPayloadSource, BlockSource, DerivedLog, IngestOutcome, IngestRunner,
        RangeSyncPlanner, SyncController, SyncOutcome, SyncRunner, process_ranges_concurrently,
    };
    use async_trait::async_trait;
    use crate::{
        chain::{HeaderStub, MemoryHeadTracker},
        cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode},
        storage::Storage,
    };
    use alloy_primitives::{Address, B256, Bytes, Log};
    use eyre::Result;
    use reth_ethereum_primitives::{Receipt, TxType};
    use reth_primitives_traits::{Header, SealedHeader};
    use std::ops::RangeInclusive;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn single_block_range() {
        let mut planner = RangeSyncPlanner::new(5, 5, 10);
        let batch = planner.next_batch().expect("batch");
        assert_eq!(batch.collect::<Vec<_>>(), vec![5]);
        assert!(planner.next_batch().is_none());
    }

    #[test]
    fn batches_are_contiguous() {
        let mut planner = RangeSyncPlanner::new(0, 9, 4);
        let first = planner.next_batch().expect("first");
        let second = planner.next_batch().expect("second");
        let third = planner.next_batch().expect("third");
        assert!(planner.next_batch().is_none());

        assert_eq!(first.collect::<Vec<_>>(), vec![0, 1, 2, 3]);
        assert_eq!(second.collect::<Vec<_>>(), vec![4, 5, 6, 7]);
        assert_eq!(third.collect::<Vec<_>>(), vec![8, 9]);
    }

    fn temp_dir() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("stateless-history-node-sync-test-{now}-{}", std::process::id()));
        path
    }

    fn base_config(data_dir: PathBuf) -> NodeConfig {
        NodeConfig {
            chain_id: 1,
            data_dir,
            rpc_bind: "127.0.0.1:0".parse().expect("valid bind"),
            start_block: 0,
            rollback_window: 64,
            retention_mode: RetentionMode::Full,
            head_source: HeadSource::P2p,
            reorg_strategy: ReorgStrategy::Delete,
        }
    }

    #[test]
    fn sync_controller_plans_from_checkpoint() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let tracker = MemoryHeadTracker::new(Some(10));
        let controller = SyncController::new(tracker, 4);
        let range = controller
            .next_range(&storage, config.start_block)
            .expect("range")
            .expect("range exists");
        assert_eq!(range.collect::<Vec<_>>(), vec![0, 1, 2, 3]);

        controller
            .mark_range_complete(&storage, 0..=3)
            .expect("mark complete");

        let next = controller
            .next_range(&storage, config.start_block)
            .expect("next range")
            .expect("next exists");
        assert_eq!(next.collect::<Vec<_>>(), vec![4, 5, 6, 7]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[derive(Clone)]
    struct VecBlockSource {
        head: u64,
        headers: Vec<HeaderStub>,
    }

    #[async_trait]
    impl BlockSource for VecBlockSource {
        async fn head(&self) -> Result<u64> {
            Ok(self.head)
        }

        async fn headers_by_number(&self, range: RangeInclusive<u64>) -> Result<Vec<HeaderStub>> {
            let start = *range.start();
            let end = *range.end();
            let mut headers = self
                .headers
                .iter()
                .filter(|h| h.number >= start && h.number <= end)
                .copied()
                .collect::<Vec<_>>();
            headers.sort_by_key(|h| h.number);
            Ok(headers)
        }
    }

    fn hash_from_u64(value: u64) -> B256 {
        let mut bytes = [0u8; 32];
        bytes[24..].copy_from_slice(&value.to_be_bytes());
        B256::from(bytes)
    }

    fn address_from_u64(value: u64) -> Address {
        let mut bytes = [0u8; 20];
        bytes[12..].copy_from_slice(&value.to_be_bytes());
        Address::from_slice(&bytes)
    }

    fn linear_headers(start: u64, end: u64, base_hash: u64) -> Vec<HeaderStub> {
        let mut headers = Vec::new();
        let mut prev_hash = hash_from_u64(base_hash);
        for number in start..=end {
            let hash = hash_from_u64(base_hash + number + 1);
            let header = HeaderStub {
                number,
                hash,
                parent_hash: prev_hash,
            };
            headers.push(header);
            prev_hash = hash;
        }
        headers
    }

    #[tokio::test]
    async fn sync_runner_advances_checkpoints() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let headers = linear_headers(0, 5, 100);
        let source = VecBlockSource { head: 5, headers };
        let mut runner = SyncRunner::new(source, 3);

        let outcome = runner
            .run_once(&storage, config.start_block)
            .await
            .expect("run once");
        assert_eq!(
            outcome,
            SyncOutcome::RangeApplied { range: 0..=2 }
        );
        assert_eq!(storage.last_indexed_block().unwrap(), Some(2));

        let outcome = runner
            .run_once(&storage, config.start_block)
            .await
            .expect("run second");
        assert_eq!(
            outcome,
            SyncOutcome::RangeApplied { range: 3..=5 }
        );
        assert_eq!(storage.last_indexed_block().unwrap(), Some(5));

        let outcome = runner
            .run_once(&storage, config.start_block)
            .await
            .expect("run third");
        assert_eq!(outcome, SyncOutcome::UpToDate { head: 5 });

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sync_runner_handles_reorg_by_rolling_back_checkpoint() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let headers = linear_headers(0, 2, 200);
        let source = VecBlockSource { head: 2, headers: headers.clone() };
        let mut runner = SyncRunner::new(source, 3);
        runner
            .run_once(&storage, config.start_block)
            .await
            .expect("initial sync");
        assert_eq!(storage.last_indexed_block().unwrap(), Some(2));

        storage.set_last_indexed_block(1).expect("rewind");
        let alt_header = HeaderStub {
            number: 2,
            hash: hash_from_u64(999),
            parent_hash: headers[1].hash,
        };
        let source = VecBlockSource { head: 2, headers: vec![alt_header] };
        runner.source = source;
        let outcome = runner
            .run_once(&storage, config.start_block)
            .await
            .expect("reorg run");
        assert_eq!(outcome, SyncOutcome::Reorg { ancestor_number: 1 });
        assert_eq!(storage.last_indexed_block().unwrap(), Some(1));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[derive(Clone)]
    struct VecPayloadSource {
        head: u64,
        blocks: Vec<BlockPayload>,
    }

    #[async_trait]
    impl BlockPayloadSource for VecPayloadSource {
        async fn head(&self) -> Result<u64> {
            Ok(self.head)
        }

        async fn blocks_by_number(&self, range: RangeInclusive<u64>) -> Result<Vec<BlockPayload>> {
            let start = *range.start();
            let end = *range.end();
            let mut blocks = self
                .blocks
                .iter()
                .filter(|block| block.header.number >= start && block.header.number <= end)
                .cloned()
                .collect::<Vec<_>>();
            blocks.sort_by_key(|block| block.header.number);
            Ok(blocks)
        }
    }

    fn receipt_with_logs(logs: Vec<Log>) -> Receipt {
        Receipt {
            tx_type: TxType::Legacy,
            success: true,
            cumulative_gas_used: 0,
            logs,
        }
    }

    fn payload_for_block(
        header: Header,
        tx_hashes: Vec<B256>,
        logs_per_tx: Vec<Vec<Log>>,
    ) -> BlockPayload {
        let receipts = logs_per_tx
            .into_iter()
            .map(receipt_with_logs)
            .collect::<Vec<_>>();
        BlockPayload {
            header,
            tx_hashes,
            receipts,
        }
    }

    fn header_with_number(number: u64, parent_hash: B256) -> Header {
        let mut header = Header::default();
        header.number = number;
        header.parent_hash = parent_hash;
        header
    }

    fn header_hash(header: &Header) -> B256 {
        SealedHeader::seal_slow(header.clone()).hash()
    }

    #[tokio::test]
    async fn ingest_runner_derives_log_metadata() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let header0 = header_with_number(0, B256::ZERO);
        let header0_hash = header_hash(&header0);
        let header1 = header_with_number(1, header0_hash);
        let header1_hash = header_hash(&header1);
        let block0 = payload_for_block(
            header0.clone(),
            vec![hash_from_u64(11), hash_from_u64(12)],
            vec![
                vec![Log::new_unchecked(
                    address_from_u64(1),
                    vec![hash_from_u64(100)],
                    Bytes::from(vec![0x01]),
                )],
                vec![Log::new_unchecked(
                    address_from_u64(2),
                    vec![hash_from_u64(200), hash_from_u64(201)],
                    Bytes::from(vec![0x02]),
                )],
            ],
        );
        let block1 = payload_for_block(
            header1.clone(),
            vec![hash_from_u64(13)],
            vec![vec![
                Log::new_unchecked(
                    address_from_u64(3),
                    vec![hash_from_u64(300)],
                    Bytes::from(vec![0x03]),
                ),
                Log::new_unchecked(
                    address_from_u64(4),
                    vec![],
                    Bytes::from(vec![0x04]),
                ),
            ]],
        );

        let source = VecPayloadSource {
            head: 1,
            blocks: vec![block0, block1],
        };
        let mut runner = IngestRunner::new(source, 10);
        let outcome = runner
            .run_once(&storage, config.start_block)
            .await
            .expect("ingest");

        let logs = match outcome {
            IngestOutcome::RangeApplied { logs, .. } => logs,
            other => panic!("unexpected outcome: {other:?}"),
        };

        let expected = vec![
            DerivedLog {
                address: address_from_u64(1),
                topics: vec![hash_from_u64(100)],
                data: Bytes::from(vec![0x01]),
                block_number: 0,
                block_hash: header0_hash,
                transaction_hash: hash_from_u64(11),
                transaction_index: 0,
                log_index: 0,
                removed: false,
            },
            DerivedLog {
                address: address_from_u64(2),
                topics: vec![hash_from_u64(200), hash_from_u64(201)],
                data: Bytes::from(vec![0x02]),
                block_number: 0,
                block_hash: header0_hash,
                transaction_hash: hash_from_u64(12),
                transaction_index: 1,
                log_index: 0,
                removed: false,
            },
            DerivedLog {
                address: address_from_u64(3),
                topics: vec![hash_from_u64(300)],
                data: Bytes::from(vec![0x03]),
                block_number: 1,
                block_hash: header1_hash,
                transaction_hash: hash_from_u64(13),
                transaction_index: 0,
                log_index: 0,
                removed: false,
            },
            DerivedLog {
                address: address_from_u64(4),
                topics: vec![],
                data: Bytes::from(vec![0x04]),
                block_number: 1,
                block_hash: header1_hash,
                transaction_hash: hash_from_u64(13),
                transaction_index: 0,
                log_index: 1,
                removed: false,
            },
        ];

        assert_eq!(logs, expected);
        assert_eq!(
            storage
                .block_header(0)
                .expect("block header")
                .expect("header exists")
                .number,
            0
        );
        assert_eq!(
            storage
                .block_tx_hashes(0)
                .expect("tx hashes")
                .expect("tx hashes exist")
                .hashes,
            vec![hash_from_u64(11), hash_from_u64(12)]
        );
        assert_eq!(
            storage
                .block_receipts(0)
                .expect("receipts")
                .expect("receipts exist")
                .receipts
                .len(),
            2
        );
        assert_eq!(
            storage
                .block_logs(0)
                .expect("logs")
                .expect("logs exist")
                .logs
                .len(),
            2
        );
        assert_eq!(storage.last_indexed_block().unwrap(), Some(1));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn ingest_runner_reorg_rolls_back_checkpoint() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let header0 = header_with_number(0, B256::ZERO);
        let header0_hash = header_hash(&header0);
        let header1 = header_with_number(1, header0_hash);
        let header1_hash = header_hash(&header1);
        let header2 = header_with_number(2, header1_hash);
        let headers = vec![header0.clone(), header1.clone(), header2.clone()];
        let blocks = headers
            .iter()
            .map(|header| {
                payload_for_block(
                    header.clone(),
                    vec![header_hash(header)],
                    vec![vec![Log::new_unchecked(
                        address_from_u64(header.number),
                        vec![],
                        Bytes::from(vec![0x01]),
                    )]],
                )
            })
            .collect::<Vec<_>>();
        let source = VecPayloadSource { head: 2, blocks };
        let mut runner = IngestRunner::new(source, 10);
        runner
            .run_once(&storage, config.start_block)
            .await
            .expect("initial ingest");
        assert_eq!(storage.last_indexed_block().unwrap(), Some(2));

        storage.set_last_indexed_block(1).expect("rewind");
        let alt_header = header_with_number(2, header_hash(&header1));
        let alt_block = payload_for_block(
            alt_header,
            vec![hash_from_u64(888)],
            vec![vec![Log::new_unchecked(
                address_from_u64(9),
                vec![],
                Bytes::from(vec![0x09]),
            )]],
        );

        runner.source = VecPayloadSource {
            head: 2,
            blocks: vec![alt_block],
        };
        let outcome = runner
            .run_once(&storage, config.start_block)
            .await
            .expect("reorg ingest");
        assert_eq!(outcome, IngestOutcome::Reorg { ancestor_number: 1 });
        assert_eq!(storage.last_indexed_block().unwrap(), Some(1));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn range_processing_respects_concurrency() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        use tokio::time::{sleep, Duration};

        let ranges = vec![0..=0, 1..=1, 2..=2, 3..=3];
        let max_concurrency = 2;
        let current = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));

        process_ranges_concurrently(ranges, max_concurrency, {
            let current = current.clone();
            let peak = peak.clone();
            move |_range| {
                let current = current.clone();
                let peak = peak.clone();
                async move {
                    let now = current.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    sleep(Duration::from_millis(10)).await;
                    current.fetch_sub(1, Ordering::SeqCst);
                    Ok(())
                }
            }
        })
        .await
        .expect("process ranges");

        assert!(peak.load(Ordering::SeqCst) <= max_concurrency);
    }
}
