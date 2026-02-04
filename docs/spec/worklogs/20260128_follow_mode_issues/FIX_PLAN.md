# Fix Plan: Follow Mode Single-Block Spin Loop

Date: 2026-01-28 (updated after discussion)

## Problem Summary

When a single block fails to fetch (even once, due to normal P2P flakiness), it gets trapped in a **self-reinforcing retry spin loop**:

1. Block N fails once (normal P2P behavior - partial response)
2. Surrounding blocks succeed → block N becomes **isolated** (only pending block in that range)
3. Scheduler's consecutiveness check forces **batch size 1** for block N
4. `requeue_failed()` has **zero delay** → immediate retry
5. `max_attempts_per_block = u32::MAX` in follow mode → escalation queue (which has cooldowns) is **never triggered**
6. Tight loop sends thousands of `GetBlockHeaders(N, 1)` per minute
7. Peers **rate-limit** the spam → empty/partial responses → "missing" → requeue → repeat 16,000+ times

**Evidence**: Block 24334255 failed 16,483 times across 5 different peers over 5+ minutes. ~145ms between retries = single-header round-trip time.

## Root Cause Analysis (confirmed by Gemini + log analysis)

See `Gemini_Answer1.md` for Gemini's full analysis.

### Three interacting bugs:

**Bug 1: No retry backoff** (`scheduler.rs`)
```rust
// scheduler.rs - requeue_failed()
// Block is pushed directly back to pending queue with NO delay
if !queued.contains(block) {
    queued.insert(*block);
    pending.push(Reverse(*block));  // Immediately available for next pop
}
```

**Bug 2: Escalation disabled in follow mode** (`mod.rs`)
```rust
// mod.rs line 373
if db_mode == DbWriteMode::Follow {
    scheduler_config.max_attempts_per_block = u32::MAX;  // Never escalates!
}
```
The escalation queue is the ONLY place with peer cooldowns (`ESCALATION_PEER_COOLDOWN = 30s`). Setting `max_attempts_per_block = u32::MAX` means blocks never reach it.

**Bug 3: Single-block batching amplifies the problem** (`scheduler.rs`)
```rust
// scheduler.rs - pop_next_batch_for_head()
if let Some(prev) = last {
    if next != prev.saturating_add(1) {
        break;  // Can't add non-consecutive blocks to batch
    }
}
```
Once a block is isolated (surrounding blocks already fetched), it can only be fetched as batch-of-1, which is more likely to be rate-limited by peers.

**Note on Bug 3**: Non-consecutive batching is NOT possible at the protocol level. ETH P2P `GetBlockHeaders` takes `(start_block, limit, skip, direction)` — you cannot request arbitrary block numbers like `[100, 105, 200]` in one message. So single-block requests for isolated blocks are unavoidable. This makes the backoff fix even more critical.

---

## Fix Plan (Revised after discussion)

### Design Principles (from discussion)

1. **No hard bans** — replace the current "5 failures → 120s ban" with exponential backoff everywhere (both fast-sync AND follow mode)
2. **Per-peer-per-block tracking in follow mode** — a peer failing on block N should not prevent it from being assigned block N+1
3. **Don't wait on the block** — when a peer is cooling down for a specific block, immediately try the next available peer for that block
4. **Peer selection prefers freshness** — prefer peers we haven't tried for this block, then peers whose cooldown expired longest ago
5. **Reset on success** — any successful response from a peer resets that peer's cooldowns

### Fix 1: Replace hard peer bans with exponential backoff (BOTH modes)

**File:** `node/src/sync/historical/scheduler.rs`

**Current behavior** (in `PeerHealthState`):
```rust
// 5 consecutive failures → 120s hard ban
if entry.consecutive_failures >= self.config.peer_failure_threshold {
    entry.banned_until = Some(Instant::now() + self.config.peer_ban_duration);
}
```

**New behavior**: Replace `banned_until: Option<Instant>` with exponential backoff:
```rust
struct PeerHealth {
    // Remove: banned_until: Option<Instant>,
    // Add:
    backoff_until: Option<Instant>,
    backoff_duration: Duration,  // current backoff level, doubles on each failure
    // Keep existing: consecutive_failures, consecutive_partials, successes, etc.
}
```

**Backoff schedule**: 0.5s → 1s → 2s → 4s → 8s → 16s → 32s → 64s → 128s (cap)

**On failure** (`record_peer_failure`):
- Double `backoff_duration` (starting from 0.5s, cap at 128s)
- Set `backoff_until = now + backoff_duration`
- Peer is still selectable for work — but `is_banned()` (rename to `is_cooling_down()`) returns true until `backoff_until`

**On success** (`record_peer_success`):
- Reset `backoff_duration` to 0
- Clear `backoff_until`
- (Same as current: reset `consecutive_failures`, etc.)

**On partial** (`record_peer_partial` — used in follow mode):
- Don't penalize or backoff - partials are normal, just adjust batch limit for that peer as we already do.

**Peer selection** (`pick_best_ready_peer_index` or equivalent):
- Skip peers where `now < backoff_until`
- Among available peers, prefer by quality score (existing logic)

**Constants:**
```rust
const BACKOFF_INITIAL: Duration = Duration::from_millis(500);
const BACKOFF_MAX: Duration = Duration::from_secs(128);
```

### Fix 2: Per-peer-per-block backoff in follow mode (CRITICAL)

**File:** `node/src/sync/historical/scheduler.rs`

This is the core fix for the spin loop. In follow mode, we need per-peer-per-block tracking so that when peer A fails on block N, we immediately try peer B instead of waiting.

**New state in scheduler** (only active in follow mode):
```rust
/// Per-peer-per-block failure tracking for follow mode.
/// Prevents spin-looping on a single block by tracking which peers
/// failed on which blocks and applying per-peer cooldowns.
struct BlockPeerBackoff {
    /// Map of (block_number, peer_id) → backoff state
    entries: HashMap<(u64, PeerId), PeerBlockBackoffEntry>,
}

struct PeerBlockBackoffEntry {
    fail_count: u32,
    cooldown_until: Instant,
}
```

**Backoff schedule per peer per block**: 0.5s → 1s → 2s → 4s → ... → 128s (cap)

**Integration with peer selection for a block**:
1. Check `BlockPeerBackoff` for this block number
2. Prefer peers that have NOT been tried for this block
3. Among tried peers, prefer the one whose cooldown expired longest ago
4. If ALL peers are still cooling down for this block, wait until the earliest cooldown expires
5. On success from ANY peer: remove all entries for this block (block is done)
6. On success from a peer on ANY block: do NOT reset entries for other blocks (per-peer-per-block is independent)

**Where to integrate** — in `next_batch_for_peer()` or a new method:
- When popping a block from normal queue, check if the assigned peer has a cooldown for that block
- If yes, skip this peer for this block (put block back, try another peer or another block)
- The escalation queue already has per-peer tracking (`peer_attempts` map with `ESCALATION_PEER_COOLDOWN`). The new `BlockPeerBackoff` replaces this concept for the normal queue.

**Cleanup**: Entries for completed blocks should be removed. When a block is marked complete, remove all entries for that block from `BlockPeerBackoff`.

### Fix 3: Enable escalation in follow mode with lower threshold

**File:** `node/src/sync/historical/mod.rs`

Change `max_attempts_per_block` from `u32::MAX` to a reasonable threshold:

```rust
if db_mode == DbWriteMode::Follow {
    scheduler_config.max_attempts_per_block = 5;
}
```

After 5 total failures (across all peers), the block moves to the escalation queue. The escalation queue already has:
- Per-peer cooldown tracking (`ESCALATION_PEER_COOLDOWN = 30s`)
- Shard-aware prioritization
- Single-block granularity

**Important change to escalation behavior**: The current escalation queue applies a 30s cooldown per peer per block, which means if we have 6 peers it takes 30s × 6 = 180s to try all peers once. This is too slow.

**Update `ESCALATION_PEER_COOLDOWN`**: With the new per-peer-per-block exponential backoff (Fix 2), the escalation queue's fixed 30s cooldown becomes redundant.
Fix:
- a) Remove the fixed cooldown from escalation and use the same exponential backoff — unify backoff logic. The `BlockPeerBackoff` from Fix 2 should be the single source of truth for per-peer-per-block cooldowns, used by both normal and escalation queues.

---

## Files to Modify

| File | Change |
|------|--------|
| `node/src/sync/historical/scheduler.rs` | (1) Replace hard bans with exponential backoff in `PeerHealthState`. (2) Add `BlockPeerBackoff` for per-peer-per-block tracking. (3) Update peer selection to check per-block cooldowns. (4) Unify escalation cooldown with `BlockPeerBackoff`. |
| `node/src/sync/historical/mod.rs` | Change `max_attempts_per_block` from `u32::MAX` to `5` in follow mode |
| `node/src/sync/historical/fetch_task.rs` | Record per-peer-per-block failures when missing blocks detected |

## Key Code Locations

| Location | What It Does |
|----------|-------------|
| `scheduler.rs:~90-91` | `peer_failure_threshold: 5`, `peer_ban_duration: 120s` — replace with backoff |
| `scheduler.rs:~130-134` | `PeerHealth` struct — replace `banned_until` with backoff fields |
| `scheduler.rs:~155-158` | `is_banned()` — rename to `is_cooling_down()`, use backoff |
| `scheduler.rs:~257-271` | `record_peer_failure()` — change from ban to exponential backoff |
| `scheduler.rs:~227-232` | `record_peer_success()` — reset backoff |
| `scheduler.rs:~693` | `requeue_failed()` — consider adding block+peer info |
| `scheduler.rs:~682` | `pop_escalation_for_peer()` — unify cooldown with `BlockPeerBackoff` |
| `scheduler.rs:~15` | `ESCALATION_PEER_COOLDOWN = 30s` — may be removed/unified |
| `mod.rs:~373` | `max_attempts_per_block = u32::MAX` — change to `5` |
| `fetch_task.rs:~288-322` | Missing blocks handler — record per-peer-per-block failure |

### Fix 4: Remove `fast_sync_batch_timeout_ms` (cleanup)

The outer fetch timeout (10s wrapping the entire `fetch_ingest_batch` call) is redundant. Individual P2P requests already have `REQUEST_TIMEOUT = 4s` each. The outer timeout adds no value — a batch of N chunks where each has a 4s timeout will naturally take up to `N * 4s`, and the 10s outer timeout would incorrectly kill legitimate multi-chunk fetches.

**Remove completely from all levels:**

| File | What to remove |
|------|---------------|
| `node/src/cli/mod.rs:206` | `pub fast_sync_batch_timeout_ms: u64` field + `#[arg]` attribute |
| `node/src/cli/mod.rs:17` | `DEFAULT_FAST_SYNC_BATCH_TIMEOUT_MS` constant |
| `node/src/cli/mod.rs:315` | Log line referencing it |
| `node/src/test_utils.rs:68` | Field in test config |
| `node/src/sync/historical/mod.rs:617` | `let fetch_timeout = ...` line |
| `node/src/sync/historical/mod.rs:773` | `fetch_timeout` passed to `FetchTaskContext` |
| `node/src/sync/historical/fetch_task.rs:30` | `pub fetch_timeout: Duration` field in `FetchTaskContext` |
| `node/src/sync/historical/fetch_task.rs:97-99` | `timeout()` wrapper around `fetch_future` |
| `node/src/sync/historical/fetch_task.rs:130-174` | Entire `handle_fetch_timeout()` function |
| `node/src/sync/historical/fetch_task.rs:156` | `BenchEvent::FetchTimeout` usage |
| `node/src/sync/historical/reorg.rs:192` | Field in reorg test config |
| `node/src/storage/sharded/mod.rs:2165` | Field in storage test config |
| `node/src/sync/historical/AGENTS.md:56,66` | Documentation references |

Also check if `BenchEvent::FetchTimeout` variant can be removed from the bench/events types.

---

## Implementation Order

1. **Fix 4 first** (remove `fast_sync_batch_timeout_ms`) — pure cleanup, simplifies the code before other changes
2. **Fix 1 next** (exponential backoff replacing bans) — straightforward refactor, affects both modes
3. **Fix 2 next** (per-peer-per-block tracking) — the core spin loop fix
4. **Fix 3 last** (lower escalation threshold) — simple constant change + unify cooldowns

## Verification

1. Build: `cargo build --manifest-path node/Cargo.toml`
2. Run tests: `cargo test --manifest-path node/Cargo.toml`
3. Integration test: Run node in follow mode with `--log --log-json-filter "warn,stateless_history_node=trace"`
4. Check logs for:
   - No more rapid-fire "missing 1" messages
   - Peer rotation visible: different peers trying the same stuck block
   - Backoff delays visible: increasing gaps between retries for the same peer+block
   - Node eventually syncing past previously-stuck blocks
5. Fast-sync test: Run a fresh sync for a small range to verify exponential backoff doesn't hurt throughput
   - Transient failures should recover quickly (0.5s initial backoff)
   - Persistent failures should backoff gracefully instead of hard-banning

## Additional Investigation (separate from this fix)

1. **WAL/bitset sync issue**: `missing_blocks_in_range()` checks bitset which doesn't include WAL data. This means blocks written to WAL during fast-sync will be re-requested in follow mode. If one of these re-fetches fails, it could seed the spin loop. Consider checking WAL presence in `missing_blocks_in_range()`.

2. **Storage writer error during reorg check**: "failed to open static segment writer" error during a read-only reorg check. Possible concurrent access issue between reorg detector and ingest pipeline on the same shard.

---

## Previously Fixed Issues (Session 1-2)

These are already implemented and working:

1. **Log truncation on quit** - FIXED: Flush log writers before `process::exit(0)` in `progress.rs`
2. **Delayed log writes** - FIXED: Time-based flushing every 2s in `json.rs`
3. **False SYNCED status** - FIXED: Staleness check in `tui.rs`
4. **TUI stuck on SEAL** - FIXED: Phase transition logic in `tui.rs`
