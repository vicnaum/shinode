### Summary
The node is stuck in a **high-frequency retry loop** for a single block (24334255), effectively performing a denial-of-service attack on itself and its peers. 

1. **Isolation**: Block 24334255 became isolated from subsequent blocks (which were successfully fetched), forcing the scheduler to request it in a **batch of size 1**.
2. **Infinite Spin**: The `requeue_failed` logic has **zero delay/backoff**, and "Follow Mode" disables the escalation queue (which tracks peer cooldowns).
3. **Rate Limiting**: Sending thousands of single-header requests (`GetBlockHeaders(N, 1)`) likely triggers peer spam protection/rate-limiting, causing them to drop the requests or return empty responses, which the node interprets as "missing", perpetuating the loop.

### Suspected Root Causes

1.  **Lack of Retry Backoff (Critical)**
    In `sync/historical/fetch_task.rs`, when a fetch fails or returns partial data, `requeue_blocks` is called immediately.
    In `sync/historical/scheduler.rs`, `requeue_failed` pushes the block directly back onto the `pending` heap with no delay.
    ```rust
    // sync/historical/scheduler.rs:693
    pub async fn requeue_failed(&self, blocks: &[u64]) -> Vec<u64> {
        // ...
        // Immediate re-insertion into priority queue
        if !queued.contains(block) {
            queued.insert(*block);
            pending.push(Reverse(*block)); 
        }
        // ...
    }
    ```
    Since 24334255 is the lowest block number, it is immediately popped again by the next available worker in `pop_next_batch_for_head` (Line 658), creating a tight spin loop.

2.  **Escalation Logic Disabled in Follow Mode**
    The `EscalationState` in `scheduler.rs` has logic to track peer attempts and enforce cooldowns (`peer_attempts` map). However, this protection is bypassed in Follow Mode because `max_attempts_per_block` is set to `u32::MAX`.
    ```rust
    // sync/historical/mod.rs:373
    if db_mode == DbWriteMode::Follow {
        scheduler_config.max_attempts_per_block = u32::MAX;
    }
    ```
    This prevents the block from ever moving to the escalation queue, which is the only place where `ESCALATION_PEER_COOLDOWN` (30s) is enforced.

3.  **Single-Block Batching**
    Because subsequent blocks (24334256+) were successfully fetched, block 24334255 is isolated. The scheduler's consecutiveness check forces it to be a batch of 1.
    ```rust
    // sync/historical/scheduler.rs:666
    if let Some(prev) = last {
        if next != prev.saturating_add(1) {
            break; // Batch ends if gap found
        }
    }
    ```
    Peers are much more likely to rate-limit or deprioritize tiny, high-frequency header requests compared to standard batches.

### Recommended Fixes

1.  **Implement Backoff:** Modify `requeue_failed` in `scheduler.rs` to accept a delay, or handle the delay in `fetch_task.rs` before calling requeue.
    ```rust
    // sync/historical/fetch_task.rs
    async fn requeue_blocks(...) {
        sleep(Duration::from_secs(1)).await; // Simple fix
        // ...
    }
    ```

2.  **Enable Escalation in Follow Mode:** Instead of `u32::MAX` attempts, use a reasonable limit (e.g., 10) even in follow mode, so stubborn blocks move to the escalation queue where `pop_escalation_for_peer` enforces peer cooldowns.

3.  **Peer Rotation:** Explicitly track which peer failed a block and avoid assigning the same block to the same peer immediately in the `Normal` queue, similar to how `EscalationState` works.