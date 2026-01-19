//! Sync and ingest orchestration.

use crate::{
    chain::{ChainError, ChainTracker, ChainUpdate, HeadTracker, HeaderStub},
    metrics::range_len,
    storage::{
        Storage, StoredBlockSize, StoredLog, StoredLogs, StoredReceipts, StoredTransaction,
        StoredTransactions, StoredTxHashes, StoredWithdrawal, StoredWithdrawals,
    },
};
use async_trait::async_trait;
use alloy_consensus::Transaction as _;
use alloy_primitives::{logs_bloom, Address, B256, Bytes, Log, TxKind};
use alloy_rlp::encode;
use eyre::Result;
use reth_ethereum_primitives::{Block, BlockBody, Receipt};
use reth_primitives_traits::{Header, SealedHeader, SignerRecoverable};
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

/// Full payload for a block: header, body, receipts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPayload {
    pub header: Header,
    pub body: BlockBody,
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

pub trait ProgressReporter {
    fn set_length(&self, len: u64);
    fn inc(&self, delta: u64);
    fn finish(&self);
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
        self.run_once_with_progress(storage, start_block, None).await
    }

    pub async fn run_once_with_progress(
        &mut self,
        storage: &Storage,
        start_block: u64,
        progress: Option<&dyn ProgressReporter>,
    ) -> Result<IngestOutcome> {
        let head = self.source.head().await?;
        storage.set_head_seen(head)?;

        let controller = SyncController::new(crate::chain::MemoryHeadTracker::new(Some(head)), self.batch_size);
        let range = match controller.next_range(storage, start_block)? {
            Some(range) => range,
            None => return Ok(IngestOutcome::UpToDate { head }),
        };

        if let Some(progress) = progress {
            progress.set_length(range_len(&range));
        }
        let mut payloads = self.source.blocks_by_number(range.clone()).await?;
        if payloads.is_empty() {
            return Ok(IngestOutcome::UpToDate { head });
        }
        payloads.sort_by_key(|payload| payload.header.number);

        let mut derived_logs = Vec::new();

        for payload in payloads {
            let BlockPayload {
                header,
                body,
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
                    storage.rollback_to(ancestor_number)?;
                    if let Some(progress) = progress {
                        progress.finish();
                    }
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

            let tx_hashes = body
                .transactions
                .iter()
                .map(|tx| *tx.hash())
                .collect::<Vec<_>>();
            if tx_hashes.len() != receipts.len() {
                return Err(eyre::eyre!(
                    "tx hash count {} does not match receipts count {} for block {}",
                    tx_hashes.len(),
                    receipts.len(),
                    header.number
                ));
            }

            let mut stored_transactions = Vec::with_capacity(body.transactions.len());
            for tx in &body.transactions {
                let from = tx
                    .recover_signer_unchecked()
                    .map_err(|err| eyre::eyre!("failed to recover signer: {err}"))?;
                let to = match tx.kind() {
                    TxKind::Call(address) => Some(address),
                    TxKind::Create => None,
                };
                stored_transactions.push(StoredTransaction {
                    hash: *tx.hash(),
                    from,
                    to,
                    value: tx.value(),
                    nonce: tx.nonce(),
                });
            }

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

            let block_size = block_rlp_size(&header, &body);

            let computed_bloom = logs_bloom(
                receipts
                    .iter()
                    .flat_map(|receipt| receipt.logs.iter().map(|log| log.as_ref())),
            );
            let mut stored_header = header.clone();
            stored_header.logs_bloom = computed_bloom;
            storage.write_block_header(header.number, stored_header)?;

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
            storage.write_block_transactions(
                header.number,
                StoredTransactions {
                    txs: stored_transactions,
                },
            )?;
            storage.write_block_withdrawals(header.number, stored_withdrawals)?;
            storage.write_block_size(header.number, StoredBlockSize { size: block_size })?;
            storage.write_block_receipts(
                header.number,
                StoredReceipts {
                    receipts: receipts,
                },
            )?;
            storage.write_block_logs(header.number, StoredLogs { logs: block_logs.clone() })?;
            storage.write_log_indexes(&block_logs)?;
            storage.set_last_indexed_block(header.number)?;

            if let Some(progress) = progress {
                progress.inc(1);
            }
        }

        if let Some(progress) = progress {
            progress.finish();
        }
        Ok(IngestOutcome::RangeApplied { range, logs: derived_logs })
    }
}

fn block_rlp_size(header: &Header, body: &BlockBody) -> u64 {
    let block = Block {
        header: header.clone(),
        body: body.clone(),
    };
    encode(&block).len() as u64
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
    use alloy_consensus::{Signed, TxLegacy};
    use alloy_primitives::{logs_bloom, Address, B256, Bytes, Log, Signature, TxKind, U256};
    use eyre::Result;
    use reth_ethereum_primitives::{BlockBody, Receipt, Transaction, TransactionSigned, TxType};
    use reth_primitives_traits::{Header, SealedHeader};
    use std::ops::RangeInclusive;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
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
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "stateless-history-node-sync-test-{now}-{}-{suffix}",
            std::process::id()
        ));
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
            verbosity: 0,
            rpc_max_request_body_bytes: crate::cli::DEFAULT_RPC_MAX_REQUEST_BODY_BYTES,
            rpc_max_response_body_bytes: crate::cli::DEFAULT_RPC_MAX_RESPONSE_BODY_BYTES,
            rpc_max_connections: crate::cli::DEFAULT_RPC_MAX_CONNECTIONS,
            rpc_max_batch_requests: crate::cli::DEFAULT_RPC_MAX_BATCH_REQUESTS,
            rpc_max_blocks_per_filter: crate::cli::DEFAULT_RPC_MAX_BLOCKS_PER_FILTER,
            rpc_max_logs_per_response: crate::cli::DEFAULT_RPC_MAX_LOGS_PER_RESPONSE,
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
        let transactions = tx_hashes
            .iter()
            .enumerate()
            .map(|(idx, hash)| signed_legacy_tx(*hash, idx as u64))
            .collect::<Vec<_>>();
        let body = BlockBody {
            transactions,
            ommers: Vec::new(),
            withdrawals: None,
        };
        BlockPayload {
            header,
            body,
            receipts,
        }
    }

    fn signed_legacy_tx(hash: B256, nonce: u64) -> TransactionSigned {
        let tx = Transaction::Legacy(TxLegacy {
            chain_id: Some(1),
            nonce,
            gas_price: 1,
            gas_limit: 21_000,
            to: TxKind::Call(Address::ZERO),
            value: U256::from(1),
            input: Bytes::new(),
        });
        Signed::new_unchecked(tx, Signature::test_signature(), hash).into()
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
                .block_transactions(0)
                .expect("txs")
                .expect("txs exist")
                .txs
                .len(),
            2
        );
        assert!(
            storage
                .block_size(0)
                .expect("size")
                .expect("size exists")
                .size
                > 0
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
        let stored_header = storage
            .block_header(0)
            .expect("block header")
            .expect("header exists");
        let stored_receipts = storage
            .block_receipts(0)
            .expect("receipts")
            .expect("receipts exist");
        let expected_bloom = logs_bloom(
            stored_receipts
                .receipts
                .iter()
                .flat_map(|receipt| receipt.logs.iter().map(|log| log.as_ref())),
        );
        assert_eq!(stored_header.logs_bloom, expected_bloom);
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
        assert!(storage.block_header(2).expect("header lookup").is_none());
        assert!(storage.block_logs(2).expect("logs lookup").is_none());
        assert!(storage.block_transactions(2).expect("txs lookup").is_none());
        assert!(storage.block_withdrawals(2).expect("withdrawals lookup").is_none());
        assert!(storage.block_size(2).expect("size lookup").is_none());
        let addr_index = storage
            .log_index_by_address_range(address_from_u64(2), 2..=2)
            .expect("address index lookup");
        assert!(addr_index.is_empty());

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
