//! Peer-driven scheduler for historical sync.

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::sync::historical::types::{FetchBatch, FetchMode};
use reth_network_api::PeerId;
use tokio::sync::Mutex;

/// How long to wait before allowing a peer to retry the same escalation block.
const ESCALATION_PEER_COOLDOWN: Duration = Duration::from_secs(30);

/// Default number of attempts before a block is promoted to escalation queue.
const DEFAULT_ESCALATION_THRESHOLD: u32 = 5;

/// Shard-aware escalation state for priority retry of difficult blocks.
///
/// Blocks that fail N attempts in the normal queue are promoted here and get
/// priority handling (checked before normal queue). The shard grouping enables
/// prioritizing blocks from nearly-complete shards to enable faster compaction.
#[derive(Debug, Default)]
struct EscalationState {
    /// Blocks awaiting retry, grouped by shard for prioritization.
    /// Key: shard_start, Value: blocks in that shard needing retry.
    shards: HashMap<u64, HashSet<u64>>,

    /// Total count for quick access.
    total_count: usize,

    /// Track which peers tried each block and when (for cooldown).
    /// Key: block_number, Value: map of peer_id -> last_attempt_timestamp.
    peer_attempts: HashMap<u64, HashMap<PeerId, Instant>>,
}

impl EscalationState {
    fn add_block(&mut self, block: u64, shard_size: u64) {
        let shard_start = (block / shard_size) * shard_size;
        if self.shards.entry(shard_start).or_default().insert(block) {
            self.total_count += 1;
        }
    }

    fn remove_block(&mut self, block: u64, shard_size: u64) {
        let shard_start = (block / shard_size) * shard_size;
        if let Some(shard_blocks) = self.shards.get_mut(&shard_start) {
            if shard_blocks.remove(&block) {
                self.total_count = self.total_count.saturating_sub(1);
                if shard_blocks.is_empty() {
                    self.shards.remove(&shard_start);
                }
            }
        }
        self.peer_attempts.remove(&block);
    }

    fn is_empty(&self) -> bool {
        self.total_count == 0
    }

    fn len(&self) -> usize {
        self.total_count
    }
}

/// Scheduler configuration.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Hard cap for blocks assigned in a single peer batch.
    pub blocks_per_assignment: usize,
    /// Initial blocks per peer batch (before AIMD adjusts).
    pub initial_blocks_per_assignment: usize,
    /// Number of attempts before a block is promoted to escalation queue.
    pub max_attempts_per_block: u32,
    pub peer_failure_threshold: u32,
    pub peer_ban_duration: Duration,
    /// Shard size for shard-aware escalation prioritization.
    pub shard_size: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            blocks_per_assignment: 32,
            initial_blocks_per_assignment: 32,
            max_attempts_per_block: DEFAULT_ESCALATION_THRESHOLD,
            peer_failure_threshold: 5,
            peer_ban_duration: Duration::from_secs(120),
            shard_size: 10_000,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PeerHealthConfig {
    peer_failure_threshold: u32,
    peer_ban_duration: Duration,
    aimd_increase_after: u32,
    aimd_partial_decrease: f64,
    aimd_failure_decrease: f64,
    aimd_min_batch: usize,
    aimd_initial_batch: usize,
    aimd_max_batch: usize,
    quality_partial_weight: f64,
    quality_min_samples: u64,
    quality_defer_threshold: f64,
}

impl PeerHealthConfig {
    pub(crate) fn from_scheduler_config(config: &SchedulerConfig) -> Self {
        let max_batch = config.blocks_per_assignment.max(1);
        let initial_batch = config.initial_blocks_per_assignment.max(1).min(max_batch);
        Self {
            peer_failure_threshold: config.peer_failure_threshold,
            peer_ban_duration: config.peer_ban_duration,
            aimd_increase_after: 5,
            aimd_partial_decrease: 0.7,
            aimd_failure_decrease: 0.5,
            aimd_min_batch: 1,
            aimd_initial_batch: initial_batch,
            aimd_max_batch: max_batch,
            quality_partial_weight: 0.5,
            quality_min_samples: 5,
            quality_defer_threshold: 0.6,
        }
    }
}

#[derive(Debug, Default)]
struct PeerHealth {
    consecutive_failures: u32,
    consecutive_partials: u32,
    banned_until: Option<Instant>,
    successes: u64,
    failures: u64,
    partials: u64,
    assignments: u64,
    assigned_blocks: u64,
    inflight_blocks: u64,
    last_assigned_at: Option<Instant>,
    last_success_at: Option<Instant>,
    last_failure_at: Option<Instant>,
    last_partial_at: Option<Instant>,
    last_error: Option<String>,
    last_error_at: Option<Instant>,
    last_error_count: u64,
    batch_limit: usize,
    batch_limit_max: usize,
    batch_limit_sum: u64,
    batch_limit_samples: u64,
    success_streak: u32,
}

impl PeerHealth {
    fn is_banned(&self) -> bool {
        self.banned_until
            .is_some_and(|until| Instant::now() < until)
    }
}

#[derive(Debug, Default)]
pub(crate) struct PeerQuality {
    pub score: f64,
    pub samples: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PeerHealthDump {
    pub peer_id: PeerId,
    pub is_banned: bool,
    pub ban_remaining_ms: Option<u64>,
    pub consecutive_failures: u32,
    pub consecutive_partials: u32,
    pub successes: u64,
    pub failures: u64,
    pub partials: u64,
    pub assignments: u64,
    pub assigned_blocks: u64,
    pub inflight_blocks: u64,
    pub last_assigned_age_ms: Option<u64>,
    pub quality_score: f64,
    pub quality_samples: u64,
    pub batch_limit: usize,
    pub batch_limit_max: usize,
    pub batch_limit_avg: Option<f64>,
    pub last_success_age_ms: Option<u64>,
    pub last_failure_age_ms: Option<u64>,
    pub last_partial_age_ms: Option<u64>,
    pub last_error: Option<String>,
    pub last_error_age_ms: Option<u64>,
    pub last_error_count: u64,
}

#[derive(Debug)]
pub(crate) struct PeerHealthTracker {
    config: PeerHealthConfig,
    health: Mutex<HashMap<PeerId, PeerHealth>>,
}

impl PeerHealthTracker {
    fn ensure_batch_limit(&self, entry: &mut PeerHealth) {
        if entry.batch_limit == 0 {
            entry.batch_limit = self.config.aimd_initial_batch.max(1);
        }
        if entry.batch_limit_max == 0 || entry.batch_limit > entry.batch_limit_max {
            entry.batch_limit_max = entry.batch_limit;
        }
    }

    fn clamp_batch_limit(&self, value: usize) -> usize {
        value
            .max(self.config.aimd_min_batch)
            .min(self.config.aimd_max_batch.max(1))
    }

    pub(crate) fn new(config: PeerHealthConfig) -> Self {
        Self {
            config,
            health: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn record_success(&self, peer_id: PeerId) {
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        self.ensure_batch_limit(entry);
        entry.successes = entry.successes.saturating_add(1);
        entry.consecutive_failures = 0;
        entry.consecutive_partials = 0;
        entry.banned_until = None;
        entry.last_success_at = Some(Instant::now());
        entry.success_streak = entry.success_streak.saturating_add(1);
        if entry.success_streak >= self.config.aimd_increase_after {
            entry.batch_limit = self.clamp_batch_limit(entry.batch_limit.saturating_add(1));
            entry.success_streak = 0;
        }
        entry.batch_limit_max = entry.batch_limit_max.max(entry.batch_limit);
    }

    pub(crate) async fn record_partial(&self, peer_id: PeerId) {
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        self.ensure_batch_limit(entry);
        entry.partials = entry.partials.saturating_add(1);
        entry.consecutive_partials = entry.consecutive_partials.saturating_add(1);
        entry.last_partial_at = Some(Instant::now());
        entry.success_streak = 0;
        let reduced =
            (entry.batch_limit as f64 * self.config.aimd_partial_decrease).floor() as usize;
        entry.batch_limit = self.clamp_batch_limit(reduced);
    }

    pub(crate) async fn record_failure(&self, peer_id: PeerId) {
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        self.ensure_batch_limit(entry);
        entry.failures = entry.failures.saturating_add(1);
        entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
        entry.last_failure_at = Some(Instant::now());
        entry.success_streak = 0;
        let reduced =
            (entry.batch_limit as f64 * self.config.aimd_failure_decrease).floor() as usize;
        entry.batch_limit = self.clamp_batch_limit(reduced);
        if entry.consecutive_failures >= self.config.peer_failure_threshold {
            entry.banned_until = Some(Instant::now() + self.config.peer_ban_duration);
            tracing::debug!(
                peer_id = ?peer_id,
                ban_seconds = self.config.peer_ban_duration.as_secs(),
                failures = entry.failures,
                "peer banned after consecutive failures"
            );
        }
    }

    pub(crate) async fn is_peer_banned(&self, peer_id: PeerId) -> bool {
        let health = self.health.lock().await;
        health
            .get(&peer_id)
            .is_some_and(PeerHealth::is_banned)
    }

    pub(crate) async fn note_error(&self, peer_id: PeerId, error: String) {
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        if entry.last_error.as_deref() == Some(error.as_str()) {
            entry.last_error_count = entry.last_error_count.saturating_add(1);
        } else {
            entry.last_error = Some(error);
            entry.last_error_count = 1;
        }
        entry.last_error_at = Some(Instant::now());
    }

    pub(crate) async fn record_assignment(&self, peer_id: PeerId, blocks: usize) {
        if blocks == 0 {
            return;
        }
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        self.ensure_batch_limit(entry);
        entry.assignments = entry.assignments.saturating_add(1);
        entry.assigned_blocks = entry.assigned_blocks.saturating_add(blocks as u64);
        let clamped = self.clamp_batch_limit(entry.batch_limit);
        entry.batch_limit = clamped;
        entry.batch_limit_sum = entry.batch_limit_sum.saturating_add(clamped as u64);
        entry.batch_limit_samples = entry.batch_limit_samples.saturating_add(1);
        entry.batch_limit_max = entry.batch_limit_max.max(clamped);
        entry.inflight_blocks = entry.inflight_blocks.saturating_add(blocks as u64);
        entry.last_assigned_at = Some(Instant::now());
    }

    pub(crate) async fn finish_assignment(&self, peer_id: PeerId, blocks: usize) {
        if blocks == 0 {
            return;
        }
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        self.ensure_batch_limit(entry);
        entry.inflight_blocks = entry.inflight_blocks.saturating_sub(blocks as u64);
    }

    pub(crate) async fn batch_limit(&self, peer_id: PeerId) -> usize {
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        self.ensure_batch_limit(entry);
        self.clamp_batch_limit(entry.batch_limit)
    }

    pub(crate) async fn set_batch_limit(&self, peer_id: PeerId, limit: u64) {
        let mut health = self.health.lock().await;
        let entry = health.entry(peer_id).or_default();
        entry.batch_limit = self.clamp_batch_limit(limit as usize);
        entry.batch_limit_max = entry.batch_limit_max.max(entry.batch_limit);
    }

    pub(crate) async fn count_banned_peers(&self, peer_ids: &[PeerId]) -> u64 {
        if peer_ids.is_empty() {
            return 0;
        }
        let now = Instant::now();
        let health = self.health.lock().await;
        let mut count = 0u64;
        for peer_id in peer_ids {
            if let Some(entry) = health.get(peer_id) {
                if entry.banned_until.is_some_and(|until| now < until) {
                    count = count.saturating_add(1);
                }
            }
        }
        count
    }

    pub(crate) async fn snapshot(&self) -> Vec<PeerHealthDump> {
        let now = Instant::now();
        let health = self.health.lock().await;
        let mut out = Vec::with_capacity(health.len());
        for (peer_id, entry) in health.iter() {
            let total = entry.successes + entry.failures + entry.partials;
            let quality_score = if total == 0 {
                1.0
            } else {
                let weighted_success = entry.successes as f64
                    + (entry.partials as f64 * self.config.quality_partial_weight);
                (weighted_success / total as f64).clamp(0.0, 1.0)
            };
            let ban_remaining_ms = entry.banned_until.and_then(|until| {
                if now < until {
                    Some((until - now).as_millis() as u64)
                } else {
                    None
                }
            });
            out.push(PeerHealthDump {
                peer_id: *peer_id,
                is_banned: entry.is_banned(),
                ban_remaining_ms,
                consecutive_failures: entry.consecutive_failures,
                consecutive_partials: entry.consecutive_partials,
                successes: entry.successes,
                failures: entry.failures,
                partials: entry.partials,
                assignments: entry.assignments,
                assigned_blocks: entry.assigned_blocks,
                inflight_blocks: entry.inflight_blocks,
                last_assigned_age_ms: entry
                    .last_assigned_at
                    .map(|t| now.duration_since(t).as_millis() as u64),
                quality_score,
                quality_samples: total,
                batch_limit: entry.batch_limit.max(self.config.aimd_min_batch),
                batch_limit_max: entry
                    .batch_limit_max
                    .max(entry.batch_limit.max(self.config.aimd_min_batch)),
                batch_limit_avg: if entry.batch_limit_samples > 0 {
                    Some(entry.batch_limit_sum as f64 / entry.batch_limit_samples as f64)
                } else {
                    None
                },
                last_success_age_ms: entry
                    .last_success_at
                    .map(|t| now.duration_since(t).as_millis() as u64),
                last_failure_age_ms: entry
                    .last_failure_at
                    .map(|t| now.duration_since(t).as_millis() as u64),
                last_partial_age_ms: entry
                    .last_partial_at
                    .map(|t| now.duration_since(t).as_millis() as u64),
                last_error: entry.last_error.clone(),
                last_error_age_ms: entry
                    .last_error_at
                    .map(|t| now.duration_since(t).as_millis() as u64),
                last_error_count: entry.last_error_count,
            });
        }
        out.sort_by(|a, b| {
            b.quality_score
                .partial_cmp(&a.quality_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    pub(crate) async fn quality(&self, peer_id: PeerId) -> PeerQuality {
        let health = self.health.lock().await;
        let Some(entry) = health.get(&peer_id) else {
            return PeerQuality {
                score: 1.0,
                samples: 0,
            };
        };
        let total = entry.successes + entry.failures + entry.partials;
        if total == 0 {
            return PeerQuality {
                score: 1.0,
                samples: 0,
            };
        }
        let weighted_success =
            entry.successes as f64 + (entry.partials as f64 * self.config.quality_partial_weight);
        PeerQuality {
            score: (weighted_success / total as f64).clamp(0.0, 1.0),
            samples: total,
        }
    }

    pub(crate) async fn should_defer_peer(&self, peer_id: PeerId, active_peers: usize) -> bool {
        // If we only have a couple of peers, keep using them even if they're not great.
        if active_peers <= 3 {
            return false;
        }
        let quality = self.quality(peer_id).await;
        if quality.samples < self.config.quality_min_samples {
            return false;
        }
        quality.score < self.config.quality_defer_threshold
    }
}

/// Peer-driven scheduler state.
#[derive(Debug)]
pub struct PeerWorkScheduler {
    config: SchedulerConfig,
    pending: Mutex<BinaryHeap<Reverse<u64>>>,
    queued: Mutex<HashSet<u64>>,
    in_flight: Mutex<HashSet<u64>>,
    completed: Mutex<HashSet<u64>>,
    attempts: Mutex<HashMap<u64, u32>>,
    peer_health: Arc<PeerHealthTracker>,
    escalation: Mutex<EscalationState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scheduler_with_blocks(config: SchedulerConfig, start: u64, end: u64) -> PeerWorkScheduler {
        let blocks = (start..=end).collect::<Vec<_>>();
        let peer_health = Arc::new(PeerHealthTracker::new(
            PeerHealthConfig::from_scheduler_config(&config),
        ));
        PeerWorkScheduler::new_with_health(config, blocks, peer_health)
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
    async fn requeue_promotes_to_escalation_after_max_attempts() {
        let mut config = SchedulerConfig::default();
        config.max_attempts_per_block = 2;
        let scheduler = scheduler_with_blocks(config, 1, 1);
        let block = 1;

        // First two attempts stay in normal queue
        scheduler.requeue_failed(&[block]).await;
        assert_eq!(scheduler.escalation_len().await, 0);

        scheduler.requeue_failed(&[block]).await;
        assert_eq!(scheduler.escalation_len().await, 0);

        // Third attempt promotes to escalation
        let escalated = scheduler.requeue_failed(&[block]).await;
        assert_eq!(escalated, vec![1]);
        assert_eq!(scheduler.escalation_len().await, 1);
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
    /// Create a scheduler with shared peer health tracking.
    pub fn new_with_health(
        config: SchedulerConfig,
        blocks: Vec<u64>,
        peer_health: Arc<PeerHealthTracker>,
    ) -> Self {
        let queued: HashSet<u64> = blocks.iter().copied().collect();
        let pending = blocks.into_iter().map(Reverse).collect::<BinaryHeap<_>>();
        Self {
            config,
            pending: Mutex::new(pending),
            queued: Mutex::new(queued),
            in_flight: Mutex::new(HashSet::new()),
            completed: Mutex::new(HashSet::new()),
            attempts: Mutex::new(HashMap::new()),
            peer_health,
            escalation: Mutex::new(EscalationState::default()),
        }
    }

    /// Returns the next batch for a peer (escalation first, then normal).
    ///
    /// Escalation blocks get priority - they are checked FIRST before the normal queue.
    /// This ensures difficult blocks don't get starved while new blocks keep arriving.
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
        if self
            .peer_health
            .should_defer_peer(peer_id, active_peers)
            .await
        {
            return FetchBatch {
                blocks: Vec::new(),
                mode: FetchMode::Normal,
            };
        }

        // ESCALATION FIRST (priority queue)
        // Returns single block for more granular retry control
        if let Some(block) = self.pop_escalation_for_peer(peer_id).await {
            return FetchBatch {
                blocks: vec![block],
                mode: FetchMode::Escalation,
            };
        }

        // Normal queue second
        let max_blocks = self.peer_health.batch_limit(peer_id).await;
        let normal = self.pop_next_batch_for_head(peer_head, max_blocks).await;
        if !normal.is_empty() {
            return FetchBatch {
                blocks: normal,
                mode: FetchMode::Normal,
            };
        }

        FetchBatch {
            blocks: Vec::new(),
            mode: FetchMode::Normal,
        }
    }

    /// Append a contiguous range of blocks to the pending queue.
    ///
    /// NOTE: Callers must ensure this range does not overlap with already scheduled blocks.
    /// This is intended for "tail" scheduling of new block ranges that were never enqueued.
    pub async fn enqueue_range(&self, range: std::ops::RangeInclusive<u64>) -> usize {
        let start = *range.start();
        let end = *range.end();
        if end < start {
            return 0;
        }
        let mut pending = self.pending.lock().await;
        let mut queued = self.queued.lock().await;
        let mut added = 0usize;
        for block in start..=end {
            if queued.insert(block) {
                pending.push(Reverse(block));
                added += 1;
            }
        }
        added
    }

    async fn pop_next_batch_for_head(&self, peer_head: u64, max_blocks: usize) -> Vec<u64> {
        let mut pending = self.pending.lock().await;
        let mut queued = self.queued.lock().await;
        let mut in_flight = self.in_flight.lock().await;

        let limit = max_blocks
            .max(1)
            .min(self.config.blocks_per_assignment.max(1));
        let mut batch = Vec::with_capacity(limit);
        let mut last: Option<u64> = None;

        while batch.len() < limit {
            let Some(Reverse(next)) = pending.peek().copied() else {
                break;
            };

            // Sharded storage removes contiguous watermark backpressure.
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

    /// Pop a block from the escalation queue for this peer.
    ///
    /// Uses shard-aware prioritization: shards with fewer missing blocks (closer to
    /// complete) get priority to enable faster compaction. Within a shard, prefers
    /// blocks this peer hasn't tried recently (cooldown tracking).
    async fn pop_escalation_for_peer(&self, peer_id: PeerId) -> Option<u64> {
        let mut escalation = self.escalation.lock().await;
        let completed = self.completed.lock().await;
        let mut in_flight = self.in_flight.lock().await;
        let now = Instant::now();

        if escalation.is_empty() {
            return None;
        }

        // Get shard priorities sorted by "closest to complete" (fewest blocks = highest priority)
        let mut shard_priorities: Vec<_> = escalation
            .shards
            .iter()
            .map(|(shard_start, blocks)| (*shard_start, blocks.len()))
            .collect();
        shard_priorities.sort_by_key(|(_, missing)| *missing);

        // Try shards in priority order
        for (shard_start, _) in shard_priorities {
            let Some(shard_blocks) = escalation.shards.get(&shard_start) else {
                continue;
            };

            // Find a block this peer hasn't tried recently
            let mut best_block: Option<u64> = None;
            let mut best_staleness: Option<Duration> = None;

            for &block in shard_blocks {
                // Skip completed blocks (will be cleaned up on success)
                if completed.contains(&block) {
                    continue;
                }

                let peer_attempts = escalation.peer_attempts.get(&block);
                let staleness = peer_attempts
                    .and_then(|attempts| attempts.get(&peer_id))
                    .map(|last| now.duration_since(*last));

                match staleness {
                    None => {
                        // Peer never tried this block - best choice
                        best_block = Some(block);
                        break;
                    }
                    Some(duration) if duration >= ESCALATION_PEER_COOLDOWN => {
                        // Peer tried but cooldown passed - acceptable
                        if best_staleness.is_none_or(|b| duration > b) {
                            best_block = Some(block);
                            best_staleness = Some(duration);
                        }
                    }
                    Some(_) => {
                        // Too recent, skip
                    }
                }
            }

            if let Some(block) = best_block {
                // Remove from shard set (will be re-added on failure)
                // SAFETY: best_block was found by iterating over shard entries above
                if let Some(shard_set) = escalation.shards.get_mut(&shard_start) {
                    shard_set.remove(&block);
                }
                escalation.total_count = escalation.total_count.saturating_sub(1);
                if escalation
                    .shards
                    .get(&shard_start)
                    .is_some_and(std::collections::HashSet::is_empty)
                {
                    escalation.shards.remove(&shard_start);
                }

                // Record this attempt
                escalation
                    .peer_attempts
                    .entry(block)
                    .or_default()
                    .insert(peer_id, now);

                // Add to in_flight
                in_flight.insert(block);

                return Some(block);
            }
        }

        None // All blocks recently tried by this peer
    }

    /// Mark blocks as completed and remove from in-flight and escalation.
    ///
    /// Returns the count of blocks that were in escalation (recovered from retry).
    pub async fn mark_completed(&self, blocks: &[u64]) -> u64 {
        let mut recovered = 0u64;
        let mut escalation = self.escalation.lock().await;
        let mut completed = self.completed.lock().await;
        let mut in_flight = self.in_flight.lock().await;
        let shard_size = self.config.shard_size;

        for block in blocks {
            in_flight.remove(block);
            completed.insert(*block);

            // Check if this block was in escalation (for recovery stats)
            if escalation.peer_attempts.contains_key(block) {
                recovered = recovered.saturating_add(1);
            }

            // Remove from escalation if present
            escalation.remove_block(*block, shard_size);
        }

        recovered
    }

    /// Requeue failed blocks or promote to escalation after max attempts.
    ///
    /// Returns list of blocks that were newly promoted to escalation.
    pub async fn requeue_failed(&self, blocks: &[u64]) -> Vec<u64> {
        let mut pending = self.pending.lock().await;
        let mut queued = self.queued.lock().await;
        let mut attempts = self.attempts.lock().await;
        let mut in_flight = self.in_flight.lock().await;
        let mut escalation = self.escalation.lock().await;
        let mut newly_escalated = Vec::new();
        let shard_size = self.config.shard_size;

        for block in blocks {
            in_flight.remove(block);
            let count = attempts.entry(*block).or_insert(0);
            *count = count.saturating_add(1);
            if *count <= self.config.max_attempts_per_block {
                // Normal retry
                if !queued.contains(block) {
                    queued.insert(*block);
                    pending.push(Reverse(*block));
                }
            } else {
                // Promote to escalation (priority queue)
                escalation.add_block(*block, shard_size);
                newly_escalated.push(*block);
            }
        }
        newly_escalated
    }

    /// Requeue a failed escalation block for indefinite retry.
    ///
    /// Unlike the old behavior, this always re-adds the block to the escalation queue.
    /// Blocks keep retrying forever until successfully fetched. The peer_attempts
    /// tracking with cooldown ensures peers aren't hammered with the same block.
    pub async fn requeue_escalation_block(&self, block: u64) {
        let mut escalation = self.escalation.lock().await;
        let completed = self.completed.lock().await;
        let mut in_flight = self.in_flight.lock().await;
        let shard_size = self.config.shard_size;

        in_flight.remove(&block);
        if completed.contains(&block) {
            escalation.remove_block(block, shard_size);
            return;
        }

        // Re-add to escalation queue (peer_attempts is preserved for cooldown tracking)
        escalation.add_block(block, shard_size);
    }

    /// Record a successful peer batch.
    pub async fn record_peer_success(&self, peer_id: PeerId) {
        self.peer_health.record_success(peer_id).await;
    }

    /// Record a partial response (some blocks returned, some missing).
    /// This is worse than success but not as bad as a full failure.
    /// Multiple consecutive partials will lead to a temporary ban.
    pub async fn record_peer_partial(&self, peer_id: PeerId) {
        self.peer_health.record_partial(peer_id).await;
    }

    /// Record a peer failure and apply bans if needed.
    pub async fn record_peer_failure(&self, peer_id: PeerId) {
        self.peer_health.record_failure(peer_id).await;
    }

    /// Check if peer is currently banned.
    pub async fn is_peer_banned(&self, peer_id: PeerId) -> bool {
        self.peer_health.is_peer_banned(peer_id).await
    }

    /// Check if all work is complete.
    pub async fn is_done(&self) -> bool {
        // IMPORTANT: avoid holding multiple async mutexes at once to prevent deadlocks with
        // fetch tasks that also take these locks in different orders.
        let pending_empty = self.pending.lock().await.is_empty();
        let inflight_empty = self.in_flight.lock().await.is_empty();
        let escalation_empty = self.escalation.lock().await.is_empty();
        pending_empty && inflight_empty && escalation_empty
    }

    /// Number of completed blocks.
    pub async fn completed_count(&self) -> usize {
        let completed = self.completed.lock().await;
        completed.len()
    }

    /// Number of pending blocks (normal queue + escalation).
    pub async fn pending_count(&self) -> usize {
        let pending = self.pending.lock().await;
        let escalation = self.escalation.lock().await;
        pending.len() + escalation.len()
    }

    pub async fn pending_main_count(&self) -> usize {
        let pending = self.pending.lock().await;
        pending.len()
    }

    pub async fn escalation_len(&self) -> usize {
        let escalation = self.escalation.lock().await;
        escalation.len()
    }

    pub async fn escalation_attempted_count(&self) -> usize {
        let escalation = self.escalation.lock().await;
        escalation.peer_attempts.len()
    }

    pub async fn attempts_len(&self) -> usize {
        let attempts = self.attempts.lock().await;
        attempts.len()
    }

    /// Number of in-flight blocks.
    pub async fn inflight_count(&self) -> usize {
        let in_flight = self.in_flight.lock().await;
        in_flight.len()
    }
}
