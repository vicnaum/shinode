# RPC Performance Research

This folder contains detailed research and analysis for the RPC performance investigation.

## Documents

| File | Description |
|------|-------------|
| [01-v1-v2-comparison.md](01-v1-v2-comparison.md) | Side-by-side comparison of v1 and v2 storage implementations |
| [02-implementation-plan.md](02-implementation-plan.md) | Detailed implementation plan for the LRU cache fix |
| [03-reth-rpc-optimization.md](03-reth-rpc-optimization.md) | How Reth optimizes RPC serving (caching patterns, static files, eth_getLogs) |
| [04-reth-gemini-research.md](04-reth-gemini-research.md) | Deep dive from Gemini on Reth caching patterns (thundering herd, hot/cold, etc.) |

## Summary

### Problem
`eth_getLogs` takes ~6 seconds for 21 blocks (should be <100ms)

### Root Cause
v2 sharded storage opens 5 segment files on every block read with no caching.
- 21-block query = 315 file open/close operations
- v1 did the same query with 3 file operations

### Solution
Add LRU cache for segment readers per shard.

### Reth's Approach (from Gemini analysis)
Reth uses a **two-layer caching strategy**:
1. **Storage layer**: `DashMap<(BlockNumber, Segment), LoadedJar>` caches open mmap handles (not data)
2. **RPC layer**: LRU cache for deserialized blocks/receipts with **thundering herd protection**

Key optimizations:
- **Hot vs Cold differentiation**: Small/recent queries use LRU, large/historical bypass it (avoids cache pollution)
- **Per-cursor buffers**: Reusable decompression buffers avoid malloc per block
- **Bloom filter pre-filtering** for `eth_getLogs`
- **No eviction in storage cache** (TODO in their code, same as our finding)

## Quick Links

- [Main README](../README.md) - Problem overview
- [Benchmark script](../bench.sh) - Test RPC performance
- [Session state](../SESSION_STATE.md) - Current investigation status

## Code References

**v2 (current, slow):**
- `node/src/storage/sharded/mod.rs:1334-1355` - `with_readers_for_present_block()`
- `node/src/storage/sharded/mod.rs:1580-1614` - `shard_segment_writers()`

**v1 (old, fast):**
- `old-v1/node/src/storage/mod.rs:417-447` - `read_range()`
- `old-v1/node/src/storage/mod.rs:538-552` - Segment initialization

**Reth (reference implementation):**
- `reth/crates/storage/provider/src/providers/static_file/manager.rs` - Static file caching with DashMap
- `reth/crates/rpc/rpc-eth-types/src/cache/mod.rs` - RPC LRU cache
- `reth/crates/rpc/rpc/src/eth/filter.rs` - eth_getLogs with bloom filtering
- `spec/reth_kb/questions/Q022-eth-getlogs.md` - Knowledge base on getLogs implementation
