# UI Progress Bars Analysis Report

## Overview

The UI system has **4 distinct visual elements** and **5 status states**. The code is spread across `main.rs` (display logic) and `sync/mod.rs` + `sync/historical/*.rs` (status updates).

---

## 1. Status States (SyncStatus enum)

Defined in `sync/mod.rs:21-28`:

| Status | Value | Text Display | When Set |
|--------|-------|--------------|----------|
| `LookingForPeers` | 0 (default) | `looking_for_peers` | When head probe fails during follow mode (`follow.rs:161`) |
| `Fetching` | 1 | `fetching` | When downloading blocks (`mod.rs:201`, `mod.rs:302`) |
| `Finalizing` | 2 | `finalizing` | After fetch completes, before DB flush (`mod.rs:956`) |
| `UpToDate` | 3 | `up_to_date` | When no work needed (`mod.rs:186`, `follow.rs:184`) |
| `Following` | 4 | `following` | When caught up in follow mode (`mod.rs:605`) |

---

## 2. UI Elements

### Element A: Yellow Startup Bars (`main.rs:528-554`)

Simple colored text bars printed to stderr during startup phases:
- **"Opening storage..."** (`main.rs:901`)
- **"Compacting N shards..."** (`main.rs:910`) - only if dirty shards exist
- **"Connecting to P2P network..."** (`main.rs:923`)
- **"Waiting for peers... X/Y"** (`main.rs:566`) - only if `min_peers > 0`
- **"Discovering chain head... N"** (`main.rs:1478`)

These use `print_status_bar()` and `clear_status_bar()` - stateless text output.

### Element B: Cyan/Blue Main Progress Bar (`main.rs:1005-1011`)

The main sync progress bar during initial historical sync:
```
[████████████████████████████░░░░░░░░░░░] 65% 6500000/10000000 | 01:23:45 | status fetching | peers 5/12 | queue 1000 | inflight 50 | failed 0 | speed 1234.5/s | eta 45m 30s
```

- Template: `{bar:40.cyan/blue} {percent:>3}% {pos}/{len} | {elapsed_precise} | {msg}`
- Created at `main.rs:1005-1011`, updated by `spawn_progress_updater()`
- The `{msg}` portion is formatted by `format_progress_message()` (`main.rs:2147-2176`)
- **Shows compaction progress only during Finalizing status**: `" | compact X/Y"` is appended when `status == Finalizing && compactions_total > 0`

### Element C: Red Failed-Recovery Bar (`main.rs:2274-2298`)

Appears **only when there are failed blocks** (`failed > 0`):
```
[▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 5/20 | recovering failed: remaining 15/20
```

- Template: `{bar:40.red/black} {pos}/{len} | {msg}`
- Created dynamically when `failed_total > 0` and doesn't exist yet
- Cleared with `finish_and_clear()` when all failed blocks are recovered
- **Rarely seen** because failed blocks are typically recovered quickly

### Element D: Green Follow Mode Bar (`main.rs:2349-2419`)

Appears **only after initial sync completes** (`processed >= total_len`):
```
[     [ 21345678 ]     ] Synced | head 21345680 | peers 8/12 | fetch 0 | failed 0
```

- Green bar shows current indexed block number centered in 40 chars
- Status text varies based on current state:
  - `Following` or `UpToDate` → "Synced"
  - `Fetching` → "Catching up (N blocks left)"
  - `Finalizing` → "Finalizing (flushing/compacting...)" + compaction counter
  - `LookingForPeers` → "Waiting for peers"

---

## 3. Identified Issues

### Issue 1: Finalizing Status Never Shows Green Bar During Initial Sync

**Root cause**: The transition from cyan bar to green bar happens at `main.rs:2345-2357` when `processed >= total_len`. But `SyncStatus::Finalizing` is set at `mod.rs:956` which is **AFTER** all blocks are processed (fetched and written to DB).

**Timeline**:
1. All blocks fetched → status = `Finalizing` at `mod.rs:956`
2. `processed` counter has already reached `total_len` by this point
3. The cyan bar transitions to green bar immediately upon reaching 100%
4. So "Finalizing" status appears on the **green follow bar**, not the cyan bar

**What you see**: The cyan bar shows "status finalizing" only if there's a brief window where `processed < total_len` but fetching is done. In practice, this window is often zero because processing finishes before the status update propagates.

### Issue 2: Status Text vs Display Text Inconsistency

In the cyan bar: `status finalizing` (snake_case from `as_str()`)
In the green bar: `Finalizing` (title case, hardcoded)

This creates confusion - the same status looks different depending on which bar is showing.

### Issue 3: Red Failed Bar - Rarely Visible

The red bar only appears when:
1. `failed > 0` AND
2. Failed blocks haven't been recovered yet

Since recovery happens automatically and quickly, this bar flashes briefly or never appears at all in normal runs.

### Issue 4: No Visual Feedback During Initial Finalizing

When initial sync reaches 100%, the cyan bar immediately finishes and the green bar appears. If compaction takes time during finalization, you see:
- Green bar with "Synced" (because status might be `UpToDate` or `Following` after `Finalizing` completes quickly)
- OR green bar with "Finalizing" but that's often missed because it transitions fast

### Issue 5: Code Scattered Across Multiple Files

The UI logic is spread across:
- `main.rs:528-569` - startup bars
- `main.rs:1004-1038` - progress bar creation
- `main.rs:2147-2176` - message formatting
- `main.rs:2231-2423` - progress updater (200 lines in one async block)
- `sync/mod.rs:21-176` - status types and stats
- `sync/historical/mod.rs`, `follow.rs`, `db_writer.rs` - status updates

This makes it hard to understand and maintain.

---

## 4. Summary Table

| UI Element | When Visible | Color | File Location |
|------------|--------------|-------|---------------|
| Startup bars | Before sync starts | Yellow | `main.rs:528-569`, called at 901, 910, 923, 566, 1478 |
| Main progress bar | During initial sync | Cyan/Blue | `main.rs:1005-1011`, updated at `2231-2358` |
| Failed recovery bar | When `failed > 0` | Red | `main.rs:2274-2298` |
| Follow mode bar | After initial sync | Green | `main.rs:2359-2419` |

---

## 5. Recommendations for Refactoring

1. **Extract UI into a separate module** (e.g., `ui/mod.rs` or `progress.rs`) with:
   - `StartupProgress` - handles yellow bars
   - `SyncProgress` - handles cyan bar during sync
   - `FollowProgress` - handles green bar in follow mode
   - `FailedRecoveryProgress` - handles red bar

2. **Unify status display text** - use consistent formatting across all bars

3. **Add explicit "finalizing" visual state** - maybe a yellow bar during finalization instead of just text

4. **Consider removing the failed bar** if it's never practically visible, or make recovery more visible

5. **Use a state machine** for UI transitions instead of multiple boolean flags (`initial_sync_done`, `follow_bar`, `failed_bar`)
