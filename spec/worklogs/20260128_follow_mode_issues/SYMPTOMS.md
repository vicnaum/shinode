# Follow Mode Issues - Symptoms

Date: 2026-01-28

## Observed Issues

### 1. Logging Problems

#### 1.1 Delayed Log Writes
- Logs are written with significant delay (not immediately)
- Buffered writes reduce real-time debugging ability
- Current buffer: 4096 records before flush

#### 1.2 Truncated Logs on Quit
- When pressing 'q' to quit, logs are truncated mid-sentence
- Example from `2026-01-28__14-28-15.logs.jsonl.part`:
  ```
  {"t_ms":225263
  ```
- `finish()` not being called on graceful shutdown, or channel closes before all records processed

### 2. Follow Mode Instability

#### 2.1 Incorrect "SYNCED" Status
- TUI shows "SYNCED" when actually 152 seconds behind
- Then switches to "CATCHING UP" with 90+ blocks behind
- Status oscillates between synced and catching up

#### 2.2 Slow Progress
- Active peers: only 1-2 out of 13 connected
- Chain head advancing but node not keeping up
- Long periods with no visible progress

#### 2.3 Long Sealing Phase
- Screenshot shows stuck at "Sealing" phase for extended time
- 1/1 shards sealed but phase doesn't transition

### 3. Screenshots Analysis

#### Screenshot 1: Catching Up (122 blocks behind)
- Phase: Not visible
- Status: "CATCHING UP"
- 122 blocks behind

#### Screenshot 2: Sealing Phase
- Phase: SEAL
- Sealing: 1/1 shards sealed (100%)
- Peers: 2 active, 3 connected
- Chain Head: 24,333,942
- Logs show: "follow: ingest epoch" at 14:29:00
- RPC server started at 14:33:52 (4+ minutes after follow started)

#### Screenshot 3: Following Phase
- Phase: FOLLOW
- Status: "CATCHING UP" - 60 blocks behind
- Peers: 1 active, 13 connected
- Chain Head: 24,334,008
- Block number displayed: 24,333,948
- RPC: Active, 0 requests

## Key Questions

1. Why only 1-2 active peers when 13 are connected?
2. Why does sealing take so long?
3. Why does status show "SYNCED" when behind?
4. Why aren't logs flushed on graceful quit?
5. Why is there a 4+ minute gap between follow start and RPC server start?

## Files to Investigate

1. `node/src/logging/json.rs` - Log buffering and flush logic
2. `node/src/sync/historical/follow.rs` - Follow mode logic
3. `node/src/ui/tui/mod.rs` - TUI status display logic
4. `node/src/run/sync_runner.rs` - Shutdown and cleanup handling

---

## Session 2 Observations (2026-01-28 ~16:00)

### New Issue: Follow Mode Permanently Behind

#### Test Run 1: `2026-01-28__15-16-14`
After applying fixes for logging and TUI issues, observed:
- Fast-sync completed successfully (dirty_shards=0)
- Follow mode started: `range_start=24334114, range_end=24334178, missing_blocks=65`
- DB writer waiting for `expected=24334126`
- But only receiving tail blocks: 24334179, 24334180, 24334181...
- **Initial missing blocks (24334126-24334178) never fetched!**
- Node permanently stuck, reorder buffer growing indefinitely

#### Test Run 2: `2026-01-28__15-42-37` (trace logging)
- Ran with `--log-json-filter "warn,stateless_history_node=trace"`
- Node was 19 blocks behind when stopped
- Sometimes synced momentarily, then fell behind again
- Need to analyze trace logs for scheduler behavior

### Key Log Evidence

From `2026-01-28__15-16-14.logs.jsonl.part`:
```json
{"t_ms":28916,"message":"follow: ingest epoch","fields":{"missing_blocks":65,"range_start":24334114,"range_end":24334178}}
{"t_ms":38919,"message":"follow reorder buffer","fields":{"buffer_len":53,"expected":24334126,"max_key":24334179}}
{"t_ms":48919,"message":"follow reorder buffer","fields":{"buffer_len":54,"expected":24334126,"max_key":24334180}}
```

The `expected` block (24334126) never changes - it's never being fetched.

### TUI Behavior After Fixes
- TUI now correctly transitions from SEAL to FOLLOW (fix confirmed)
- But follow mode itself is broken - not fetching backfill blocks

### Hypothesis
The scheduler receives initial missing blocks in constructor but:
1. Either doesn't add them to the work queue
2. Or tail-scheduled blocks take priority
3. Or peer selection only picks peers for new blocks, not backfill

### Files to Investigate (Updated)
1. `node/src/sync/historical/scheduler.rs` - `PeerWorkScheduler::new_with_health()`, work queue logic
2. `node/src/sync/historical/mod.rs` - Scheduler creation at line 405-409
3. `node/src/sync/historical/mod.rs` - `next_batch_for_peer()` at line 721-723
