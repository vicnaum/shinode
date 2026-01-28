# P2P Discovery Visibility

## Problem

Out of 5,000 seeded cached peers + reth DHT discovery + 100 concurrent dials + 500ms refill interval — only **7 peers established sessions** in 130 seconds. We have zero visibility into why.

## What's in the logs today

**Peer cache**: Loaded 5,000 cached peers at startup:
```
cache_kept=5000, cache_seeded=5000, cache_expired=0, cache_total=5000
```

**Peer connections over the full run** — only 7 peers connected, trickling in slowly:

| Time | Peer (short) | Useful? |
|------|-------------|---------|
| t=0s | `0xc2a7` | disappeared from final report — likely disconnected early |
| t=0s | `0xd70b` | Yes (40 successes, but 8 receipt timeouts) |
| t=9s | `0x9d83` | No (0 successes, 8 failures) |
| t=20s | `0xc530` | No (0 successes, 8 failures) |
| t=72s | `0x23f0` | Yes (59 successes, best peer) |
| t=80s | `0x0262` | No (0 successes, 6 failures) |
| t=94s | `0x5916` | Yes (25 successes, second-best) |

**Resources timeline** — `peers_total` over 130s:
```
2 peers for first ~20s
2-3 peers until ~70s
grew to 5-9 by the end
```

## What's NOT in the logs

There is **zero visibility** into what's happening between "5,000 peers seeded" and "7 connected." Specifically, we don't log:

1. **Dial attempts** — How many of the 5,000 cached peers did reth try to connect to? We set `max_concurrent_dials=100`, but there's no log showing how many dials were attempted or are in progress.

2. **Dial failures** — Why did most dials fail? Timeout? Refused? Wrong chain? We don't know. Reth handles this internally.

3. **Handshake failures** — The peer watcher (`spawn_peer_watcher`) only logs `ActivePeerSession` events (successful connections). Failed handshakes are silently dropped. The only filter visible is the genesis hash check, which silently `continue`s.

4. **Discovery progress** — `spawn_peer_discovery_watcher` listens for DHT-discovered peers and caches them, but never logs anything.

5. **Head probe failures** — There's a `debug!("failed to probe peer head")` log, but none show up, meaning all head probes for the 7 connected peers succeeded.

## The connection funnel (blind spots)

```
5,000 cached peers seeded
  → ??? dials attempted
    → ??? handshake attempts
      → 7 ActivePeerSession events
        → 3 actually useful
```

## Plan: Add P2P discovery instrumentation

### Goal

Make the peer connection funnel visible so we can diagnose slow peer acquisition.

### Approach

TODO — design the instrumentation approach here.

Questions to answer:
- What does reth's `NetworkHandle` API expose about connection state?
- Can we get dial attempt / failure counts?
- Can we log periodic network state summaries?
- Should we add counters for handshake failures (genesis mismatch, etc.)?
- Should we emit structured events for the discovery watcher?
