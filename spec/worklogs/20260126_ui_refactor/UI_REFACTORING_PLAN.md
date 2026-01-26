# UI Progress Bars Refactoring Plan

## Goals

1. **Consolidate UI code** into a dedicated module
2. **Fix the "stuck on cyan bar" bug** where Finalizing never transitions properly
3. **Unify rendering** to avoid mixed ANSI/indicatif competition
4. **Simplify state management** with a clear state machine
5. **Add distinct Finalizing stage** with teal color for clear visual feedback
6. **Fix failed recovery bar** coordination with MultiProgress

---

## Phase 1: Extract UI Module

### 1.1 Create `node/src/ui/mod.rs`

Move all UI-related code from `main.rs` into a new module:

```
node/src/ui/
├── mod.rs           # Public API, re-exports
├── progress.rs      # Main progress tracking logic
├── bars.rs          # Bar rendering (indicatif wrappers)
└── state.rs         # UI state machine
```

### 1.2 Define Clear Public API

```rust
// ui/mod.rs
pub struct ProgressUI { ... }

impl ProgressUI {
    pub fn new(total_blocks: u64, is_tty: bool) -> Self;
    pub fn update(&self, snapshot: &SyncProgressSnapshot);
    pub fn finish(&self);
}
```

### 1.3 Move Functions

| From `main.rs` | To |
|----------------|-----|
| `print_status_bar()` | `ui/bars.rs` |
| `clear_status_bar()` | `ui/bars.rs` |
| `format_progress_message()` | `ui/progress.rs` |
| `spawn_progress_updater()` | `ui/progress.rs` |
| `wait_for_min_peers()` | Keep in `main.rs` but use `ui::` for display |

---

## Phase 2: Unify Rendering System

### 2.1 Use `MultiProgress` for All Bars

Replace independent `ProgressBar` instances with a coordinated `MultiProgress`:

```rust
pub struct ProgressUI {
    multi: MultiProgress,
    startup_bar: Option<ProgressBar>,    // Yellow status messages
    main_bar: Option<ProgressBar>,       // Cyan/blue sync progress
    finalizing_bar: Option<ProgressBar>, // Teal/bright cyan finalizing
    follow_bar: Option<ProgressBar>,     // Green follow status
    failed_bar: Option<ProgressBar>,     // Red failed recovery
}
```

### 2.2 Convert Yellow Bars to Indicatif

Replace raw ANSI `print_status_bar()` with an indicatif spinner/bar:

```rust
// Before: eprint!("\r\x1b[...]{message}\x1b[0m");
// After:
let bar = multi.add(ProgressBar::new_spinner());
bar.set_style(ProgressStyle::with_template("{spinner} {msg}")
    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"));
bar.set_message("Opening storage...");
```

Benefits:
- No raw terminal escape codes
- Proper coordination with other bars
- Consistent refresh rates

---

## Phase 3: Fix State Transition Bugs

### 3.1 Problem: `processed < total_len` When Finalizing

**Current behavior**: Bar transition depends on `processed >= total_len`, but `processed` only increments on successful processing. Failed blocks prevent transition.

**Solution**: Track completion via scheduler state, not just `processed` counter.

Add new field to `SyncProgressStats`:

```rust
pub struct SyncProgressStats {
    // ... existing fields ...
    fetch_complete: AtomicBool,  // NEW: true when all fetch tasks done
}
```

Update transition logic:

```rust
// Before:
if processed >= total_len { /* transition to green */ }

// After:
let fetch_done = snapshot.fetch_complete;
let all_processed = processed >= total_len;
let has_failures = snapshot.failed > 0;

if fetch_done && (all_processed || has_failures) {
    // Transition to follow bar, show appropriate status
}
```

### 3.2 Problem: Finalizing Status Set Too Late

**Current behavior**: `SyncStatus::Finalizing` is set after all fetch tasks complete, but by then `processed` may already equal `total_len`.

**Solution**: Set Finalizing earlier, when fetch completes but DB flush is pending:

```rust
// In run_ingest_pipeline(), after fetch_tasks.join_next() loop:
stats.set_fetch_complete(true);  // NEW
stats.set_status(SyncStatus::Finalizing);
```

### 3.3 Unify Status Display Text

Create a single source for status display:

```rust
impl SyncStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            SyncStatus::LookingForPeers => "Waiting for peers",
            SyncStatus::Fetching => "Syncing",
            SyncStatus::Finalizing => "Finalizing",
            SyncStatus::UpToDate => "Synced",
            SyncStatus::Following => "Synced",
        }
    }
}
```

Use this everywhere instead of `as_str()` for user-facing text.

---

## Phase 4: Keep Failed Recovery Bar (Option C)

### 4.1 Analysis

The red "failed recovery" bar is rarely seen because:
- Default `max_attempts_per_block = 3` is rarely exceeded
- In follow mode, `max_attempts_per_block = u32::MAX` (blocks never fail)

However, when it does appear, there could be many failed blocks requiring recovery, which can take significant time. A dedicated progress bar provides clear visual feedback.

### 4.2 Implementation: Keep with Proper Coordination

**Keep the separate failed recovery bar** but fix coordination issues:

1. **Use `MultiProgress`** to prevent line conflicts with other bars
2. **Show 0-100% progress** as failed blocks are recovered
3. **Delay appearance** - only show if failed count persists for > 2 seconds (avoids flicker for quick recoveries)
4. **Clear visual distinction** - keep red color to indicate "attention needed"

```rust
pub struct ProgressUI {
    multi: MultiProgress,
    startup_bar: Option<ProgressBar>,   // Yellow status messages
    main_bar: Option<ProgressBar>,      // Cyan sync progress
    finalizing_bar: Option<ProgressBar>, // Cyan/teal finalizing progress
    follow_bar: Option<ProgressBar>,    // Green follow status
    failed_bar: Option<ProgressBar>,    // Red failed recovery (KEPT)
}
```

**Failed bar behavior**:
```rust
// Only create after 2 seconds of failed > 0
if snapshot.failed > 0 && failed_bar.is_none() {
    if failed_first_seen.elapsed() > Duration::from_secs(2) {
        let bar = multi.add(ProgressBar::new(failed_total));
        bar.set_style(ProgressStyle::with_template(
            "{bar:40.red/black} {pos}/{len} | Recovering failed blocks..."
        ));
        failed_bar = Some(bar);
    }
}

// Update progress: shows recovery from 0% to 100%
if let Some(ref bar) = failed_bar {
    let recovered = failed_total - snapshot.failed;
    bar.set_position(recovered);

    if snapshot.failed == 0 {
        bar.finish_and_clear();
        failed_bar = None;
    }
}
```

---

## Phase 5: State Machine Design

### 5.1 Define UI States

```rust
pub enum UIState {
    Startup(StartupPhase),
    Syncing,
    Finalizing,
    Following,
}

pub enum StartupPhase {
    OpeningStorage,
    CompactingShards(usize),  // count
    ConnectingP2P,
    WaitingForPeers(usize, usize),  // current, min
    DiscoveringHead(u64),  // best_head
}
```

### 5.2 State Transitions

```
Startup(OpeningStorage)           [YELLOW]
    ↓
Startup(CompactingShards)         [YELLOW] (if dirty shards exist)
    ↓
Startup(ConnectingP2P)            [YELLOW]
    ↓
Startup(WaitingForPeers)          [YELLOW] (if min_peers > 0)
    ↓
Startup(DiscoveringHead)          [YELLOW]
    ↓
Syncing                           [CYAN] SyncStatus::Fetching
    ↓
Finalizing                        [TEAL] SyncStatus::Finalizing, shows compaction progress
    ↓
Following                         [GREEN] SyncStatus::Following/UpToDate
    ↓ (reorg or new blocks)
Syncing                           [CYAN] (back if significant work)
    ↓
Following                         [GREEN]

(At any point during Syncing/Finalizing, if failed > 0 for > 2s):
    → RecoveringFailed            [RED] overlay bar, clears when done
```

### 5.3 Rendering Per State

| UIState | Visual | Color |
|---------|--------|-------|
| `Startup(*)` | Spinner + message | **Yellow** (RGB 255,200,0) |
| `Syncing` | Progress bar + stats | **Cyan/Blue** (indicatif default) |
| `Finalizing` | Progress bar showing compaction | **Teal/Bright Cyan** (RGB 0,200,200) |
| `Following` | Status bar + block number | **Green** (RGB 0,128,0) |
| `RecoveringFailed` | Progress bar (overlay) | **Red** (indicatif red/black) |

### 5.4 Color Scheme Rationale

Each stage has a **distinct color** for instant visual recognition:

```
Yellow  → "Starting up, please wait"
Cyan    → "Actively downloading blocks"
Teal    → "Download done, compacting DB"  (NEW - distinct from cyan)
Green   → "Fully synced, following head"
Red     → "Recovering failed blocks" (attention needed)
```

**Teal for Finalizing** distinguishes it from:
- Cyan (still fetching) - user knows fetch is done
- Green (fully synced) - user knows DB work is pending

Suggested teal color: `RGB(0, 200, 200)` or `RGB(64, 224, 208)` (turquoise)

---

## Phase 6: Implementation Order

### Step 1: Create UI module structure (no behavior change)
- Create `ui/mod.rs`, `ui/bars.rs`, `ui/state.rs`
- Move types and helper functions
- Keep `main.rs` calling into the new module

### Step 2: Add `MultiProgress` coordinator
- Wrap all bars in `MultiProgress`
- Convert yellow bars to indicatif
- Verify no visual regressions

### Step 3: Add `fetch_complete` tracking
- Add field to `SyncProgressStats`
- Set it in `run_ingest_pipeline()`
- Update transition logic

### Step 4: Fix failed recovery bar coordination
- Move `failed_bar` into `MultiProgress`
- Add 2-second delay before showing (avoid flicker)
- Ensure proper cleanup when recovery completes

### Step 5: Add teal Finalizing bar
- Create distinct teal-colored bar for Finalizing state
- Show compaction progress (X/Y shards compacted)
- Transition: cyan bar finishes → teal bar appears → teal finishes → green bar

### Step 6: Implement state machine
- Replace boolean flags with `UIState` enum
- Clean up transition logic

### Step 7: Polish
- Consistent status text formatting
- Test all edge cases (empty range, immediate up-to-date, etc.)

---

## Testing Checklist

After refactoring, verify these scenarios work correctly:

### Startup Phase (Yellow)
- [ ] Fresh start → shows yellow "Opening storage..." → proceeds
- [ ] Dirty shards on startup → shows yellow "Compacting N shards..." → proceeds
- [ ] `--min-peers 5` → shows yellow "Waiting for peers 0/5" → proceeds when met
- [ ] Head discovery → shows yellow "Discovering chain head..." → proceeds

### Sync Phase (Cyan)
- [ ] Fresh start with many missing blocks → shows cyan progress bar with stats
- [ ] Progress bar shows: status, peers, queue, inflight, failed, speed, eta

### Finalizing Phase (Teal) - NEW
- [ ] After fetch completes → cyan bar finishes → teal bar appears
- [ ] Teal bar shows compaction progress (X/Y shards)
- [ ] Teal bar finishes → green bar appears

### Follow Phase (Green)
- [ ] After finalizing → shows green bar with block number + "Synced"
- [ ] Fresh start with zero missing blocks → shows "Synced" immediately
- [ ] Reorg during follow → briefly shows "Catching up" → returns to "Synced"

### Failed Recovery (Red)
- [ ] When blocks fail → red bar appears after 2 seconds
- [ ] Red bar shows 0-100% recovery progress
- [ ] When all recovered → red bar clears cleanly

### Edge Cases
- [ ] Start with `--end-block` (no follow) → cyan → teal → exits (no green)
- [ ] Processing failure during sync → still transitions (with error indication)
- [ ] Ctrl+C during any phase → clean exit, no garbled terminal
- [ ] Non-TTY output → no progress bars, only log messages

---

## Files to Modify

| File | Changes |
|------|---------|
| `node/src/main.rs` | Remove UI functions, add `mod ui`, update calls |
| `node/src/ui/mod.rs` | NEW: Public API |
| `node/src/ui/bars.rs` | NEW: Bar rendering |
| `node/src/ui/state.rs` | NEW: State machine |
| `node/src/ui/progress.rs` | NEW: Progress updater logic |
| `node/src/sync/mod.rs` | Add `fetch_complete` field, update `SyncStatus` display |
| `node/src/sync/historical/mod.rs` | Set `fetch_complete` flag |

---

## Decisions Made

1. **Finalizing visual**: Use a **teal progress bar** showing compaction progress (X/Y shards)
   - Distinct color makes it clear this is a separate stage from syncing

2. **Failed recovery bar**: **Keep as separate bar** (Option C)
   - Shows 0-100% progress as failed blocks are recovered
   - Only appears if failed count persists > 2 seconds
   - Useful when many blocks fail and recovery takes time

3. **Finalizing color**: **Teal/Bright Cyan** (RGB 0,200,200)
   - Distinct from cyan (fetching) and green (synced)
   - Signals "download done, DB work in progress"

---

## Open Questions

1. **What happens if the terminal is resized?**
   - Current code doesn't handle this
   - `indicatif` handles some cases but custom ANSI doesn't

2. **Should we support `--no-progress` flag?**
   - Some users may want clean log output only
   - Easy to add with the new module structure

3. **Should Finalizing bar show compaction only, or also include "flushing WAL" phase?**
   - Currently `compactions_total` is set, but there's also WAL flush time
   - Could show two-phase: "Flushing..." then "Compacting X/Y"
