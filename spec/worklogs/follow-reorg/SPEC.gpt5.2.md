## Live Sync Follow + Reorg Resilience Spec (v0.1.6)

### Context / problem statement
We have a fast historical ingest pipeline (`node/src/sync/historical/*`) that can:
- backfill a configured range
- persist headers/tx hashes/tx metadata/receipts into static files (NippyJar)
- serve a minimal RPC surface required by `rindexer` (`eth_blockNumber`, `eth_getBlockByNumber`, `eth_getLogs`, etc.)

However, the node does **not** yet implement a robust “live follow” loop:
- It can backfill to a computed “safe head” (usually `head - rollback_window`)
- It can then idle (RPC stays up), but it does not continuously ingest newly-safe blocks

We also need **reorg resilience** so the node remains on canonical chain:
- If we follow the tip (`rollback_window=0`) reorgs are expected and must be handled.
- Even if we follow a “safe head” (`rollback_window>0`) deep reorgs can still occur and must be handled (rare but possible).

This spec defines a detailed implementation plan for:
- **Follow mode**: continuously ingest new blocks as head advances.
- **Reorg detection**: detect canonical divergence while following.
- **Rollback + re-ingest**: tail-prune static segments back to the common ancestor and continue ingesting.
- **Compatibility with rindexer**: ensure RPC semantics and reorg behavior match rindexer’s expectations.

Reference docs:
- Reth canonical/reorg semantics: `spec/reth_kb/questions/Q018-canonical-chain-reorgs.md`, `Q020-rollback-semantics-reorgs.md`, `Q034-live-download-coordination.md`
- rindexer reorg expectations: `spec/repomix-output.rindexer.xml` (and `rindexer/core/src/indexer/reorg.rs` in repo form)

---

### Goals (what “done” means)
- **Follow loop exists**: node runs indefinitely, repeatedly ingesting until it is up to date, then keeps up with new heads.
- **Reorg resilience exists**:
  - Detect a reorg while following (canonical mismatch).
  - Roll back storage via static-file tail pruning to a common ancestor.
  - Resume ingest on the canonical chain.
- **RPC compatibility**:
  - `eth_blockNumber` reflects the node’s current canonical tip that it can serve (the last fully indexed block).
  - On reorg rollback, `eth_blockNumber` may decrease; rindexer must tolerate this.
  - `eth_getLogs` must only reflect the node’s canonical chain. After rollback, logs from reverted blocks must disappear from subsequent queries.
- **Operational safety**:
  - No partial corruption: storage rollback is atomic at the segment level (tail prune + metadata update).
  - Graceful shutdown: follow loop stops cleanly and flushes peer cache.
- **Test coverage**:
  - Unit tests for reorg detection + ancestor search logic.
  - Integration-ish tests: simulate a reorg against a fake header source and ensure we prune and re-ingest.

---

### Non-goals (this pass)
- CL / beacon API integration (finalized/safe head sources) and forkchoice validation.
- Heavy correctness hardening:
  - receiptsRoot validation against header
  - multi-peer majority vote for headers/receiptsRoot
- “Removed logs” tombstone semantics (we use delete-on-rollback; logs disappear after reorg).
- Log indexing build (address/topic0 index) – tracked separately in roadmap.
- WS subscriptions / pubsub.

---

### Terminology & invariants

#### Heads
- **Observed head**: `p2p_head` returned by current peers (can fluctuate).
- **Head seen**: `head_seen` persisted in storage, defined as **max observed head so far**.
  - Invariant: `head_seen` is monotonic non-decreasing.
- **Rollback window**: `rollback_window` from config (u64 blocks).
- **Target head (what we ingest to)**:
  - If `rollback_window > 0`: `target_head = head_seen - rollback_window`
  - If `rollback_window == 0`: `target_head = head_seen`
- **Configured end**: optional `--end-block` clamps `target_head`.

#### Storage checkpoints
- **Start block**: configured `--start-block` (and stored in `meta.json` as `start_block`).
- **Last indexed block**: `last_indexed_block` persisted in storage.
  - Invariant: storage contains canonical data for `start_block..=last_indexed_block` (except gaps explicitly represented as “missing rows”, if any).
  - Invariant: `eth_blockNumber` returns `last_indexed_block` (or 0 / error when empty, depending on existing behavior).

#### Canonicality
This node is stateless and does not execute blocks. Canonicality is determined by:
- Header parent links (hash chain continuity)
- Reorg handling via rollback + re-ingest

We are explicitly **best-effort** for now:
- P2P head is used as a head hint.
- (Future) CL integration can provide safe/finalized heads for stronger canonicality.

---

### Compatibility constraints (rindexer)

#### rindexer’s reorg model (important)
rindexer primarily detects reorgs by **block number rollback**:
- It periodically reads `eth_blockNumber` (“latest”).
- It indexes only up to `safe_block_number = latest_block_number - reorg_safe_distance` (mainnet default 12).
- If `eth_blockNumber` goes backwards within the safe distance, it treats it as an expected reorg.

#### What we must guarantee for rindexer
- `eth_blockNumber` is allowed to go backwards during a rollback (this is how rindexer notices reorgs).
- After rollback, `eth_getLogs` and `eth_getBlockByNumber` must serve the new canonical chain (old fork data must not persist above the ancestor).

#### Avoid “double safe distance”
If our node indexes only up to `target_head = head_seen - rollback_window`, and rindexer also subtracts its own safe distance, you can end up lagging by `rollback_window + rindexer_safe_distance`.

Recommended combos:
- **Best freshness with rindexer default settings**:
  - Run node with `--rollback-window 0` (follow tip)
  - Let rindexer default safe distance (12) provide indexing safety
  - Node still must handle reorgs because it writes tip blocks.
- **Safer node (less reorg churn) + adjust rindexer**:
  - Run node with `--rollback-window 12` (or 64)
  - Configure rindexer `reorg_safe_distance = 0` (so it doesn’t double-lag)

This spec must support both modes (rollback_window=0 and >0).

---

### Design overview (target architecture)

We introduce a dedicated follow orchestrator in the historical subsystem:

```
node/src/sync/historical/
  follow.rs   # NEW: follow loop orchestration
  reorg.rs    # NEW: reorg detection + ancestor search + rollback
```

The follow orchestrator will:
- continuously sample head (via P2P peer view)
- compute `target_head`
- plan the next ingest range from storage checkpoints
- run the existing `run_ingest_pipeline(...)` for that range
- when up to date, wait and continue (do NOT exit)

Reorg logic will:
- detect canonical mismatch before ingesting a new range (cheap “parent check”)
- on mismatch, find common ancestor, rollback storage, and restart planning

The orchestrator must be stoppable:
- use a shutdown signal (e.g., `tokio::sync::watch::Receiver<bool>` or `CancellationToken`)
- shutdown must flush peer cache and stop background work

---

### Follow loop semantics (exact behavior)

#### 1) Inputs
- `Storage` (Arc)
- `PeerPool` (Arc)
- `NodeConfig`
- Optional progress hooks (existing: `ProgressReporter`, `SyncProgressStats`)
- Optional benchmark stats (not required for follow, but keep hooks consistent)

#### 2) Loop pseudocode (reference)

```
loop {
  // A) Observe head (best effort)
  observed_head = pool.best_head()
  head_seen = max(storage.head_seen(), observed_head)
  storage.set_head_seen(head_seen)

  // B) Compute target head
  target_head = clamp_end_block(head_seen - rollback_window)

  // C) Determine next start
  last = storage.last_indexed_block()
  start = max(config.start_block, last+1)

  // D) If nothing to do: up to date, sleep and continue
  if start > target_head:
     mark_status_up_to_date()
     sleep(follow_poll_interval)
     continue

  // E) Reorg preflight: ensure range connects to stored head
  if last exists:
     if parent_mismatch(last, start):
         handle_reorg(last, start)
         continue // after rollback, recompute from checkpoints

  // F) Ingest next range (bounded)
  range = start..=target_head
  outcome = run_ingest_pipeline(storage, pool, config, range, head_seen, ...)

  // G) Continue immediately (head may have advanced while ingesting)
}
```

#### 3) Follow polling interval
We need a configurable sleep interval when up to date.

Defaults:
- mainnet: 1s polling is fine (cheap)
- optional: dynamic based on observed block time (future)

Planned flag:
- `--follow-poll-ms <u64>` (default 1000)

(If we prefer fewer flags, we can bake 1s as a constant, but a flag is better for operators.)

---

### Reorg detection (exact behavior)

We must detect a reorg **before writing new blocks**, to avoid polluting storage further.

#### Primary reorg signal: parent hash mismatch
When we are about to ingest `start = last_indexed + 1`:
- Read stored header at `last_indexed`, compute its sealed hash.
- Fetch network header at `start` (from peers), compare:
  - `network_header.parent_hash` vs `stored_last_hash`
- If mismatch → canonical divergence → reorg.

This is O(1) network header fetch per ingest iteration, and avoids relying on head numbers.

#### Additional signal (optional): head number rollback
Observed head may go backward due to peer churn. We **do not** treat this alone as a reorg signal.
We only treat it as informational and continue using `head_seen` as monotonic max.

---

### Reorg handling (rollback + resume)

#### Step 0: When a mismatch is detected
We enter a “reorg resolution” procedure:
- Find the highest common ancestor between:
  - stored chain (our persisted headers)
  - network canonical chain (as reported by peers)
- Roll back storage to that ancestor by pruning static segments
- Resume normal follow loop planning

#### Step 1: Choose a reorg search depth
We need a bounded search to avoid pathological behavior.

Introduce a max search depth:
- `REORG_SEARCH_MAX_DEPTH_DEFAULT = max(rollback_window, 2048)`
- Optional CLI: `--reorg-max-depth <u64>` (default `2048`)

Rationale:
- If `rollback_window` is 0 (tip follow), we still want a reasonable max search.
- 2048 headers is cheap to fetch and compare, and should cover realistic deep reorg bounds.

#### Step 2: Find common ancestor (highest matching block)
We will compare stored header hashes vs network header hashes for a window:

- Let `high = last_indexed`
- Let `low = max(config.start_block, high - reorg_max_depth)`
- Fetch network headers for `[low..=high]` (chunked at max request size).
- For each header number `n` in the window:
  - `stored_hash(n)`:
    - read stored header at `n`
    - compute `SealedHeader::seal_slow(header).hash()`
  - `network_hash(n)`:
    - compute `SealedHeader::seal_slow(network_header).hash()`
  - Compare hashes
- The common ancestor is the **largest n** where `stored_hash(n) == network_hash(n)`.

#### Step 3: If no match in the window
This indicates one of:
- very deep reorg beyond `reorg_max_depth`
- our storage is corrupted
- we’re querying inconsistent peers (non-canonical / lying / partial)

Fallback policy (v0.1 minimal and safe):
- Roll back to `rollback_to = low.saturating_sub(1)`:
  - If `low == config.start_block`, this effectively clears all stored data (within our retained range).
  - Then we re-ingest from `config.start_block`.
- Emit a high-severity log line and increment a counter.

(We avoid halting the node; recovery is preferred.)

#### Step 4: Rollback write semantics
Rollback must be implemented as:
- `storage.rollback_to(ancestor_number)`
  - tail-prune static segments past ancestor
  - update `meta.json` (`last_indexed_block`)

After rollback:
- Next follow loop iteration will compute a new start based on `last_indexed_block`.
- We then ingest forward again.

#### Step 5: Metrics & observability
We must record:
- reorg count
- reorg depth (`old_last_indexed - ancestor`)
- time spent resolving reorg
- whether fallback “window miss” happened

Add log line fields:
- `reorg_old_tip`, `reorg_new_tip`, `reorg_depth`, `ancestor`, `search_low`, `search_high`, `fallback_used`

---

### Changes required in codebase (detailed checklist)

#### A) New modules + APIs
- [ ] Add `node/src/sync/historical/follow.rs`
  - [ ] `pub struct FollowConfig { poll_ms: u64, reorg_max_depth: u64 }`
  - [ ] `pub async fn run_follow_loop(storage: Arc<Storage>, pool: Arc<PeerPool>, config: &NodeConfig, follow: FollowConfig, progress: Option<Arc<dyn ProgressReporter>>, stats: Option<Arc<SyncProgressStats>>) -> Result<()>`
  - [ ] Must loop forever until shutdown signal is triggered.
- [ ] Add `node/src/sync/historical/reorg.rs`
  - [ ] `pub enum ReorgResolution { NoReorg, RolledBack { ancestor: u64, depth: u64, fallback: bool } }`
  - [ ] `pub async fn ensure_contiguous_or_reorg(storage: &Storage, pool: &PeerPool, start: u64, last_indexed: u64, reorg_max_depth: u64, start_block: u64) -> Result<ReorgResolution>`
  - [ ] Implement:
    - [ ] parent mismatch detection
    - [ ] ancestor search
    - [ ] rollback invocation

#### B) P2P helpers (minimal surface)
- [ ] Add a small helper to fetch headers by number range for reorg checking.
  - Option 1 (preferred): implement in `p2p/mod.rs`:
    - [ ] `pub(crate) async fn fetch_headers_for_peer(peer: &NetworkPeer, range: RangeInclusive<u64>) -> Result<Vec<Header>>`
  - Option 2: reuse existing `request_headers_chunked` on a selected peer.

The key requirement: reorg module must be able to fetch headers `[low..=high]` reliably.

#### C) Main wiring (node entrypoint)
- [ ] Update `node/src/main.rs` non-benchmark path:
  - [ ] Replace the “single ingest then idle” behavior with the follow loop.
  - [ ] Keep RPC server running throughout follow.
  - [ ] Spawn follow as a background task and await ctrl-c:
    - on ctrl-c: stop follow loop, flush peer cache, stop RPC.
- [ ] Add CLI flags:
  - [ ] `--follow` (optional; decide default behavior)
    - Option A: follow by default when `--head-source p2p` and `--benchmark disabled`
    - Option B: require explicit `--follow`
  - [ ] `--follow-poll-ms` (default 1000)
  - [ ] `--reorg-max-depth` (default 2048)

Decision note:
- For MVP long-running service, **follow-by-default** is preferred.
- If we keep non-follow mode, it should exit after catching up (useful for batch runs).

#### D) Storage requirements
- [ ] Confirm existing APIs are sufficient:
  - [ ] `Storage::block_header(n)` and `Storage::rollback_to(ancestor)`
  - [ ] `Storage::last_indexed_block()` and `Storage::set_last_indexed_block()`
  - [ ] `Storage::set_head_seen(head)`
- [ ] If needed, add `Storage::block_hash(n)` helper that computes sealed header hash.

---

### Test plan (detailed)

#### Unit tests (fast, deterministic)
- [ ] Add tests for “parent mismatch triggers reorg resolution”
  - Given stored headers 0..=10 and network header 11 with wrong parent → mismatch detected
- [ ] Add tests for “ancestor search finds highest match”
  - Stored: 0..=10, Network: identical 0..=6 and diverges at 7..=10
  - Expect ancestor=6
- [ ] Add tests for “window miss fallback”
  - With `reorg_max_depth=3`, last_indexed=10, match only at 0..=5 → no match in window [7..=10]
  - Expect fallback rollback_to=6 (low-1) and re-ingest from 7

Implementation approach for tests:
- Create a small `FakeHeaderSource` that can return headers by number.
- In reorg module, accept an abstracted `HeaderSource` (trait) for tests OR construct a fake `PeerPool` wrapper.

#### Integration-ish tests (storage + rollback behavior)
- [ ] Write headers into storage for a range (e.g., 0..=20) using the normal ingest write path.
- [ ] Simulate a reorg by providing a fake “network header chain” that diverges at N and returns a different header hash for N..=20.
- [ ] Run `ensure_contiguous_or_reorg` and verify:
  - storage pruned to ancestor
  - `last_indexed_block` updated
  - after a subsequent “ingest”, data exists again on the canonical chain.

#### Manual verification checklist
- [ ] Run node with `--rollback-window 0` and confirm it continues ingesting beyond initial catch-up (does not idle forever).
- [ ] Run node with `--rollback-window 64` and confirm it continues ingesting newly-safe blocks as head advances.
- [ ] Point rindexer at it and confirm it keeps indexing (no permanent stall on reorg).

---

### Acceptance criteria / “Definition of Done”
- [ ] Node keeps ingesting in steady state (follow mode works).
- [ ] Reorg parent mismatch is detected and causes rollback + re-ingest (tested).
- [ ] Storage tail-prune rollback works end-to-end in follow flow (tested).
- [ ] `eth_blockNumber` may decrease on rollback and rindexer tolerates it.
- [ ] `cargo test --manifest-path node/Cargo.toml` passes.
- [ ] Documentation updated:
  - [ ] ROADMAP v0.1.6 checkboxes updated after implementation
  - [ ] README mentions follow mode and rollback_window interaction with rindexer safe distance

