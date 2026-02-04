# Fix Plan v2: Follow Mode Stall Issues

Based on RESEARCH_v2.md analysis of the `2026-01-28__23-06-44` run.

## Phase 1: Stop the Spin-Loop (Bug 1 + Bug 2)

**Files:** `node/src/sync/historical/mod.rs`, `node/src/p2p/mod.rs`

### 1a. Check cooldown BEFORE stale-head ban

In `run_ingest_pipeline` (mod.rs ~line 767), before the stale-head check, add:

```rust
// Skip peers already on stale-head cooldown — don't re-ban, don't re-probe, don't log
if peer_health.is_stale_head_cooling_down(peer.peer_id).await {
    // Recycle with delay to avoid spin
    let ready_tx = ready_tx.clone();
    drop(permit);
    tokio::spawn(async move {
        sleep(Duration::from_millis(500)).await;
        let _ = ready_tx.send(peer);
    });
    continue;
}
```

This prevents the 12.2M-event spin-loop. Need to add `is_stale_head_cooling_down()` to `PeerHealthTracker` if it doesn't exist (or reuse `is_peer_cooling_down`).

### 1b. Refresh peer head from pool on receive

In `run_ingest_pipeline` (mod.rs ~line 624), when receiving peers from ready channel:

```rust
while let Ok(mut peer) = ready_rx.try_recv() {
    // Refresh head_number from pool — re-probes update the pool, not the clone
    if let Some(head) = pool.get_peer_head(peer.peer_id) {
        peer.head_number = head;
    }
    if ready_set.insert(peer.peer_id) {
        ready_peers.push(peer);
    }
}
```

Also add `get_peer_head(peer_id) -> Option<u64>` to `PeerPool`.

### 1c. Drop truly stale peers instead of recycling

If a peer is >10,000 blocks behind, don't recycle it at all. It's either a light client or still syncing. The `peer_feeder` will re-add it if it reconnects.

```rust
if peer.head_number > 0 && peer.head_number < min_required {
    let gap = min_required - peer.head_number;
    if gap > 10_000 {
        // Truly stale peer — drop from ready pool entirely
        tracing::debug!(peer_id = ?peer.peer_id, peer_head = peer.head_number, gap, "dropping truly stale peer");
        drop(permit);
        continue; // Don't recycle via ready_tx
    }
    // ... existing ban + re-probe for moderately stale peers ...
}
```

## Phase 2: Log Deduplication (Bug 4)

**Files:** `node/src/logging/json.rs` (or wherever the JSON logger is)

### 2a. Add dedup layer to the JSON logger

Track the last N log messages. If the same `(message, fields)` repeats within a 1-second window, suppress and count:

```
// Instead of 8,883,913 identical lines:
{"t_ms":107401,"message":"peer head too far behind, banning for 120s","fields":{"peer_id":"0xb3fe...","peer_head":23935725},"count":8883913}
```

Implementation options:
- A ring buffer of recent `(message_hash, timestamp)` entries
- On match within window: increment counter, suppress write
- On window expiry or different message: flush with count

### 2b. Rate-limit per-message in hot paths

For messages known to be spammy, add `tracing` rate limiting:
- `peer head too far behind` → max once per peer per 30s
- `peer head re-probe failed` → max once per peer per 30s
- `scheduler: peer cooling down` → max once per peer per 30s
- `ingest: empty batch for peer` → max once per peer per 5s
- `peer head re-probed successfully` → max once per peer per 30s

## Implementation Order

1. **Phase 1** (1a, 1b, 1c) — Fixes the spin-loop, which is the sole root cause of "0 active peers". Reduces logs from 10GB to <100MB. Quick wins, minimal risk.

2. **Phase 2** (2a or 2b) — Prevents log size explosion even if new spin-loops are introduced. Defense in depth.

NOTE: No Phase 3 needed. The follow mode fetching itself works correctly once the spin-loop is fixed — the node keeps up with the chain at 1 block per ~12s slot. The `discover_head_p2p` + `run_tail_scheduler` + parallel pipeline already handles both tip-following and catching up from gaps (via `GetBlockHeaders(baseline+1, 1024)` which discovers multi-block gaps at once).

## Testing

- Run with `--start-block` near tip, observe follow mode behavior
- Verify: no 10GB log files, no 0-active-peer states
- Verify: when synced, node stays within 1-2 blocks of tip
- Verify: stale peers (400k behind) are dropped, not spin-looped
