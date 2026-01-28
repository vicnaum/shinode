# RPC Performance Regression Hunt

## Goal
Find the commit where `eth_getLogs` performance regressed from ~instant (100k blocks in seconds) to ~6 seconds for 20 blocks.

## Test Command
```bash
cast logs --rpc-url http://localhost:8545 --from-block 24328130 --to-block 24328151
```

## Node Command
```bash
cargo run --bin stateless-history-node --release --manifest-path node/Cargo.toml -- --start-block 24320000 --shard-size 100
```

## Data Directory
- Location: `./data` (default)
- **DO NOT DELETE** - contains synced blocks needed for testing
- Schema: v2 (compatible with commits from `4e59cbc` onwards, Jan 22 2026)

## Strategy

### Phase 1: Confirm Regression
1. Run benchmark on current HEAD
2. Confirm ~6s response time

### Phase 2: Find Boundary (Coarse)
Test these key commits in order (newest to oldest):
1. Current HEAD - TUI improvements
2. Pre-TUI commits
3. Pre-clippy refactor
4. Earlier sync/storage changes

Stop when we find a "fast" commit.

### Phase 3: Binary Search (Fine)
Once we have slow/fast boundary, bisect to find exact commit.

## Failure Handling

| Failure Type | Detection | Action |
|--------------|-----------|--------|
| Build fails | `cargo build` exit code != 0 | Mark `[BUILD FAIL]`, skip to next |
| Node won't start | No PID after 5s | Mark `[START FAIL]`, skip to next |
| RPC not ready | `eth_blockNumber` fails after 60s | Mark `[RPC TIMEOUT]`, skip to next |
| RPC hangs | `cast logs` takes >30s | Mark `[SLOW >30s]`, record actual time |
| Schema mismatch | Node error on startup | Mark `[SCHEMA FAIL]`, stop (hit v1 boundary) |
| Data not found | RPC returns error | Mark `[NO DATA]`, skip to next |

## Recording Results

Format in COMMITS.md:
```
- [ ] `abc1234` - commit message
  - Result: [STATUS] Time: XXXms / Error: reason
```

Statuses:
- `[OK]` - Completed successfully
- `[SLOW >30s]` - Completed but very slow
- `[BUILD FAIL]` - Cargo build failed
- `[START FAIL]` - Node didn't start
- `[RPC TIMEOUT]` - RPC never became ready
- `[SCHEMA FAIL]` - Incompatible schema version
- `[NO DATA]` - RPC couldn't find the blocks
- `[SKIP]` - Skipped (not relevant)

## Script Usage

```bash
# Start node in background, wait for ready, run benchmark, kill node
./rpc-benchmark/bench.sh

# Just run benchmark (node already running)
./rpc-benchmark/bench.sh --no-node
```
