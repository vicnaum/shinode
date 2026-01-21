## Live Follow + Reorgs Spec (v0.1.6)

### Context / problem statement
We have a fast historical backfill pipeline that stops once it reaches the safe head
(`head - rollback_window`). The node then idles and does **not** keep up with new heads.
We also do not yet implement explicit reorg handling while following the chain.

This spec defines:
- **Live follow mode**: keep ingesting new blocks as the head advances.
- **Reorg handling**: detect reorgs from P2P headers and roll back static files
  within a bounded rollback window.
- **Compatibility**: behavior must be compatible with rindexer’s reorg model
  (block number rollback detection + safe distance).

This plan should **not** reintroduce legacy ingestion code; we only add the missing
follow + reorg logic to the modern pipeline.

---

### Goals (what “done” means)
- **Follow loop**: after backfill reaches safe head, keep polling for new heads
  and ingest any new safe ranges indefinitely.
- **Reorg detection**: detect parent-hash mismatches at the ingestion boundary and
  find a common ancestor within `rollback_window`.
- **Rollback**: tail-prune static files above the common ancestor, update checkpoints,
  and continue following.
- **rindexer compatibility**:
  - `eth_blockNumber` reflects the **last indexed block** (can decrease on reorg).
  - `eth_getLogs` returns **canonical** logs only (no removed logs needed).
- **Benchmark unaffected**: benchmark modes remain “fetch once, exit”.

---

### Non-goals (this pass)
- CL/Engine integration (safe/finalized forkchoice inputs).
- Reorg handling **beyond** `rollback_window` (will log and re-bootstrap if needed).
- Log tombstones / `removed=true` handling.
- Serving new RPC methods.

---

### Terminology
- **Unsafe head**: best head observed from P2P.
- **Safe head**: `unsafe_head - rollback_window`.
- **Last indexed**: last block number fully persisted in storage.
- **Canonical header hash**: `SealedHeader::seal_slow(header).hash()`.

---

### Design overview

#### 1) Live follow loop (high-level)
- Start in **backfill** mode:
  - Compute `safe_head = head_at_startup - rollback_window`.
  - Ingest `start_block..=safe_head`.
- Switch to **follow** mode:
  - Periodically sample the current head from P2P.
  - Compute `safe_head`.
  - If `last_indexed < safe_head`, ingest `last_indexed+1..=safe_head`.
  - If `last_indexed >= safe_head`, sleep and repeat.

#### 2) Reorg detection point
Before ingesting a new range:
- Read stored header for `last_indexed`.
- Fetch header for `last_indexed` from the network.
- If hashes differ, a reorg occurred and we must find a common ancestor.

#### 3) Common ancestor search (bounded)
Search backward from `last_indexed` down to `last_indexed - rollback_window`:
- For each block number `n`:
  - Load stored header from storage.
  - Fetch network header by number.
  - Compare hashes. First match is the common ancestor.
- If none found, abort follow and emit a warning (requires manual reset or
  a wider rollback window).

#### 4) Rollback action
Once `ancestor` is found:
- `storage.rollback_to(ancestor)` (tail-prune static segments).
- Set `last_indexed = ancestor`.
- Continue follow by re-ingesting forward to `safe_head`.

---

### rindexer compatibility constraints
- rindexer detects reorgs by **block number rollback**.
- rindexer indexes only up to **safe head** (`latest - reorg_safe_distance`).

Implications for us:
- Keep `rollback_window` conservative (default 64 is fine).
- `eth_blockNumber` should always match `last_indexed`.
- After rollback, `eth_getLogs` for affected ranges must reflect the new canonical
  chain (pruned logs disappear).

---

### Reth reference (conceptual alignment)
Reth’s approach (simplified for us):
- Maintain canonical chain and detect reorgs by walking parents to a common ancestor.
- Unwind/rollback to ancestor on disk when canonical changes.
We replicate this idea without execution/state, only headers + receipts + static files.

---

### Required new behaviors (detailed)

#### Follow loop logic
- **Poll interval**: configurable, default `follow_poll_interval_ms = 3000`.
- Loop steps:
  1) Sample `unsafe_head` via P2P.
  2) Compute `safe_head = unsafe_head - rollback_window`.
  3) Read `last_indexed` from storage.
  4) If `last_indexed < safe_head`, ingest `last_indexed+1..=safe_head`.
  5) Else sleep for `follow_poll_interval_ms`.

#### Reorg detection logic
- If `last_indexed` is `None`, start from `start_block`.
- Else:
  - Fetch stored header at `last_indexed`.
  - Fetch network header at `last_indexed`.
  - If hashes match, proceed with ingest.
  - If hashes mismatch, run ancestor search.

#### Ancestor search
- Scan descending:
  - `candidate = last_indexed`
  - `min = last_indexed.saturating_sub(rollback_window)`
- Fetch headers from network in descending batches (e.g., 32 at a time).
- For each `candidate`:
  - Compare stored header hash vs network header hash.
  - First match is the ancestor.
- No match:
  - Log error and stop follow mode (requires operator intervention).

---

### Implementation plan (checkboxes)

#### Phase 1 — Spec + CLI config plumbing
- [ ] Add `follow_poll_interval_ms` to `NodeConfig` with a sensible default (e.g. 3000).
- [ ] Document the new follow behavior in `README.md`.

#### Phase 2 — Add reorg detection helpers
- [ ] Add a reorg helper module (e.g., `node/src/sync/historical/reorg.rs`):
  - [ ] `fetch_header_hashes(range, peer_pool) -> HashMap<u64, B256>`
  - [ ] `find_common_ancestor(storage, peer_pool, last_indexed, rollback_window) -> Option<u64>`
  - [ ] `needs_reorg(storage, peer_pool, last_indexed) -> bool`
- [ ] Unit tests for:
  - [ ] ancestor search finds a common ancestor
  - [ ] ancestor search returns `None` when outside rollback window

#### Phase 3 — Follow loop integration
- [ ] In `main.rs` follow mode:
  - [ ] after backfill, keep polling for `safe_head`
  - [ ] call reorg detection before ingesting new ranges
  - [ ] on reorg: rollback and continue
  - [ ] on no new range: sleep and retry
- [ ] Ensure benchmark modes are unchanged (no follow loop).

#### Phase 4 — Metrics + logging
- [ ] Add counters:
  - [ ] `reorgs_detected_total`
  - [ ] `reorg_rollback_depth`
  - [ ] `follow_idle_loops`
- [ ] Log reorg events with:
  - [ ] last indexed
  - [ ] ancestor
  - [ ] rollback depth

#### Phase 5 — Integration tests
- [ ] Add tests simulating:
  - [ ] normal follow progress (head advances, no reorg)
  - [ ] reorg within rollback window
  - [ ] reorg beyond rollback window (expect graceful stop)
- [ ] Validate that `eth_blockNumber` rolls back after reorg.

---

### Acceptance criteria
- [ ] Node follows new heads indefinitely after backfill.
- [ ] Reorg within `rollback_window` triggers rollback and continues.
- [ ] Reorg beyond `rollback_window` stops follow with a clear error log.
- [ ] `eth_blockNumber` equals `last_indexed` and can decrease on reorg.
- [ ] rindexer can index without errors using the safe head.

---

### Risks and mitigations
- **Risk: false reorg detection due to peer inconsistency**
  - Mitigation: compare against a stable peer or retry header fetch with a second peer.
- **Risk: rollback window too small**
  - Mitigation: allow configuration; document recommended values (12–64).
- **Risk: live follow overload**
  - Mitigation: poll interval + bounded pipeline + existing backpressure.

