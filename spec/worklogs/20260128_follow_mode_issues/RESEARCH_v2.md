# Follow Mode Issues - Research v2 (2026-01-29)

Analysis of the latest run `2026-01-28__23-06-44` with improved logging.

## Executive Summary

Two bugs are causing follow mode to stall (plus a log spam issue):

1. **Stale-peer spin-loop** (causes 10GB logs, CPU waste, starves pipeline): Peers with genuinely old heads get banned+recycled in a tight loop (12.2M ban events for 2 peers), monopolizing the main loop and preventing healthy peers from being assigned work
2. **Re-probe head_number never reaches the pipeline**: `re_probe_peer_head` updates the pool, but the peer clone circulating in the ready channel retains its stale `head_number` forever — re-probes are wasted

**Corrected understanding:** The follow mode fetching itself is NOT broken. When the spin-loop doesn't starve the pipeline, the node keeps up with the chain at 1 block per ~12s slot. The "130 blocks behind, 0 active peers" state is caused entirely by Bug 1 monopolizing the main loop.

## Bug 1: Stale-Peer Spin-Loop (Critical - causes 10GB logs)

### Evidence

```
Total ban events: 12,215,929
Total unique peers:  2

  8,883,913 bans  peer=0xb3fe...  (head=23,935,725 — 400k blocks behind)
  3,332,016 bans  peer=0xc9d8...  (similar)
```

All 8.8M bans for `0xb3fe` happen at the **same millisecond** (t=107401ms). This is a tight spin-loop.

### Root Cause

The stale-head check at `mod.rs:775` runs **before** the scheduler's `is_peer_cooling_down()` check. The flow:

1. Peer selected from `ready_peers` (line 763-764)
2. Stale check: `peer.head_number (23.9M) < min_required (24.3M)` → true
3. `set_stale_head_cooldown(120s)` called (line 782-783)
4. Re-probe spawned (line 796)
5. **Peer recycled immediately**: `ready_tx.send(peer)` (line 824)
6. Main loop: `ready_rx.try_recv()` picks it up again (line 624)
7. Back to step 1 → tight loop, thousands of times per millisecond

The cooldown set at step 3 is only checked in `scheduler.next_batch_for_peer()` (scheduler.rs:665), but **we never reach the scheduler** because the stale check at step 2 catches it first.

### Fix

Before the stale-head check, check `is_peer_cooling_down()`. If the peer is already cooling down, skip it without logging, without re-probing, and recycle with a delay.

Also: genuinely stale peers (400k behind) should be **dropped entirely** from the ready pool, not recycled. They will never catch up.

## Bug 2: Re-probe Updates Pool But Not Pipeline Peer

### Evidence

At t=606s, the re-probe spam shows:
```
peer head re-probed successfully  new_head=24336560  probe_block=24336560
```
Hundreds of successful re-probes for the same peer at the same millisecond. But the peer keeps getting banned.

### Root Cause

`re_probe_peer_head()` calls `pool.update_peer_head(peer_id, header.number)` — this updates the **pool's copy** of the peer.

But the peer object in the pipeline's ready channel is a **clone** created when `spawn_peer_feeder` first sent it. The `peer_feeder` only sends NEW peers (`known.insert()` guard at line 118). Once a peer is "known", it's never refreshed from the pool.

So the peer clone circulating through `ready_tx → ready_rx → ready_peers → ready_tx` keeps its original stale `head_number` forever. The re-probe is completely wasted.

### Fix

When a peer arrives from `ready_rx`, refresh its `head_number` from the pool:
```rust
while let Ok(mut peer) = ready_rx.try_recv() {
    // Refresh head from pool in case re-probe updated it
    if let Some(fresh_head) = pool.get_peer_head(peer.peer_id) {
        peer.head_number = fresh_head;
    }
    if ready_set.insert(peer.peer_id) {
        ready_peers.push(peer);
    }
}
```

## NOT a Bug: One-Block-at-a-Time Tail Fetching

### Initial (incorrect) concern

Initially suspected that processing only 1 block per ~12s would be too slow to keep up.

### Evidence that it IS keeping up

Progress checks from 23-06-44 run (when spin-loop doesn't starve the pipeline):
```
t=  37s: completed= 67, delta=67, pending=0, inflight=0, ready_peers=5
t=  97s: completed= 72, delta= 2, pending=0, inflight=0, ready_peers=7
t= 157s: completed= 76, delta= 2, pending=0, inflight=0, ready_peers=12
t= 307s: completed= 89, delta= 3, pending=0, inflight=0, ready_peers=15
t= 608s: completed=114, delta= 3, pending=0, inflight=0, ready_peers=24
t= 699s: completed=121, delta= 2, pending=0, inflight=0, ready_peers=29
```

`pending=0, inflight=0` consistently — every block is processed as soon as it arrives. Rate is 2-3 blocks per 30s = ~1 block per 12s, which exactly matches Ethereum's slot time. The node IS keeping up 1:1 with the chain.

### How head discovery works

1. `run_head_tracker` (follow.rs:79) polls every **1 second** (`FOLLOW_POLL_MS = 1000`)
2. Each poll calls `discover_head_p2p()` which probes **3 peers** with `GetBlockHeaders(baseline+1, 1024)`
3. Takes the highest block number from responses → updates `head_seen`
4. `run_tail_scheduler` watches `head_seen` and sends `next_to_schedule..=head_seen` as a range
5. If head jumps by N blocks, it sends a range of N blocks at once

The 12-second gap between blocks is just Ethereum's slot time. Each block is fetched in ~200-500ms (even with 2-4 peer attempts for propagation lag), well within the 12-second window.

### The initial 65-block deficit

The 65-block gap from the fast-sync → follow transition is fetched during the first follow epoch in ~800ms. After that, the deficit is zero and the node tracks the chain 1:1.

### Future consideration: catching up from large gaps

If the node falls behind significantly (e.g., from Bug 1 starving the pipeline), it will need to catch up from a multi-block deficit. The `discover_head_p2p` probe uses `limit=1024`, so `head_seen` would jump ahead by many blocks at once, and `run_tail_scheduler` would send the full range. The existing parallel pipeline would then process the gap using multiple peers, similar to fast-sync. This should work correctly once Bug 1 is fixed.

### How Reth handles it (for reference)

Reth uses a different architecture — it learns the head from the Consensus Layer (not P2P probing) and has two sync modes:
- **Live sync (< 32 blocks)**: Parallel block downloads via `BasicBlockDownloader`
- **Backfill (> 32 blocks)**: Full staged pipeline

We don't need this architecture since our P2P head discovery already works correctly.

## Bug 4: Log Spam (10GB from 2 peers)

### Evidence

The 12.2M ban events for 2 peers generate ~10GB of logs. At t=107401ms, the same ban message is logged 8.8M times with **identical parameters and timestamp**.

### Fix: Log Deduplication

When the same log message + fields repeat within a short window (e.g. 1 second), batch them:
```
[8,883,913x] peer head too far behind, banning for 120s  peer=0xb3fe... peer_head=23935725
```

This should be implemented in the JSON logger to avoid:
1. Disk I/O waste (10GB → ~1KB for this case)
2. Analysis difficulty (impossible to grep through 10GB)
3. Performance impact (writing 10GB of logs)

## Prioritized Fix Order

1. **Bug 1 (spin-loop)**: Check cooldown BEFORE stale-head ban. Drop truly stale peers. This is the single root cause of the "0 active peers, 130 blocks behind" state — the spin-loop monopolizes the main loop and starves healthy peers of assignment time.

2. **Bug 2 (re-probe ineffective)**: Refresh peer head_number from pool when receiving from ready channel. Without this, peers with temporarily stale heads never recover even after successful re-probes. (Lower priority than Bug 1 since the genuinely stale peers from the logs are 400k blocks behind and should just be dropped.)

3. **Bug 3 (log spam)**: Add log deduplication to prevent 10GB log files. Defense in depth.
