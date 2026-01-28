# RPC Performance Binary Search

## Strategy
1. Test oldest compatible commit first (confirm it was fast)
2. Test newest (confirm it's slow)
3. Binary search to find the regression

**Schema v2 boundary:** `4e59cbc` (Jan 22) - oldest compatible with current data

---

## Test Order

### Step 1: Establish Boundaries

| # | Commit | Description | Result | Time |
|---|--------|-------------|--------|------|
| 1 | `4e59cbc` | **OLDEST** - sharded storage v2 | | |
| 2 | `6efc7ad` | **NEWEST** - current HEAD | | |

**If Step 1 shows: old=fast, new=slow → proceed to binary search**

---

### Step 2: Binary Search

Total commits between boundaries: ~58

| Round | Commit | Description | Result | Time | Direction |
|-------|--------|-------------|--------|------|-----------|
| 1 | `e2cae7e` | ~middle: refactor(ui): consolidate UI into UIController | | | |
| 2 | | (depends on round 1) | | | |
| 3 | | (depends on round 2) | | | |
| 4 | | (depends on round 3) | | | |
| 5 | | (depends on round 4) | | | |
| 6 | | (final narrowing) | | | |

**Binary search targets (precomputed midpoints):**

```
4e59cbc (oldest, #1) -------- e2cae7e (#30) -------- 6efc7ad (newest, #58)
                                  |
                    If SLOW: search left half
                    If FAST: search right half
```

---

### Reference: All Commits (oldest to newest)

For calculating midpoints during binary search:

```
#1  4e59cbc feat(storage): implement sharded storage v2
#2  a8c1740 feat(v0.2): benchmark warmup and batched fast-sync WAL writes
#3  5e33b84 chore: drop WAL single-write path
#4  8021ebb feat(v0.2): bound benchmark tracing and stream events
#5  a2831f0 Reduce compaction memory; serialize compactions
#6  4f8cbd9 feat(node): add optional jemalloc allocator
#7  17675ee perf(node): speed up shard compaction finalize
#8  1a54741 bench(node): force exit on Ctrl-C during ingest
#9  91cade5 ui: show compaction progress during finalizing
#10 e8d4486 perf(node): raw-byte shard compaction from WAL
#11 4d465aa perf(node): mmap WAL for faster shard compaction
#12 4030602 ingest: size pipeline buffers from --fast-sync-max-buffered-blocks
... (sync/perf commits)
#20 c1662f2 sync: stream missing ranges and long-lived follow
#21 b71d4e6 follow: don't cap scheduling by stale peer head
#22 7b6f10a refactor: replace benchmark mode with log artifact flags
... (refactoring)
#30 e2cae7e refactor(ui): consolidate UI into UIController  <-- MIDPOINT
... (more UI work)
#40 237999f refactor: remove dead code and apply cleanup decisions
#41 3fc2e16 feat: restore transaction storage and atomic compaction
... (escalation queue, clippy)
#50 aefa9f2 refactor: apply maximum clippy preset
#51 17a59d3 Merge pull request #1 from vicnaum/clippy-refactor
... (TUI ratatui work)
#58 6efc7ad feat: TUI improvements (current HEAD)
```

---

## Results Log

| Test | Commit | Time (ms) | Status | Notes |
|------|--------|-----------|--------|-------|
| 1 | | | | |
| 2 | | | | |
| 3 | | | | |
| 4 | | | | |
| 5 | | | | |
| 6 | | | | |
| 7 | | | | |
| 8 | | | | |

---

## Conclusion

**Regression introduced in commit:** `_____________`

**Cause:** `_____________`
