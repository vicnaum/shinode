# Priority Escalation Queue Plan

## Problem Statement

The current escalation/failed blocks recovery mechanism has several issues:

1. **Runs opportunistically during sync** - Escalation kicks in whenever the normal queue is temporarily empty, causing UI flickering
2. **Blocks can be silently dropped** - When all peers have tried a block, it's removed from escalation but stays in `failed` forever
3. **`escalation.active` never resets** - New failed blocks won't trigger another escalation round
4. **`is_done()` ignores failed blocks** - Sync can "complete" while failed blocks remain
5. **No indefinite retry** - Blocks that fail with all peers are abandoned

## Solution: Priority Escalation Queue

### Core Concept

1. **Priority queue for troubled blocks** - After N attempts (default: 5), blocks are promoted to an "escalation queue"
2. **Escalation has priority** - Free peers check escalation queue FIRST, then normal queue
3. **Indefinite retry** - Escalation blocks keep retrying forever until fetched
4. **Peer rotation with cooldown** - Track which peers tried each block, prefer peers who haven't tried recently
5. **Shard-aware prioritization** - Prioritize blocks from shards closest to completion (enables earlier compaction)
6. **Clean UI** - No recovery bar during sync; only show red recovery bar when ONLY escalation blocks remain

### Scope

- **Fast-sync only** - Follow mode already has `max_attempts_per_block = u32::MAX` (infinite retries)
- Escalation logic does not apply to follow mode

---

## Data Structures

### EscalationState (revised)

```rust
// In scheduler.rs

/// Cooldown before same peer retries same escalation block
const ESCALATION_PEER_COOLDOWN: Duration = Duration::from_secs(30);

/// Attempts in normal queue before promotion to escalation
const ESCALATION_THRESHOLD: u32 = 5;

struct EscalationState {
    /// Blocks awaiting retry, grouped by shard for prioritization
    /// Key: shard_start, Value: blocks in that shard needing retry
    shards: HashMap<u64, HashSet<u64>>,

    /// Total count for quick access
    total_count: usize,

    /// Track which peers tried each block and when
    /// Key: block_number, Value: map of peer_id -> last_attempt_timestamp
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
                self.total_count -= 1;
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
```

### Shard Completion Tracking

```rust
// Track how many blocks each shard is missing (for prioritization)
// This can be derived from the storage or tracked separately

struct ShardCompletionInfo {
    shard_start: u64,
    shard_size: u64,
    missing_count: usize,  // Lower = higher priority
}
```

---

## Core Logic Changes

### 1. `next_batch_for_peer()` - Priority Order

```rust
pub async fn next_batch_for_peer(
    &self,
    peer_id: PeerId,
    peer_head: u64,
    active_peers: usize,
) -> FetchBatch {
    if self.is_peer_banned(peer_id).await {
        return FetchBatch::empty();
    }
    if self.peer_health.should_defer_peer(peer_id, active_peers).await {
        return FetchBatch::empty();
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

    FetchBatch::empty()
}
```

### 2. `pop_escalation_for_peer()` - Shard-Aware with Peer Preference

```rust
async fn pop_escalation_for_peer(&self, peer_id: PeerId) -> Option<u64> {
    let mut escalation = self.escalation.lock().await;
    let now = Instant::now();
    let shard_size = self.config.shard_size;

    if escalation.is_empty() {
        return None;
    }

    // Get shard completion info and sort by "closest to complete"
    // (fewest missing blocks = highest priority)
    let mut shard_priorities: Vec<_> = escalation.shards.iter()
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

        for &block in shard_blocks.iter() {
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
                    if best_staleness.map(|b| duration > b).unwrap_or(true) {
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
            escalation.shards.get_mut(&shard_start).unwrap().remove(&block);
            escalation.total_count -= 1;
            if escalation.shards.get(&shard_start).map(|s| s.is_empty()).unwrap_or(false) {
                escalation.shards.remove(&shard_start);
            }

            // Record this attempt
            escalation.peer_attempts
                .entry(block)
                .or_default()
                .insert(peer_id, now);

            // Add to in_flight
            self.in_flight.lock().await.insert(block);

            return Some(block);
        }
    }

    None  // All blocks recently tried by this peer
}
```

### 3. `requeue_failed()` - Escalation After N Attempts

```rust
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
        *count += 1;

        if *count <= ESCALATION_THRESHOLD {
            // Normal retry
            if queued.insert(*block) {
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
```

### 4. `requeue_escalation_block()` - Simple Requeue

```rust
pub async fn requeue_escalation_block(&self, block: u64) {
    let mut escalation = self.escalation.lock().await;
    let mut in_flight = self.in_flight.lock().await;
    let shard_size = self.config.shard_size;

    in_flight.remove(&block);

    // Re-add to escalation queue
    // peer_attempts is preserved for cooldown tracking
    escalation.add_block(block, shard_size);
}
```

### 5. `mark_completed()` - Clean Up Escalation

```rust
pub async fn mark_completed(&self, blocks: &[u64]) -> u64 {
    let mut recovered = 0u64;
    let mut escalation = self.escalation.lock().await;
    let mut completed = self.completed.lock().await;
    let mut failed = self.failed.lock().await;
    let mut in_flight = self.in_flight.lock().await;
    let shard_size = self.config.shard_size;

    for block in blocks {
        in_flight.remove(block);
        completed.insert(*block);

        if failed.remove(block) {
            recovered += 1;
        }

        // Also remove from escalation if present
        escalation.remove_block(*block, shard_size);
    }

    recovered
}
```

### 6. `is_done()` - Include Escalation

```rust
pub async fn is_done(&self) -> bool {
    let pending_empty = self.pending.lock().await.is_empty();
    let inflight_empty = self.in_flight.lock().await.is_empty();
    let escalation_empty = self.escalation.lock().await.is_empty();

    pending_empty && inflight_empty && escalation_empty
}
```

### 7. Remove `maybe_start_escalation()`

No longer needed - escalation is checked on every `next_batch_for_peer()` call.

### 8. Remove `escalation.active` Flag

No longer needed - we just check if escalation queue has blocks.

---

## Stats Changes

### SyncProgressSnapshot

```rust
pub struct SyncProgressSnapshot {
    pub processed: u64,
    pub failed: u64,
    pub queue: u64,
    pub inflight: u64,
    pub peers_active: u64,
    pub peers_total: u64,
    pub status: SyncStatus,
    pub compactions_done: u64,
    pub compactions_total: u64,
    pub head_block: u64,
    pub head_seen: u64,
    pub fetch_complete: bool,
    pub escalation: u64,  // NEW: blocks in escalation queue
}
```

### SyncProgressStats

```rust
impl SyncProgressStats {
    // Add new field
    escalation: AtomicU64,

    pub fn set_escalation(&self, count: u64) {
        self.escalation.store(count, Ordering::SeqCst);
    }

    // Update snapshot() to include escalation
}
```

---

## UI Changes

### Progress Updater Logic

```rust
// In main.rs spawn_progress_updater()

let snapshot = stats.snapshot();

// Determine if we're in "recovery only" mode
let normal_queue_empty = snapshot.queue == 0;
let has_escalation = snapshot.escalation > 0;
let only_recovery_left = normal_queue_empty && has_escalation && !snapshot.fetch_complete;

if only_recovery_left {
    // Switch to red recovery bar
    if recovery_bar.is_none() {
        let rb = create_failed_bar(&multi, snapshot.escalation);
        rb.set_message(format!("Recovering {} blocks...", snapshot.escalation));
        recovery_bar = Some(rb);
        // Hide the main sync bar
        bar.set_draw_target(ProgressDrawTarget::hidden());
    }

    if let Some(ref rb) = recovery_bar {
        rb.set_length(snapshot.escalation.max(1));
        rb.set_position(0);  // We don't track "recovered from escalation" separately
        rb.set_message(format!("Recovering {} blocks...", snapshot.escalation));
    }
} else {
    // Normal sync mode
    if let Some(rb) = recovery_bar.take() {
        rb.finish_and_clear();
        bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
    }

    // Update normal bar with escalation count in status
    let msg = format_progress_message(
        snapshot.status,
        peers_active,
        snapshot.peers_total,
        snapshot.queue,
        snapshot.inflight,
        snapshot.escalation,  // Changed from "failed" to "escalation"
        // ...
    );
    bar.set_message(msg);
}

// When escalation empties, recovery is done
if snapshot.escalation == 0 && recovery_bar.is_some() {
    if let Some(rb) = recovery_bar.take() {
        rb.finish_and_clear();
    }
}
```

### Status Message Format

Change "failed X" to "slow X" or "retry X" in the status message to be less alarming:

```rust
pub fn format_progress_message(..., escalation: u64, ...) -> String {
    format!(
        "status {} | peers {}/{} | queue {} | inflight {} | retry {}{} | speed {:.1}/s | eta {}",
        // ...
        escalation,  // "retry X" instead of "failed X"
        // ...
    )
}
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `sync/historical/scheduler.rs` | Rewrite `EscalationState`, add shard-aware logic, change priority order, add peer tracking |
| `sync/historical/mod.rs` | Update stats reporting, remove `maybe_start_escalation` calls |
| `sync/mod.rs` | Add `escalation` field to `SyncProgressStats` and `SyncProgressSnapshot` |
| `main.rs` | Update UI logic for recovery-only mode |
| `ui/progress.rs` | Update `format_progress_message` signature |

---

## Configuration

```rust
// In scheduler.rs or cli/mod.rs

/// Attempts in normal queue before promotion to escalation queue
pub const DEFAULT_ESCALATION_THRESHOLD: u32 = 5;

/// Cooldown before same peer retries same escalation block
pub const DEFAULT_ESCALATION_PEER_COOLDOWN_SECS: u64 = 30;
```

These could be made CLI-configurable later if needed.

---

## Edge Cases

### 1. All Peers Have Tried a Block Recently

If all connected peers have tried a block within the cooldown period:
- `pop_escalation_for_peer()` returns `None` for all peers
- Block stays in escalation queue
- Eventually cooldown expires and peers can retry
- New peers connecting get immediate access to try

### 2. Peer Disconnects Mid-Attempt

- Block is in `in_flight`
- Fetch times out or errors
- `requeue_escalation_block()` is called
- Block returns to escalation queue
- Peer's attempt timestamp is preserved

### 3. Block Succeeds After Many Failures

- `mark_completed()` removes from escalation queue
- `peer_attempts` for that block is cleaned up
- Stats updated

### 4. Shard Completes

- As blocks succeed, shard's missing count decreases
- When all blocks in a shard are fetched, shard is removed from `escalation.shards`
- Shard can now compact

### 5. Very Large Escalation Queue

- Shard-aware prioritization ensures shards close to completion get priority
- This prevents "starvation" where nearly-complete shards wait behind large incomplete shards

---

## Testing Plan

1. **Unit test**: Block promotion after N attempts
2. **Unit test**: Peer cooldown respected
3. **Unit test**: Shard prioritization (fewer missing = higher priority)
4. **Unit test**: Block removal on success
5. **Integration test**: Escalation completes when all blocks fetched
6. **Manual test**: UI shows recovery bar only when normal queue empty

---

## Success Criteria

1. No blocks are ever "lost" or abandoned
2. UI doesn't flicker during normal sync
3. Red recovery bar only appears when ONLY escalation blocks remain
4. Shards closer to completion get priority for faster compaction
5. Peers aren't hammered with the same block repeatedly
