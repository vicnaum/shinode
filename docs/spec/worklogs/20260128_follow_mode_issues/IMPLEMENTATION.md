# Implementation: Follow Mode Stall Fix

Implements `FIX_PLAN_v2.md`. Fixes the follow-mode stall where 2 stale peers
generated 12.2M ban events, monopolized the main loop, and starved healthy
peers of work.

## Changes

### Phase 1: Stop the Spin-Loop

#### 1a. Cooldown gate (`node/src/sync/historical/mod.rs`)

Before the stale-head check, if a peer is already cooling down (stale-head or
backoff), recycle it with a 500ms delay. This prevents the tight loop of
re-banning, re-logging, and re-probing that generated millions of events.

#### 1b. Drop truly stale peers (`node/src/sync/historical/mod.rs`)

Peers more than 10,000 blocks behind `min_required` are dropped entirely
instead of being banned and recycled. The `peer_feeder` will re-add them if
they reconnect with a fresh snapshot.

#### 1c. Refresh peer head_number from pool (`node/src/sync/historical/mod.rs`, `node/src/p2p/mod.rs`)

Added `PeerPool::get_peer_head()` to look up a peer's current head number.
The ingest pipeline now refreshes `peer.head_number` from the pool at three
sites:

1. `try_recv` loop (line ~624) — bulk receive
2. Blocking receive path (line ~643)
3. After peer selection, before stale-head check (line ~767)

This ensures `re_probe_peer_head()` updates (which write to the pool) actually
reach the pipeline's peer clones, so a re-probed peer won't be re-banned on
its next cycle.

### Phase 2: Log Deduplication (`node/src/logging/json.rs`)

The JSON log writer's background thread now deduplicates consecutive identical
records. Records are hashed by `(message, fields)` — ignoring `t_ms`, `level`,
`target`, `file`, and `line`. If the next record matches the previous one and
arrives within a 1-second window, a counter increments instead of writing. When
a different record arrives (or on timeout/flush/disconnect), the buffered
record is flushed with an optional `"count": N` field.

This provides defense-in-depth against log size explosions even if new
spin-loops are introduced.

## Verification

- `cargo build` passes
- `cargo test` passes (41 tests)
- Expected runtime behavior:
  - No 10GB log files
  - No "0 active peers" state from stale-peer monopolization
  - Stale peers (400k behind) logged once as "dropping", not millions of times
  - Log file shows `"count":N` for any remaining repeated messages
