# Follow Mode Issues - Investigation

## Issue 1: Truncated Logs on Quit

### Root Cause Found
**File:** `node/src/ui/progress.rs` lines 534-548

When user presses 'q' to quit:
```rust
if should_quit {
    // Wait for completion or timeout (3 seconds)
    if let Some(rx) = completion_rx.take() {
        let _ = tokio::time::timeout(
            Duration::from_secs(3),
            rx,
        ).await;
    }

    // Restore terminal and force exit
    let _ = tui.lock().restore();
    std::process::exit(0);  // <-- BUG: doesn't run destructors!
}
```

`std::process::exit(0)` immediately terminates the process without running destructors. This means:
- `JsonLogWriter.finish()` is never called
- Buffered log data (up to 4096 records) is lost
- Files are truncated mid-write

### Fix Required
Before calling `process::exit(0)`, need to:
1. Flush all log writers (call `finish()` on each)
2. Or use a proper return path that allows destructors to run

---

## Issue 2: Delayed Log Writes

### Current Behavior
**File:** `node/src/logging/json.rs` lines 96-108

```rust
let handle = std::thread::spawn(move || -> eyre::Result<()> {
    let mut since_flush = 0usize;
    for record in rx {
        serde_json::to_writer(&mut writer, &record)?;
        writer.write_all(b"\n")?;
        since_flush = since_flush.saturating_add(1);
        if since_flush >= 4096 {  // <-- Only flush every 4096 records
            writer.flush()?;
            since_flush = 0;
        }
    }
    writer.flush()?;
    Ok(())
});
```

### Problem
- Logs are only flushed every 4096 records
- For real-time debugging, this delay is too long
- Combined with the quit bug, can lose up to 4096 records

### Possible Fixes
1. Flush more frequently (every 100-500 records)
2. Add periodic time-based flush (every 1-5 seconds)
3. Add a `flush()` method for on-demand flushing

---

## Issue 3: Follow Mode Shows "SYNCED" When Behind

### Observations from Screenshots
- TUI shows "SYNCED" status
- But actually 152 seconds behind
- Then switches to "CATCHING UP" with 90+ blocks behind
- Only 1-2 active peers out of 13 connected

### Investigation Needed
1. Check how "SYNCED" status is determined in follow.rs
2. Check TUI state update logic
3. Check head tracking accuracy

### Key Code Locations
- `node/src/sync/historical/follow.rs` - `SyncStatus::UpToDate` / `SyncStatus::Following`
- `node/src/ui/tui.rs` - Status display logic
- `node/src/run/trackers.rs` - Head tracking

---

## Issue 4: Long Sealing Phase

### Observation
- Screenshot shows stuck at "Sealing" for extended time
- 1/1 shards sealed at 100% but phase doesn't transition

### Investigation Needed
- Check sealing flow in `db_writer.rs`
- Check phase transition logic in `sync_runner.rs`
- Check if TUI phase updates are out of sync

---

## Issue 5: Low Active Peers

### Observation
- 13 peers connected
- Only 1-2 active (doing work)
- Most peers idle

### Possible Causes
1. Peers at lower head than needed
2. Quality scoring excluding peers
3. Work queue exhausted (already caught up)
4. Scheduling bug

### Related Code
- `node/src/sync/historical/scheduler.rs` - Peer selection
- `node/src/sync/historical/mod.rs` - `pick_best_ready_peer_index()`

---

## Issue 6: "Last block Xs ago" Not Updating

### Root Cause
`last_block_received_ms` is only set in `fetch_task.rs:259` when blocks are completed during fetch tasks.

In follow mode, if:
1. We're caught up and waiting for new blocks
2. Or if fetches are failing/stalling

The timestamp won't update, so "last block Xs ago" keeps increasing even if we think we're synced.

### Fix Needed
Consider updating `last_block_received_ms` based on head tracking, not just fetch completion. When `head_seen` equals `our_head`, we're synced and the timestamp should reflect that.

---

## Summary of Issues Found

### Critical (data loss)
1. **Log truncation on quit** - `std::process::exit(0)` bypasses destructors

### High Priority
2. **Delayed log writes** - 4096 record buffer is too large for debugging
3. **False "SYNCED" status** - `is_synced()` checks `our_head >= chain_head` but `chain_head` may be stale
4. **"Last block Xs ago"** - Only updates on fetch completion, not when caught up

### Medium Priority
5. **Long sealing phase** - Need to investigate if actual delay or UI state lag
6. **Low active peers** - Only 1-2 of 13 peers working

---

## Fixes Applied (2026-01-28 Session 2)

### Issue 1: Log Truncation on Quit - FIXED
**Files modified:**
- `node/src/ui/progress.rs` - Added `log_writer` and `resources_writer` params to `spawn_tui_progress_updater`, flush before `process::exit(0)`
- `node/src/run/startup.rs` - Added `LogWriters` struct, passed writers to `setup_ui`
- `node/src/run/sync_runner.rs` - Pass `tracing_guards.log_writer` and `resources_writer` to setup

### Issue 2: Delayed Log Writes - FIXED
**File:** `node/src/logging/json.rs`
- Changed writer thread to use `recv_timeout(500ms)` instead of blocking `recv`
- Added time-based flush: every 2 seconds OR every 4096 records
- Constants: `FLUSH_COUNT=4096`, `FLUSH_INTERVAL=2s`, `RECV_TIMEOUT=500ms`

### Issue 3: False "SYNCED" Status - PARTIALLY FIXED
**File:** `node/src/ui/tui.rs`
- Added `FOLLOW_STALENESS_THRESHOLD_MS = 30_000` constant
- Added `last_block_received_ms` field to `TuiState`
- Updated `is_synced()` to check for staleness in follow mode

### Issue 4: TUI Stuck on SEAL Phase - FIXED
**Root Cause:** Backwards phase transition prevention was too aggressive.
- When fast-sync SEAL phase completed and follow mode started fetching, status became `Fetching`
- `Fetching` mapped to `Phase::Sync` (ordinal 1)
- But `Sync (1) < Seal (4)`, so transition was blocked!
- TUI stayed stuck on SEAL forever

**Fix in `node/src/ui/tui.rs`:**
- When at SEAL or later and status is `Fetching`, map to `Phase::Follow` instead of `Phase::Sync`
- Allow transitions to `Phase::Follow` even if it looks like backwards transition

---

## Issue 7: Follow Mode Backfill Bug (NEW - CRITICAL)

### Symptoms
From logs `2026-01-28__15-16-14.logs.jsonl.part`:
- Follow ingest epoch starts with `missing_blocks=65, range_start=24334114, range_end=24334178`
- DB writer shows `expected=24334126` (first missing block after already-present ones)
- Reorder buffer keeps growing: `buffer_len=54, 55, 56...` with `min_key=24334127, max_key=24334180+`
- **Block 24334126 NEVER arrives!**

### Root Cause Analysis
1. Follow mode calls `run_ingest_pipeline` with `MissingBlocks::Precomputed(blocks)`
2. The `blocks` vec should contain missing blocks [24334126, 24334127, ..., 24334178]
3. Scheduler is initialized with these blocks at line 405-409 of `mod.rs`
4. BUT only tail blocks (24334179+) are being fetched
5. The initial missing blocks (24334126-24334178) are NOT being scheduled to peers

### Evidence
```
"tail: appended range" logs show: 24334179, 24334180, 24334181, 24334182...
```
These are NEW head blocks from the tail scheduler, not the original missing blocks.

### Suspected Cause
The scheduler may not be properly processing the initial `blocks` vec, or peer selection is only picking up tail-scheduled work. The `PeerWorkScheduler::new_with_health()` receives the blocks but they may not be getting into the work queue correctly.

### Key Code Locations
- `node/src/sync/historical/mod.rs:405-409` - Scheduler creation with initial blocks
- `node/src/sync/historical/scheduler.rs` - `PeerWorkScheduler::new_with_health()`
- `node/src/sync/historical/mod.rs:424-459` - Tail task adding new ranges

### Additional Issue: Storage Writer Error
At t=30876ms, reorg check fails with:
```
"reorg check failed" with error "failed to open static segment writer"
```
This is strange because reorg check only READS, shouldn't open writers. May be concurrent access issue between:
- Reorg detector (reading headers)
- Ingest pipeline (writing blocks)
Both accessing the same shard (24330000).

---

## Latest Log Files

| Timestamp | Description |
|-----------|-------------|
| `2026-01-28__15-16-14` | First follow mode test after fixes, showed backfill bug |
| `2026-01-28__15-39-15` | Short run |
| `2026-01-28__15-42-37` | Latest trace-level run, 19 blocks behind before stopped |

---

## Issue 7: Deep Dive (Session 3 - Trace Log Analysis)

### Test Run: `2026-01-28__15-42-37` (trace-level logging)

**Timeline:**
- `t=25552ms`: Fast-sync completes, `switch_tip=24334245`, `head_seen=24334309`
- `t=26619ms`: Follow mode starts with `missing_blocks=65, range_start=24334246, range_end=24334310`
- `t=26765ms`: First "missing 1 block" message from peer
- `t=26765ms-339494ms`: **16,483 consecutive "missing 1 block" retry attempts** (5+ minutes!)

### Key Evidence from Logs

**Reorder Buffer State** (consistent across all checks):
```json
{"expected":24334255,"min_key":24334256,"max_key":24334310+,"buffer_len":55-57}
```

**Pattern Analysis:**
- `expected=24334255` - DB writer is waiting for this specific block
- `min_key=24334256` - First block actually IN the reorder buffer
- Block 24334255 is NEVER in the buffer
- 55+ blocks (24334256-24334310+) ARE arriving successfully

### Root Cause Identified (FINAL - confirmed by Gemini analysis)

**Self-reinforcing retry spin loop.** See `Gemini_Answer1.md` and `FIX_PLAN.md`.

The block failed ONCE due to normal P2P flakiness, then got trapped:
1. Block isolated (surrounding blocks succeeded) → forced batch-of-1
2. `requeue_failed()` has zero delay → immediate retry
3. `max_attempts_per_block = u32::MAX` → escalation (with cooldowns) never triggered
4. Tight loop sends thousands of `GetBlockHeaders(N,1)` → peers rate-limit → always "missing"

**Fix plan in `FIX_PLAN.md`**: Add backoff delay + lower escalation threshold.

---

### Earlier Analysis (partially correct - scheduler IS working, but the retry loop is the bug)

**NOT a scheduler bug.** The scheduler is working correctly:
1. Scheduler assigns batch starting from block 24334255 (first pending block)
2. Peer receives request for headers 24334255-24334310
3. Peer returns headers 24334256-24334310 (55 headers) **BUT NOT 24334255**
4. Code marks block 24334255 as "missing" and requeues it
5. Successfully returned blocks go to DB writer's reorder buffer
6. Repeat 16,483 times...

**The PEER is not returning block 24334255.** This is likely:
1. **Peer data gap**: Peer doesn't have block 24334255 (propagation lag or corruption)
2. **Single peer dominance**: Same peer (`0x9d8394...`) handles all requests; no rotation to other peers
3. **AIMD feedback loop**: The peer's batch limit gets reduced on each partial response, but it keeps getting selected due to quality scoring

### Supporting Evidence

**Fetch failure pattern:**
```
t=26765ms: "ingest batch missing headers or payloads" {missing:1, peer_id:"0x9d8394..."}
t=26910ms: "ingest batch missing headers or payloads" {missing:1, peer_id:"0x9d8394..."}
... (16,483 times total)
```

All failures are from the SAME peer (`0x9d8394...`).

**Other peer banned:**
```json
{"t_ms":88732,"message":"peer banned after consecutive failures","failures":5,"peer_id":"0xc90b8a8..."}
```
The other peer got banned for unrelated channel errors, leaving only the problematic peer.

### Why This Causes Permanent Stall

1. DB writer waits for blocks IN ORDER (expected=24334255)
2. Block 24334255 never arrives
3. Reorder buffer grows with blocks 24334256, 24334257, etc.
4. Head keeps advancing (tail scheduler adds 24334311, 24334312, etc.)
5. Gap never closes → node permanently behind

### Code Path

**Key files:**
- `node/src/p2p/mod.rs:652-678` - `fetch_payloads_for_peer()` builds `missing_blocks` from header response gaps
- `node/src/sync/historical/fetch_task.rs:288-322` - Handles partial responses, calls `requeue_blocks()`
- `node/src/sync/historical/scheduler.rs:641-675` - `pop_next_batch_for_head()` only pops consecutive blocks

**The single-block gap breaks the consecutive batch logic:**
```rust
// scheduler.rs pop_next_batch_for_head()
if let Some(prev) = last {
    if next != prev.saturating_add(1) {
        break;  // Can't add non-consecutive blocks to batch
    }
}
```

Since block 24334255 is always requeued and is the smallest pending block, it's always at the front of the queue. The scheduler correctly keeps trying to fetch it first.

### Potential Fixes

1. **Peer rotation on repeated single-block failures**
   - If the same block fails N times from the same peer, try a different peer
   - Track per-block per-peer failure counts

2. **Escalation queue enhancement**
   - Current escalation threshold (`max_attempts_per_block`) is `u32::MAX` in follow mode
   - Need a way to escalate "stuck" blocks to try other peers sooner

3. **Multi-peer parallel fetch for stuck blocks**
   - When a block is stuck, fan out requests to multiple peers simultaneously
   - First response wins

4. **Better peer health tracking for partial responses**
   - Mark peers that consistently return partial batches as "unreliable for backfill"
   - Prefer other peers for historical blocks

---

## Next Steps

1. ~~Fix logging bugs~~ - DONE
2. ~~Fix TUI stuck on SEAL~~ - DONE
3. ~~Fix false SYNCED status~~ - PARTIALLY DONE (staleness check added)

4. **Fix single-block stall bug** (CRITICAL)
   - Implement peer rotation for stuck blocks
   - Or: reduce escalation threshold in follow mode
   - Or: add per-block-per-peer failure tracking

5. **Investigate storage writer error during reorg check**
   - Why does a read operation get a "failed to open static segment writer" error?
   - Check for concurrent access issues between reorg detector and ingest pipeline

6. **Test with more peers**
   - Current test only had 2 peers, one got banned
   - With more healthy peers, peer rotation might naturally resolve the issue
