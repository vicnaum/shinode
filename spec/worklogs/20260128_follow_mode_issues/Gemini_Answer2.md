Based on the analysis of the Repomix file and the provided logs/symptoms, here is the detailed breakdown of the bug.

### Summary
*   **Root Cause:** "Escalation Deadlock." In follow mode, blocks near the tip naturally fail to fetch occasionally (propagation lag). After 5 failures, they enter the **Escalation Queue**. The scheduler applies an exponential **per-peer-per-block backoff** capped at 128 seconds. When all connected peers have recently failed a specific block, the scheduler refuses to assign that block to anyone.
*   **The Stall:** With a small set of "tip" blocks and a finite set of peers, the system quickly reaches a state where **every** peer is on backoff for **every** escalation block. The ingest loop spins (active peers = 0) waiting for backoff timers to expire, while the tail scheduler keeps piling new blocks behind the stuck ones.
*   **Fix:** Make the per-block backoff configuration context-aware. In `Follow` mode, the maximum backoff should be drastically reduced (e.g., 5 seconds) compared to Historical sync (128 seconds), acknowledging that "missing" at the tip is often a temporary propagation state, not a permanent data corruption.

---

### Detailed Trace of the Stall

1.  **Ingest Pipeline Execution (`sync/historical/mod.rs`)**
    *   The node enters `run_ingest_pipeline` (line 538).
    *   It enters the main loop at line 621.
    *   It attempts to fetch blocks by acquiring a semaphore (line 643) and selecting a ready peer (line 648).

2.  **Assignment Request (`sync/historical/scheduler.rs`)**
    *   The loop calls `scheduler.next_batch_for_peer(peer_id, head_cap)` (line 661).
    *   In `scheduler.rs`, `next_batch_for_peer` (line 228) **prioritizes the escalation queue**:
        ```rust
        if let Some(block) = self.pop_escalation_for_peer(peer_id).await { ... }
        ```

3.  **Escalation Check & Backoff (`sync/historical/scheduler.rs`)**
    *   `pop_escalation_for_peer` (line 307) iterates through the difficult blocks.
    *   For each block, it checks `backoff.is_cooling_down(block, peer_id)` (line 325).
    *   **The Bug:** `BlockPeerBackoff` (line 203) uses hardcoded constants:
        *   `BACKOFF_INITIAL` = 500ms.
        *   `BACKOFF_MAX` = 128s.
    *   Because these blocks are at the tip, multiple peers fail them (missing payload). Each failure increases the backoff exponentially.
    *   Once a block has been tried by all available peers recently, `pop_escalation_for_peer` returns `None`.

4.  **Normal Queue Blocked**
    *   `next_batch_for_peer` falls through to `pop_next_batch_for_head` (normal queue).
    *   However, if blocks are in the escalation queue, they are usually the *lowest* block numbers (the ones blocking progress). The system logic often implies we shouldn't skip far ahead, or simply the normal queue is empty because the `tail` is appending blocks that haven't entered `pending` yet or are blocked by the `head_cap` logic if the peer is slightly behind.
    *   Even if the normal queue has blocks, if the peer is considered "cooling down" for the escalation blocks (which it effectively is), the scheduler might yield nothing or the loop might cycle peer-by-peer finding no work.

5.  **The Spin Loop (`sync/historical/mod.rs`)**
    *   Back in `mod.rs`, `batch.blocks.is_empty()` is true (line 665).
    *   The code drops the semaphore permit (line 666).
    *   It spawns a task to sleep 50ms and re-queue the peer (line 668).
    *   **Result:** `peers_active` remains 0. The loop spins rapidly (throttled only by peer re-queue delay). `queue` remains 0 in logs because the metrics might be latching onto the `pending` count (which drains into escalation) or simply because the system is deadlocked waiting for the "stuck" blocks to clear.

---

### Root Cause Analysis

The logic in `sync/historical/scheduler.rs` assumes that a missing block is a rare, semi-permanent failure (common in historical sync). It protects peers from spinning on bad blocks using aggressive exponential backoff (up to 128s).

In **Follow Mode**, a missing block is common and transient (propagation latency). A backoff of 128s is catastrophic for tip-chasing. With 28 peers and 12 failing blocks, the probability of a "total coverage backoff" (where no peer is allowed to try any pending block) approaches 100%.

**Why `queue=0`?**
The tail scheduler adds blocks to `pending`. When they fail 5 times, `requeue_failed` moves them to `escalation`.
In `sync/historical/stats.rs`, the queue metric is updated via `scheduler.pending_count()`, which sums `pending` + `escalation`.
If logs show `queue=0` despite the tail appending, it suggests the blocks are seemingly vanishing or being stuck in `in_flight` state indefinitely without being cleared, OR the log snapshot is misleading. However, the **`peers_active=0`** is the definitive proof of the scheduler refusing to assign work.

---

### Recommended Fix

We need to make the `BlockPeerBackoff` configuration dependent on the sync mode (`FastSync` vs `Follow`). In Follow mode, the max backoff should be short (e.g., 5-10s).

#### 1. Modify `SchedulerConfig` in `sync/historical/scheduler.rs`

Add a field to control the maximum backoff duration for block-peer pairs.

```rust
// sync/historical/scheduler.rs

#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub blocks_per_assignment: usize,
    pub initial_blocks_per_assignment: usize,
    pub max_attempts_per_block: u32,
    pub shard_size: u64,
    // ADD THIS:
    pub max_block_backoff: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            blocks_per_assignment: 32,
            initial_blocks_per_assignment: 32,
            max_attempts_per_block: DEFAULT_ESCALATION_THRESHOLD,
            shard_size: 10_000,
            // ADD THIS:
            max_block_backoff: BACKOFF_MAX, // Default to 128s for safety
        }
    }
}
```

#### 2. Update Backoff Logic in `sync/historical/scheduler.rs`

Update `BlockPeerBackoff::record_failure` to accept the limit from config (you'll need to pass the config or limit down to this method).

*Note: Since `BlockPeerBackoff` is a private struct, you should pass the limit into `record_block_peer_failure` or store it in `BlockPeerBackoff` struct on creation.*

**Preferred approach (Store in Scheduler):**

```rust
// sync/historical/scheduler.rs around line 402

    pub async fn record_block_peer_failure(&self, block: u64, peer_id: PeerId) {
        let mut backoff = self.block_peer_backoff.lock().await;
        // Pass the limit from self.config
        backoff.record_failure(block, peer_id, self.config.max_block_backoff);
    }
```

And update the private method:

```rust
// sync/historical/scheduler.rs around line 203

impl BlockPeerBackoff {
    fn record_failure(&mut self, block: u64, peer_id: PeerId, max_backoff: Duration) {
        let key = (block, peer_id);
        let entry = self.entries.entry(key).or_insert(BlockPeerCooldown {
            fail_count: 0,
            cooldown_until: Instant::now(),
        });
        entry.fail_count = entry.fail_count.saturating_add(1);
        let backoff = if entry.fail_count <= 1 {
            BACKOFF_INITIAL
        } else {
            let factor = 2u32.saturating_pow(entry.fail_count.saturating_sub(1));
            (BACKOFF_INITIAL * factor).min(max_backoff) // USE PARAMETER
        };
        entry.cooldown_until = Instant::now() + backoff;
    }
    // ...
}
```

#### 3. Configure Pipeline in `sync/historical/mod.rs`

Update `run_ingest_pipeline` to set this config based on the DB mode.

```rust
// sync/historical/mod.rs around line 563

    let mut scheduler_config = SchedulerConfig {
        blocks_per_assignment: max_blocks,
        initial_blocks_per_assignment: initial_blocks,
        // Set backoff based on mode
        max_block_backoff: if db_mode == DbWriteMode::Follow {
            Duration::from_secs(5) // Retry frequently in follow mode
        } else {
            Duration::from_secs(120) // Conservative in historical sync
        },
        ..SchedulerConfig::default()
    };
```