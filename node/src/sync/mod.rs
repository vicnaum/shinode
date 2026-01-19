//! Sync and ingest orchestration.

use crate::{
    chain::{ChainError, ChainTracker, ChainUpdate, HeadTracker, HeaderStub},
    storage::Storage,
};
use eyre::Result;
use std::ops::RangeInclusive;

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
pub trait BlockSource: Send + Sync {
    fn head(&self) -> Result<u64>;
    fn headers_by_number(&self, range: RangeInclusive<u64>) -> Result<Vec<HeaderStub>>;
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

    pub fn run_once(&mut self, storage: &Storage, start_block: u64) -> Result<SyncOutcome> {
        let head = self.source.head()?;
        storage.set_head_seen(head)?;

        let controller = SyncController::new(super::MemoryHeadTracker::new(Some(head)), self.batch_size);
        let range = match controller.next_range(storage, start_block)? {
            Some(range) => range,
            None => return Ok(SyncOutcome::UpToDate { head }),
        };

        let headers = self.source.headers_by_number(range.clone())?;
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

#[cfg(test)]
mod tests {
    use super::{BlockSource, RangeSyncPlanner, SyncController, SyncOutcome, SyncRunner};
    use crate::{
        chain::{HeaderStub, MemoryHeadTracker},
        cli::{HeadSource, NodeConfig, ReorgStrategy, RetentionMode},
        storage::Storage,
    };
    use eyre::Result;
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

    impl BlockSource for VecBlockSource {
        fn head(&self) -> Result<u64> {
            Ok(self.head)
        }

        fn headers_by_number(&self, range: RangeInclusive<u64>) -> Result<Vec<HeaderStub>> {
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

    fn linear_headers(start: u64, end: u64, base_hash: u64) -> Vec<HeaderStub> {
        let mut headers = Vec::new();
        let mut prev_hash = base_hash;
        for number in start..=end {
            let hash = prev_hash + 1;
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

    #[test]
    fn sync_runner_advances_checkpoints() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let headers = linear_headers(0, 5, 100);
        let source = VecBlockSource { head: 5, headers };
        let mut runner = SyncRunner::new(source, 3);

        let outcome = runner
            .run_once(&storage, config.start_block)
            .expect("run once");
        assert_eq!(
            outcome,
            SyncOutcome::RangeApplied { range: 0..=2 }
        );
        assert_eq!(storage.last_indexed_block().unwrap(), Some(2));

        let outcome = runner
            .run_once(&storage, config.start_block)
            .expect("run second");
        assert_eq!(
            outcome,
            SyncOutcome::RangeApplied { range: 3..=5 }
        );
        assert_eq!(storage.last_indexed_block().unwrap(), Some(5));

        let outcome = runner
            .run_once(&storage, config.start_block)
            .expect("run third");
        assert_eq!(outcome, SyncOutcome::UpToDate { head: 5 });

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sync_runner_handles_reorg_by_rolling_back_checkpoint() {
        let dir = temp_dir();
        let config = base_config(dir.clone());
        let storage = Storage::open(&config).expect("storage");

        let headers = linear_headers(0, 2, 200);
        let source = VecBlockSource { head: 2, headers: headers.clone() };
        let mut runner = SyncRunner::new(source, 3);
        runner
            .run_once(&storage, config.start_block)
            .expect("initial sync");
        assert_eq!(storage.last_indexed_block().unwrap(), Some(2));

        storage.set_last_indexed_block(1).expect("rewind");
        let alt_header = HeaderStub {
            number: 2,
            hash: 999,
            parent_hash: headers[1].hash,
        };
        let source = VecBlockSource { head: 2, headers: vec![alt_header] };
        runner.source = source;
        let outcome = runner
            .run_once(&storage, config.start_block)
            .expect("reorg run");
        assert_eq!(outcome, SyncOutcome::Reorg { ancestor_number: 1 });
        assert_eq!(storage.last_indexed_block().unwrap(), Some(1));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
