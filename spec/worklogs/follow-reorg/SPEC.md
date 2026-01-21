# Live Follow + Reorg Handling Spec (Stateless History Node v0.1.6)

This spec defines how to add **continuous follow** and **reorg resilience** to the existing `node/src/sync/historical/*` ingest pipeline and `Storage` (NippyJar static files).

It is intentionally scoped to the current repository shape and APIs, and should not reintroduce older/removed modules.

---

## 0) Context (current state in this repo)

What we have today:
- **Historical ingest pipeline**: `node/src/sync/historical/mod.rs::run_ingest_pipeline(...)` can ingest a configured block range using multiple peers and write to storage via a batched DB writer (`db_writer.rs`).
- **Storage**: `node/src/storage/mod.rs::Storage` persists:
  - `meta.json` (`last_indexed_block`, `head_seen`, etc.)
  - static segments (`headers`, `tx_hashes`, `tx_meta`, `receipts`, `block_sizes`)
  - supports rollback via `Storage::rollback_to(ancestor_number)` (tail-prunes segments + updates `meta.json`).
- **RPC**: `node/src/rpc/mod.rs` serves:
  - `eth_blockNumber` as `Storage::last_indexed_block()` (can decrease after rollback)
  - `eth_getBlockByNumber` and `eth_getLogs` from stored headers/tx hashes/receipts.

What is missing:
- **Live follow**: after the node catches up, it stops ingesting and just idles (RPC stays up).
- **Explicit reorg handling while following**: no “preflight” continuity check, no ancestor search, no rollback + re-ingest loop.

Critical implementation reality (must be addressed explicitly):
- The current P2P peer “head numbers” are only computed **once** on session establishment (`node/src/p2p/mod.rs`), and are **not updated** as the chain advances.
- Therefore, “poll `pool.best_head()`” is **not sufficient** for a true live follow loop. We must add a minimal **moving head discovery** mechanism.

---

## 1) Goals (Definition of Done)

### Functional
- **Continuous follow**: node runs indefinitely and keeps ingesting newly available blocks.
- **Reorg resilience**:
  - Detect canonical divergence before writing new blocks (cheap “parent mismatch” preflight).
  - Find a common ancestor within a bounded search depth.
  - Roll back storage to the ancestor using `Storage::rollback_to`, then re-ingest forward.
- **No corruption**: rollback + resume must leave storage in a consistent, restart-safe state.

### Compatibility (rindexer-first)
- `eth_blockNumber` **equals** `last_indexed_block` and **may decrease** on rollback.
- `eth_getLogs` returns **canonical logs only** (delete-on-rollback; no tombstones / `removed=true` recovery required for v0.1).

### Constraints
- **Benchmarks unchanged**: `--benchmark probe` and `--benchmark ingest` remain “run once, exit”.
- **Minimum code**: reuse the existing ingest pipeline; only add the missing follow + head discovery + reorg logic.

---

## 2) Non-goals (this pass)
- Consensus Layer / Engine API / beacon API integration (safe/finalized forkchoice).
- ReceiptsRoot verification and multi-peer majority validation (can be added later as “trust hardening”).
- “Removed logs” semantics (tombstones), WS subscriptions, or extra RPC methods.

---

## 3) Terminology
- **Stored tip**: `last_indexed_block` persisted in `meta.json`. This is what RPC serves.
- **Observed head**: best-effort estimate of the current chain head from the P2P network.
- **Target head**: the highest block number we will attempt to ingest this iteration.
  - For `rollback_window > 0`: `target_head = observed_head - rollback_window`
  - For `rollback_window == 0`: `target_head = observed_head` (tip-follow)
- **Rollback window** (`rollback_window`): product knob that trades freshness for reduced reorg churn.

---

## 4) rindexer compatibility (very important)

rindexer live indexing computes:
- `safe_block_number = latest_block_number - reorg_safe_distance`
  - mainnet default is **12**
  - non-mainnet default is **64** (`rindexer/core/src/indexer/reorg.rs::reorg_safe_distance_for_chain`)

### Avoid “double safe distance”
If our node reports `eth_blockNumber` that is already lagging by `rollback_window`, and rindexer subtracts `reorg_safe_distance`, the effective lag is:

\[
\text{effective lag} \approx \text{rollback\_window} + \text{rindexer reorg\_safe\_distance}
\]

Recommended operating modes:
- **Freshest (recommended for rindexer defaults)**:
  - Node: `--rollback-window 0` (tip-follow)
  - rindexer: keep defaults (mainnet 12)
- **Less churn (operator prefers fewer rollbacks)**:
  - Node: `--rollback-window 12` or `64`
  - rindexer: set `reorg_safe_distance = 0` (so it doesn’t double-lag)

This spec must support both.

---

## 5) Design overview

We add three pieces:

1) **Observed head discovery** (NEW)
- A small, bounded, best-effort routine that estimates the moving head using `GetBlockHeaders` by **number**.
- This avoids relying on “peer head numbers” that do not update over time.

2) **Follow loop orchestration** (NEW)
- A loop that:
  - discovers `observed_head`
  - computes `target_head`
  - plans `start..=target_head` from storage checkpoints
  - runs the existing ingest pipeline for that range
  - sleeps when up-to-date

3) **Reorg detection + rollback** (NEW)
- A cheap preflight check before ingesting a new range:
  - fetch the next header (`start = last_indexed + 1`)
  - compare its `parent_hash` to the stored sealed hash at `last_indexed`
- If mismatch: find ancestor, rollback, restart planning.

---

## 6) Observed head discovery (P2P-only, best-effort)

### Why we need this
The node currently computes peer head numbers only once per session, so `pool.best_head()` can stall indefinitely. Live follow requires a head estimate that advances without requiring peer churn.

### Head discovery algorithm (minimal, good-enough)

Inputs:
- `pool: PeerPool`
- `baseline: u64` (typically `last_indexed_block.unwrap_or(start_block)`)
- `probe_peers: usize` (e.g., 3)
- `probe_limit: usize` (e.g., 1024 headers per probe)

Procedure:
- For each of up to `probe_peers` peers:
  - request headers for `start = baseline + 1`, `limit = probe_limit`
  - if the peer returns `k > 0` headers, the peer proves `head >= (baseline + k)`
  - track the maximum “last header number” across probes
- Return `observed_head = max(last_header_number_over_probes, baseline)`

Notes:
- We intentionally accept that peers can be behind or inconsistent; taking the max across a few peers is enough as a head hint.
- This head estimate is not a canonicality guarantee; correctness is handled by the reorg mechanism + bounded rollback.

### Interaction with the existing scheduler (must be handled)
The ingest scheduler limits assignments by a `peer_head` value (`PeerWorkScheduler::next_batch_for_peer(peer_id, peer_head, ...)`).

Today `run_ingest_pipeline` passes `NetworkPeer.head_number`, which is only set once at session establishment and can go stale. For live follow this can **stall scheduling** even if peers would serve newer blocks.

**Minimal fix (recommended):**
- When scheduling ingest batches, pass a *global cap* (e.g. `target_head`) as `peer_head` instead of the stale per-peer `head_number`.
- Rely on existing retry/rotation to handle peers that are actually behind.

**Alternative (more work):**
- Refresh per-peer head estimates periodically and update the pool.

---

## 7) Reorg detection + resolution

### 7.1 Preflight reorg detection (O(1))
Before ingesting a range starting at `start = last_indexed + 1`:
- Load `stored_last_header` from `Storage::block_header(last_indexed)`.
- Compute `stored_last_hash = SealedHeader::seal_slow(stored_last_header).hash()`.
- Fetch `network_start_header` (header at `start`) from P2P (try a few peers).
- If `network_start_header.parent_hash != stored_last_hash` → **reorg detected**.

Rationale:
- This is cheaper and less error-prone than comparing arbitrary “same-height hashes” against a single peer.
- It matches the same conceptual boundary check used by reth (parent connectivity determines whether the head extends the chain).

### 7.2 Common ancestor search (bounded)
Inputs:
- `high = last_indexed`
- `low = max(config.start_block, high - reorg_max_depth)`
- A chosen “anchor peer” (the peer that produced the mismatching `network_start_header`, or another healthy peer)

Procedure:
- Fetch network headers for `[low..=high]` (chunked).
- For `n` descending from `high` to `low`:
  - `stored_hash(n)` = sealed hash of `Storage::block_header(n)` (if missing, treat as no match)
  - `network_hash(n)` = sealed hash of network header `n`
  - First match (largest `n`) is the ancestor.

### 7.3 Rollback
When ancestor `a` is found:
- Call `Storage::rollback_to(a)`
- Resume follow planning (next `start` becomes `a + 1`)

### 7.4 Deep reorg / no ancestor found
We must pick a policy. Recommended (robust, self-healing):
- If no match in `[low..=high]`, roll back to `low - 1` (or clear to “empty” if `low == start_block`) and re-ingest forward.

Alternative (strict, operator-visible):
- Stop follow mode and require manual intervention.

This is an open decision (see Questions).

---

## 8) Follow loop (exact semantics)

### Inputs
- `Storage`
- `PeerPool`
- `NodeConfig`
- `follow_poll_ms: u64` (default 1000)
- `reorg_max_depth: u64` (default: `max(rollback_window, 2048)`)

### Loop
Repeat until shutdown:
1) Load checkpoints:
   - `last_indexed = storage.last_indexed_block()`
2) Compute `baseline = max(config.start_block, last_indexed.unwrap_or(config.start_block))`
3) Discover `observed_head` via head discovery (Section 6).
4) Compute `target_head`:
   - if `rollback_window > 0`: `target_head = observed_head - rollback_window`
   - else: `target_head = observed_head`
   - clamp by `--end-block` if provided
5) Compute `start = max(config.start_block, last_indexed.map(|n| n + 1).unwrap_or(config.start_block))`
6) If `start > target_head`: sleep `follow_poll_ms` and continue.
7) Reorg preflight:
   - if `last_indexed.is_some()`: run Section 7.1
   - if reorg: resolve + rollback; continue loop
8) Run ingest for `range = start..=target_head` using existing `run_ingest_pipeline(...)`.
9) Immediately loop again (head may have advanced).

---

## 9) Code changes (intended minimal diffs)

### New modules
- `node/src/sync/historical/follow.rs`
  - `run_follow_loop(...)`
- `node/src/sync/historical/reorg.rs`
  - `ensure_contiguous_or_rollback(...)`
  - `find_common_ancestor(...)`
- `node/src/p2p/head.rs` (or keep in `p2p/mod.rs` if preferred)
  - `estimate_observed_head(...)` using header probes

### Wiring
- `node/src/main.rs`
  - In normal mode (non-benchmark), replace “ingest until up-to-date then idle” with follow loop:
    - RPC stays running
    - follow loop runs until ctrl-c

### CLI flags (optional)
- `--follow-poll-ms <u64>` (default 1000)
- `--reorg-max-depth <u64>` (default 2048 or `max(rollback_window, 2048)`)

If we want “follow by default” (recommended), no `--follow` flag is necessary; we can retain an opt-out like `--no-follow` only if needed.

---

## 10) RPC behavior during follow + reorg

No new endpoints required for rindexer.

Required semantics:
- `eth_blockNumber`: always returns `Storage::last_indexed_block` (hex). May decrease after rollback.
- `eth_getBlockByNumber("latest", false)`: uses the same `last_indexed_block` and returns `null` if empty.
- `eth_getLogs`: queries only stored canonical data; after rollback, logs from reverted blocks are gone.

---

## 11) Metrics + logging (minimum)

Add log lines (INFO/WARN) for:
- head discovery samples and chosen `observed_head`
- follow idle cycles (up to date)
- reorg detected (old tip, start, mismatch)
- rollback performed (ancestor, depth, search window, fallback used)

Add counters/gauges (either existing metrics module or simple tracing):
- `follow_observed_head` (gauge)
- `follow_target_head` (gauge)
- `reorgs_detected_total` (counter)
- `reorg_rollback_depth` (histogram or counter w/ depth)

---

## 12) Test plan

### Unit tests (fast)
- Reorg preflight: parent mismatch triggers resolution.
- Ancestor search: returns highest matching ancestor.
- Deep reorg fallback policy (whatever we choose).

Recommended testing shape:
- Introduce a small `HeaderSource` trait for the reorg module so tests can inject a fake network header map without needing P2P wiring.

### Integration-ish tests (storage + rollback)
- Write a short contiguous chain into `Storage` using `write_block_bundle_batch`.
- Provide a fake “network” chain that diverges at N.
- Run reorg resolution and assert:
  - storage pruned to ancestor
  - `last_indexed_block` updated accordingly

---

## 13) Open questions (need your input)

1) **Deep reorg policy**: if no ancestor is found within `reorg_max_depth`, do we:
   - (A) stop follow mode and require manual intervention, or
   - (B) auto-rebootstrap by rolling back further and re-ingesting?

2) **Default freshness mode**: do you want the node’s *default* to be:
   - (A) `rollback_window=64` (current default; less churn but can “double lag” with rindexer defaults), or
   - (B) `rollback_window=0` (freshest; relies on reorg handling + indexer safe distance)?

3) **Reorg detection quorum**: for the preflight header at `start`, should we require:
   - (A) “first peer that returns a header” (simplest), or
   - (B) “2-of-3 peers agree on the header hash” (more robust against inconsistent peers)?

