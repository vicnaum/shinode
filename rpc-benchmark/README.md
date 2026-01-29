# RPC Performance Investigation

## Problem Statement

The `eth_getLogs` RPC endpoint is extremely slow - taking ~6 seconds for a 21-block range query. Previously, the same operation was nearly instant (rindexer could sync 100k blocks in seconds).

### Symptoms

1. **Slow RPC response**: `cast logs --from-block 24328130 --to-block 24328151` takes ~6 seconds
2. **CPU/IO bound**: The delay appears to be in storage layer, not network
3. **Regression**: This was fast in older commits (pre Jan 22, 2026)

### Test Command

```bash
# Start the node
cargo run --bin stateless-history-node --release --manifest-path node/Cargo.toml -- \
    --start-block 24320000 --shard-size 100

# Test RPC performance (in another terminal)
time cast logs --rpc-url http://localhost:8545 --from-block 24328130 --to-block 24328151
```

### Expected vs Actual

| Metric | Expected | Actual |
|--------|----------|--------|
| Response time | <100ms | ~6000ms |
| File operations | Minimal | 300+ per query |

---

## Root Cause Analysis

### Timeline

- **Jan 21, 2026** - Commit `11721f1` (schema v1) - **FAST** - known working
- **Jan 22, 2026** - Commit `4e59cbc` (schema v2) - **SLOW** - regression introduced

The regression was introduced with the "sharded storage v2" refactor in commit `4e59cbc`.

### Technical Cause

The v2 storage opens segment files **on every single block read** with no caching.

For a 21-block `eth_getLogs` query:
- `has_block` check: 21 calls (fast, just bitset lookup)
- `block_headers_range`: 21 blocks × 5 segment files = 105 file opens
- `block_tx_hashes_range`: 21 blocks × 5 segment files = 105 file opens
- `block_receipts_range`: 21 blocks × 5 segment files = 105 file opens

**Total: 315+ file open/close operations for 21 blocks**

### Code Location

The problematic pattern is in `node/src/storage/sharded/mod.rs`:

```rust
fn with_readers_for_present_block<T>(...) -> Result<Option<T>> {
    // ...
    let segments = shard_segment_readers(&sorted_dir, shard_start)?;  // Opens 5 files!
    // ...
}
```

This is called by `block_header()`, `block_tx_hashes()`, `block_receipts()`, etc.

---

## Investigation Approach

### 1. Binary Search (Initial Plan)

We initially planned to binary search through commits to find the regression. However, we discovered:
- The regression is at the **schema boundary** (v1 → v2)
- Schema v1 data is incompatible with v2 code
- Binary search within v2 commits is not useful - they're all slow

### 2. Code Comparison (Current Approach)

Compare v1 and v2 storage implementations side-by-side to understand:
- How v1 handled file access (likely persistent handles or different architecture)
- What v2 changed that caused the regression
- How to fix v2 while keeping the new sharded architecture benefits

---

## Repository Structure

```
rpc-benchmark/
├── README.md              # This file - problem overview
├── SESSION_STATE.md       # Session context for continuity
├── bench.sh               # Benchmark script
├── PLAN.md                # Original investigation plan
├── COMMITS.md             # Binary search tracking (superseded)
└── research/
    ├── README.md                    # Research index
    ├── 01-v1-v2-comparison.md       # v1 vs v2 storage analysis
    └── 02-implementation-plan.md    # LRU cache fix implementation
```

### Worktrees for Comparison

```bash
# Main repo (current v2 code)
./node/src/storage/sharded/mod.rs

# Old v1 code (via git worktree)
./old-v1/node/src/storage/mod.rs
```

To recreate the worktree if needed:
```bash
git worktree add old-v1 11721f1
```

---

## Files to Compare

| Component | v2 (Current) | v1 (Old) |
|-----------|--------------|----------|
| Storage main | `node/src/storage/sharded/mod.rs` | `old-v1/node/src/storage/mod.rs` |
| RPC handler | `node/src/rpc/mod.rs` | `old-v1/node/src/rpc/mod.rs` |

Key functions to compare:
- `block_header()` / `block_headers_range()`
- `block_receipts()` / `block_receipts_range()`
- File handle management
- Caching strategies

---

## Data Directory

**Location:** `./data` (default)

**DO NOT DELETE** - contains synced blocks needed for testing

**Schema:** v2 (only compatible with commits from `4e59cbc` onwards)

---

## How to Continue This Investigation

1. Read `RESEARCH.md` for detailed code comparison findings
2. The fix likely involves caching segment readers per shard
3. Key question: How did v1 avoid repeated file opens?

---

## Benchmark Script Usage

```bash
# Full test (build, start node, benchmark, kill)
./rpc-benchmark/bench.sh

# Just benchmark (node already running)
./rpc-benchmark/bench.sh --no-node --no-build

# Keep node running after benchmark
./rpc-benchmark/bench.sh --no-kill
```

---

## Key Commits

| Commit | Date | Description | Schema | Performance |
|--------|------|-------------|--------|-------------|
| `11721f1` | Jan 21 | Last known fast | v1 | FAST |
| `4e59cbc` | Jan 22 | Sharded storage v2 | v2 | SLOW |
| `6efc7ad` | Current | TUI improvements | v2 | SLOW |
