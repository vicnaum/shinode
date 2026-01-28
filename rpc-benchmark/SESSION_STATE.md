# Session State - RPC Performance Investigation

**Last Updated:** 2026-01-28

## Current Status: ROOT CAUSE FOUND

The RPC slowness has been identified. See `research/` folder for detailed analysis.

## Summary

**Problem:** `eth_getLogs` takes ~6 seconds for 21 blocks (should be <100ms)

**Root Cause:** v2 sharded storage opens segment files on every block read with no caching.
- For a 21-block query: **315 file open/close operations** (vs 3 in v1)

**Regression Commit:** `4e59cbc` (Jan 22, 2026) - "feat(storage): implement sharded storage v2"

## Files in This Directory

| File | Purpose |
|------|---------|
| `README.md` | Overview, problem statement, how to continue |
| `SESSION_STATE.md` | This file - quick reference |
| `bench.sh` | Benchmark script |
| `PLAN.md` | Original investigation plan |
| `COMMITS.md` | Binary search tracking (superseded) |
| `research/` | Detailed analysis subfolder |
| `research/01-v1-v2-comparison.md` | v1 vs v2 storage comparison |
| `research/02-implementation-plan.md` | LRU cache fix implementation plan |
| `research/03-reth-rpc-optimization.md` | How Reth handles RPC caching and static files |
| `research/04-reth-gemini-research.md` | Gemini deep dive on Reth patterns (thundering herd, hot/cold) |

## Git State

```bash
# Main working tree
git branch  # ui-ratatui-dashboard

# Old v1 code for comparison
ls old-v1/  # git worktree at commit 11721f1
```

To recreate worktree if needed:
```bash
git worktree add old-v1 11721f1
```

## Key Findings

### Why v1 is Fast
- Segments opened ONCE at startup
- Range reads use single file open + cursor iteration
- File operations for 21-block query: **3**

### Why v2 is Slow
- Segments opened on EVERY block read
- Range functions iterate and call per-block reads
- Each block read opens 5 segment files
- File operations for 21-block query: **315**

### How Reth Does It
- `DashMap<(BlockNumber, Segment), LoadedJar>` caches open NippyJar files
- `Arc<DataReader>` shares mmap handles across multiple cursors
- Multi-level caching: RPC layer LRU + static file cache + mmap
- Bloom filter pre-filtering for `eth_getLogs` (skips blocks without matching logs)
- See `research/03-reth-rpc-optimization.md` for details

## Proposed Fix

**Option 1 (Recommended):** Cache segment readers per shard in `ShardState`

```rust
struct ShardState {
    dir: PathBuf,
    meta: ShardMeta,
    bitset: Bitset,
    segments: Option<ShardSegments>,  // NEW: Cached readers
}
```

See `research/02-implementation-plan.md` for full implementation details.

## Next Steps

1. Implement segment caching in `node/src/storage/sharded/mod.rs`
2. Run benchmark to verify fix
3. Test for regressions (writes, compaction, shard boundaries)

## Quick Commands

```bash
# Run benchmark (with node already running)
./rpc-benchmark/bench.sh --no-node --no-build

# Start node
cargo run --bin stateless-history-node --release --manifest-path node/Cargo.toml -- \
    --start-block 24320000 --shard-size 100

# Test RPC
time cast logs --rpc-url http://localhost:8545 --from-block 24328130 --to-block 24328151
```

## Data Directory

**Location:** `./data`
**Schema:** v2
**DO NOT DELETE** - contains synced test data
