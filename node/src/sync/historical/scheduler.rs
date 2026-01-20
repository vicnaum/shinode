//! Peer-driven scheduler for historical sync.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    time::{Duration, Instant},
};

use reth_network_api::PeerId;
use tokio::sync::Mutex;
use crate::sync::historical::types::{FetchBatch, FetchMode};

/// Scheduler configuration.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub blocks_per_assignment: usize,
    pub max_attempts_per_block: u32,
    pub peer_failure_threshold: u32,
    pub peer_ban_duration: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            blocks_per_assignment: 32,
            max_attempts_per_block: 3,
            peer_failure_threshold: 5,
            peer_ban_duration: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Default)]
struct PeerHealth {
    consecutive_failures: u32,
    banned_until: Option<Instant>,
}

impl PeerHealth {
    fn is_banned(&self) -> bool {
        self.banned_until
            .map(|until| Instant::now() < until)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    fn ban_remaining(&self) -> Option<Duration> {
        self.banned_until.and_then(|until| {
            let now = Instant::now();
            if now < until {
                Some(until - now)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Default)]
struct EscalationState {
    active: bool,
    queue: VecDeque<u64>,
    queued: HashSet<u64>,
    attempts: HashMap<u64, HashSet<PeerId>>,
}

/// Peer-driven scheduler state.
#[derive(Debug)]
pub struct PeerWorkScheduler {
    config: SchedulerConfig,
    pending: Mutex<BinaryHeap<Reverse<u64>>>,
    queued: Mutex<HashSet<u64>>,
    in_flight: Mutex<HashSet<u64>>,
    completed: Mutex<HashSet<u64>>,
    failed: Mutex<HashSet<u64>>,
    attempts: Mutex<HashMap<u64, u32>>,
    peer_health: Mutex<HashMap<PeerId, PeerHealth>>,
    escalation: Mutex<EscalationState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheduler_with_blocks(config: SchedulerConfig, start: u64, end: u64) -> PeerWorkScheduler {
        let blocks = (start..=end).collect::<Vec<_>>();
        PeerWorkScheduler::new(config, blocks)
    }

    #[tokio::test]
    async fn next_batch_respects_head_and_consecutive() {
        let mut config = SchedulerConfig::default();
        config.blocks_per_assignment = 3;
        let scheduler = scheduler_with_blocks(config, 0, 9);
        let peer_id = PeerId::random();

        let batch = scheduler.next_batch_for_peer(peer_id, 4, 1).await;
        assert_eq!(batch.blocks, vec![0, 1, 2]);
        assert_eq!(batch.mode, FetchMode::Normal);

        let batch = scheduler.next_batch_for_peer(peer_id, 4, 1).await;
        assert_eq!(batch.blocks, vec![3, 4]);
        assert_eq!(batch.mode, FetchMode::Normal);

        let batch = scheduler.next_batch_for_peer(peer_id, 4, 1).await;
        assert!(batch.blocks.is_empty());
    }

    #[tokio::test]
    async fn requeue_marks_failed_after_max_attempts() {
        let mut config = SchedulerConfig::default();
        config.max_attempts_per_block = 2;
        let scheduler = scheduler_with_blocks(config, 1, 1);
        let block = 1;

        scheduler.requeue_failed(&[block]).await;
        assert_eq!(scheduler.failed_count().await, 0);

        scheduler.requeue_failed(&[block]).await;
        assert_eq!(scheduler.failed_count().await, 0);

        scheduler.requeue_failed(&[block]).await;
        assert_eq!(scheduler.failed_count().await, 1);
    }

    #[tokio::test]
    async fn escalation_retries_failed_blocks() {
        let mut config = SchedulerConfig::default();
        config.max_attempts_per_block = 1;
        let scheduler = scheduler_with_blocks(config, 1, 1);
        let peer_id = PeerId::random();

        let batch = scheduler.next_batch_for_peer(peer_id, 10, 1).await;
        assert_eq!(batch.blocks, vec![1]);

        scheduler.requeue_failed(&[1]).await;
        let batch = scheduler.next_batch_for_peer(peer_id, 10, 1).await;
        assert_eq!(batch.blocks, vec![1]);

        scheduler.requeue_failed(&[1]).await;
        let batch = scheduler.next_batch_for_peer(peer_id, 10, 1).await;
        assert_eq!(batch.blocks, vec![1]);
        assert_eq!(batch.mode, FetchMode::Escalation);
    }

    #[tokio::test]
    async fn peer_ban_triggers_after_threshold() {
        let mut config = SchedulerConfig::default();
        config.peer_failure_threshold = 2;
        config.peer_ban_duration = Duration::from_secs(60);
        let scheduler = scheduler_with_blocks(config, 0, 0);
        let peer_id = PeerId::random();

        scheduler.record_peer_failure(peer_id).await;
        assert!(!scheduler.is_peer_banned(peer_id).await);

        scheduler.record_peer_failure(peer_id).await;
        assert!(scheduler.is_peer_banned(peer_id).await);
    }
}

impl PeerWorkScheduler {
    /// Create a scheduler with an initial set of pending blocks.
    pub fn new(config: SchedulerConfig, blocks: Vec<u64>) -> Self {
        let queued: HashSet<u64> = blocks.iter().copied().collect();
        let pending = blocks.into_iter().map(Reverse).collect::<BinaryHeap<_>>();
        Self {
            config,
            pending: Mutex::new(pending),
            queued: Mutex::new(queued),
            in_flight: Mutex::new(HashSet::new()),
            completed: Mutex::new(HashSet::new()),
            failed: Mutex::new(HashSet::new()),
            attempts: Mutex::new(HashMap::new()),
            peer_health: Mutex::new(HashMap::new()),
            escalation: Mutex::new(EscalationState::default()),
        }
    }

    /// Returns the next batch for a peer (normal or escalation).
    pub async fn next_batch_for_peer(
        &self,
        peer_id: PeerId,
        peer_head: u64,
        active_peers: usize,
    ) -> FetchBatch {
        if self.is_peer_banned(peer_id).await {
            return FetchBatch {
                blocks: Vec::new(),
                mode: FetchMode::Normal,
            };
        }

        let normal = self.pop_next_batch_for_head(peer_head).await;
        if !normal.is_empty() {
            return FetchBatch {
                blocks: normal,
                mode: FetchMode::Normal,
            };
        }

        self.maybe_start_escalation().await;
        let escalation = self.pop_escalation_for_peer(peer_id, active_peers).await;
        FetchBatch {
            blocks: escalation,
            mode: FetchMode::Escalation,
        }
    }

    async fn pop_next_batch_for_head(&self, peer_head: u64) -> Vec<u64> {
        let mut pending = self.pending.lock().await;
        let mut queued = self.queued.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        let mut batch = Vec::with_capacity(self.config.blocks_per_assignment);
        let mut last: Option<u64> = None;

        while batch.len() < self.config.blocks_per_assignment {
            let Some(Reverse(next)) = pending.peek().copied() else {
                break;
            };

            if next > peer_head {
                break;
            }
            if let Some(prev) = last {
                if next != prev.saturating_add(1) {
                    break;
                }
            }

            pending.pop();
            queued.remove(&next);
            in_flight.insert(next);
            batch.push(next);
            last = Some(next);
        }

        batch
    }

    async fn maybe_start_escalation(&self) {
        let pending_len = self.pending.lock().await.len();
        if pending_len > 0 {
            return;
        }
        let failed_blocks: Vec<u64> = {
            let failed = self.failed.lock().await;
            failed.iter().copied().collect()
        };
        if failed_blocks.is_empty() {
            return;
        }

        let mut escalation = self.escalation.lock().await;
        if escalation.active {
            return;
        }
        for block in failed_blocks {
            if escalation.queued.insert(block) {
                escalation.queue.push_back(block);
            }
        }
        escalation.active = true;
    }

    async fn pop_escalation_for_peer(
        &self,
        peer_id: PeerId,
        active_peers: usize,
    ) -> Vec<u64> {
        let mut escalation = self.escalation.lock().await;
        let completed = self.completed.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        let mut iterations = escalation.queue.len();
        while iterations > 0 {
            iterations -= 1;
            if let Some(block) = escalation.queue.pop_front() {
                escalation.queued.remove(&block);
                if completed.contains(&block) {
                    escalation.attempts.remove(&block);
                    continue;
                }
                let entry = escalation.attempts.entry(block).or_default();
                if entry.contains(&peer_id) {
                    escalation.queue.push_back(block);
                    escalation.queued.insert(block);
                    continue;
                }
                if active_peers > 0 && entry.len() >= active_peers {
                    escalation.attempts.remove(&block);
                    continue;
                }
                entry.insert(peer_id);
                in_flight.insert(block);
                return vec![block];
            }
        }
        Vec::new()
    }

    /// Mark blocks as completed and remove from in-flight.
    pub async fn mark_completed(&self, blocks: &[u64]) -> u64 {
        let mut recovered = 0u64;
        let mut escalation = self.escalation.lock().await;
        let mut completed = self.completed.lock().await;
        let mut failed = self.failed.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        for block in blocks {
            in_flight.remove(block);
            completed.insert(*block);
            if failed.remove(block) {
                recovered = recovered.saturating_add(1);
            }
        }

        if !blocks.is_empty() {
            let removal: HashSet<u64> = blocks.iter().copied().collect();
            escalation.queue.retain(|block| !removal.contains(block));
            for block in &removal {
                escalation.queued.remove(block);
                escalation.attempts.remove(block);
            }
        }

        recovered
    }

    /// Requeue failed blocks or mark them failed after max attempts.
    pub async fn requeue_failed(&self, blocks: &[u64]) -> Vec<u64> {
        let mut pending = self.pending.lock().await;
        let mut queued = self.queued.lock().await;
        let mut attempts = self.attempts.lock().await;
        let mut failed = self.failed.lock().await;
        let mut in_flight = self.in_flight.lock().await;
        let mut newly_failed = Vec::new();

        for block in blocks {
            in_flight.remove(block);
            let count = attempts.entry(*block).or_insert(0);
            *count += 1;
            if *count <= self.config.max_attempts_per_block {
                if !queued.contains(block) {
                    queued.insert(*block);
                    pending.push(Reverse(*block));
                }
            } else {
                if failed.insert(*block) {
                    newly_failed.push(*block);
                }
            }
        }
        newly_failed
    }

    /// Requeue a failed escalation block if more peers remain to try.
    pub async fn requeue_escalation_block(&self, block: u64, active_peers: usize) {
        let mut escalation = self.escalation.lock().await;
        let completed = self.completed.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        in_flight.remove(&block);
        if completed.contains(&block) {
            escalation.attempts.remove(&block);
            return;
        }

        let tried = escalation.attempts.get(&block).map(|s| s.len()).unwrap_or(0);
        if active_peers == 0 || tried >= active_peers {
            return;
        }
        if escalation.queued.insert(block) {
            escalation.queue.push_back(block);
        }
    }

    /// Record a successful peer batch.
    pub async fn record_peer_success(&self, peer_id: PeerId) {
        let mut health = self.peer_health.lock().await;
        let entry = health.entry(peer_id).or_default();
        entry.consecutive_failures = 0;
        entry.banned_until = None;
    }

    /// Record a peer failure and apply bans if needed.
    pub async fn record_peer_failure(&self, peer_id: PeerId) {
        let mut health = self.peer_health.lock().await;
        let entry = health.entry(peer_id).or_default();
        entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
        if entry.consecutive_failures >= self.config.peer_failure_threshold {
            entry.banned_until = Some(Instant::now() + self.config.peer_ban_duration);
        }
    }

    /// Check if peer is currently banned.
    pub async fn is_peer_banned(&self, peer_id: PeerId) -> bool {
        let health = self.peer_health.lock().await;
        health
            .get(&peer_id)
            .map(|entry| entry.is_banned())
            .unwrap_or(false)
    }

    /// Check if all work is complete.
    pub async fn is_done(&self) -> bool {
        let pending = self.pending.lock().await;
        let in_flight = self.in_flight.lock().await;
        let escalation = self.escalation.lock().await;
        pending.is_empty() && in_flight.is_empty() && escalation.queue.is_empty()
    }

    /// Number of completed blocks.
    #[allow(dead_code)]
    pub async fn completed_count(&self) -> usize {
        let completed = self.completed.lock().await;
        completed.len()
    }

    /// Number of pending blocks.
    pub async fn pending_count(&self) -> usize {
        let pending = self.pending.lock().await;
        let escalation = self.escalation.lock().await;
        pending.len() + escalation.queue.len()
    }

    pub async fn pending_main_count(&self) -> usize {
        let pending = self.pending.lock().await;
        pending.len()
    }

    pub async fn escalation_len(&self) -> usize {
        let escalation = self.escalation.lock().await;
        escalation.queue.len()
    }

    pub async fn escalation_attempted_count(&self) -> usize {
        let escalation = self.escalation.lock().await;
        escalation.attempts.len()
    }

    /// Number of in-flight blocks.
    pub async fn inflight_count(&self) -> usize {
        let in_flight = self.in_flight.lock().await;
        in_flight.len()
    }

    /// Number of failed blocks.
    #[allow(dead_code)]
    pub async fn failed_count(&self) -> usize {
        let failed = self.failed.lock().await;
        failed.len()
    }
}
