# RPC Benchmark Results

Tracking `eth_getLogs` performance across commits.

## Test Environment
- **Block range**: 24328130 - 24328229 (100 blocks)
- **RPC endpoint**: http://localhost:8545
- **Shard size**: 100
- **Iterations**: 3

## Results

| Date | Commit | Description | Blocks | Min | Max | Avg | Notes |
|------|--------|-------------|--------|-----|-----|-----|-------|
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 100 | 404ms | 617ms | **477ms** | shard-size 100 |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 1000 | 282ms | 1117ms | **561ms** | shard-size 100 |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 10000 | 26069ms | 26893ms | **26462ms** | shard-size 100, 6.5x slower |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 100 | 397ms | 572ms | **462ms** | shard-size 1000 |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 1000 | 351ms | 538ms | **420ms** | shard-size 1000 |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 10000 | 3186ms | 5127ms | **3890ms** | shard-size 1000 ✅ matches V1! |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 100 | 381ms | 554ms | **448ms** | shard-size 10000 |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 1000 | 306ms | 433ms | **381ms** | shard-size 10000 |
| 2026-01-28 | `2c9397a` | V2 + LRU cache fix | 10000 | 3273ms | 4267ms | **3658ms** | shard-size 10000 ✅ |
| 2026-01-28 | `6efc7ad` | V2 before cache (master) | 21 | - | - | ~6000ms | Original slow performance |
| 2026-01-28 | `76ab097` | **V1 baseline (target)** | 100 | 471ms | 556ms | **500ms** | Last v1 commit |
| 2026-01-28 | `76ab097` | **V1 baseline (target)** | 1000 | 412ms | 733ms | **580ms** | Scales well! |
| 2026-01-28 | `76ab097` | **V1 baseline (target)** | 10000 | 3371ms | 5181ms | **4066ms** | 0.4ms/block |

## Analysis

### Problem
Storage v2 introduced sharded segments. Each `eth_getLogs` call was reopening 5 segment files per block:
- 21 blocks × 3 data types × 5 files = **315 file opens** per query

### Solution
Added LRU cache (20 shards) for `ShardSegments` in commit `2c9397a`:
- Cache hit: reuse already-open segment readers
- Result: ~5-15 file opens instead of 315

### Performance Comparison
| Version | Time/100 blocks | Per block |
|---------|-----------------|-----------|
| V1 baseline | ~525ms | ~5.2ms |
| V2 + cache | ~478ms | ~4.8ms |
| V2 no cache | ~6000ms (21 blks) | ~286ms |

**Conclusion**: LRU cache fix + appropriate shard size restores V2 performance to V1 levels!

### Shard Size Impact
| Shard Size | 10k blocks | Shards accessed | Performance |
|------------|------------|-----------------|-------------|
| 100 | 26,500ms | 100 shards | ❌ 6.5x slower |
| 1000 | 3,890ms | 10 shards | ✅ Matches V1 |
| 10000 | 3,658ms | 1 shard | ✅ Slightly faster |

**Recommendation**: Use shard-size 1000+ for production workloads with large range queries. Diminishing returns beyond 1000.

---

## Rindexer Benchmark

Testing event indexing performance using rindexer against local RPC.

### Test: UniswapV3Pool Swap Events

| Date | Commit | Block Range | Events | Time | Events/sec | Notes |
|------|--------|-------------|--------|------|------------|-------|
| 2026-01-28 | `76ab097` | 24320000-24330000 | 26,345 | **6s** | ~4,390/s | V1, 10k blocks |
| 2026-01-28 | `2c9397a` | 24320000-24330000 | 26,345 | **5s** | ~5,270/s | V2 + cache, shard-size 1000 ✅ |
| 2026-01-28 | `2c9397a` | 24320000-24330000 | 26,345 | **5s** | ~5,270/s | V2 + cache, shard-size 10000 ✅ |

**Details:**
- Contract: `0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b` (UniswapV3Pool)
- Event: `Swap`
- RPC: `http://127.0.0.1:8545`
- Batch 1: 24320000-24325000 → 12,368 logs
- Batch 2: 24325001-24330000 → 13,977 logs
- Output: CSV to `./rindexer_csv`
