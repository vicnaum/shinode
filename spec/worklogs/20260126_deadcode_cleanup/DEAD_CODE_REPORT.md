## Unified Dead/Legacy/Repeated Code Report (Claude + Codex + Gemini + GPT-5.2)

Generated: 2026-01-26

### How to use this checklist

- Each item is **tickable** (`- [ ]`) so you can go through cleanup one-by-one.
- Where the four reports disagree (or a report appears outdated), the item is still listed but marked as **false positive / already removed** with a short note.
- For remaining items that require an explicit choice, see `deadcode/DEAD_CODE_DECISIONS.md`.

### Legend (classification)

- **Confirmed-dead**: No non-test call sites (or only referenced by itself).
- **Test-only in prod module**: Used only by tests but compiled in non-test builds.
- **Stub / scope decision**: Present but returns `None`/empty; keep only if it matches the product roadmap.
- **Refactor (duplication)**: Not dead, but repeated/boilerplate.
- **False positive / already removed**: Mentioned by at least one report, but not present or actually used.

---

## `node/src/metrics.rs`

- [x] `lag_to_head()` — **Confirmed-dead**
  - **Claude**: Unused; has tests but not called.
  - **Codex**: High-confidence dead (no call sites).
  - **Gemini**: Unused helper.
  - **GPT-5.2**: Never referenced.
  - **Consensus**: **Strong (4/4)** → safe to remove or wire up.
  - **Status**: Removed from `node/src/metrics.rs` (tests deleted).

- [x] `rate_per_sec()` — **Confirmed-dead** + **Refactor (duplication)**
  - **Claude**: Unused; has tests but not called.
  - **Codex**: High-confidence dead.
  - **Gemini**: Unused; duplicated by `sync/historical/stats.rs::rate_per_sec`.
  - **GPT-5.2**: Unused; also flags duplicate helper with `stats.rs`.
  - **Consensus**: **Strong (4/4)** → delete or reuse; also decide where the “rate/sec” helper should live.
  - **Status**: Removed from `node/src/metrics.rs` (tests deleted).

---

## `node/src/main.rs`

- [x] `total_blocks_to_head()` — **Test-only in prod module**
  - **Claude**: Not mentioned.
  - **Codex**: Dead code (unused helper).
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: “Never referenced.”
  - **Consensus**: **Moderate (2/4)**. Current tree: used in tests, not used in runtime path.
  - **Suggested action**: Move it into the `#[cfg(test)]` module, or remove tests’ dependency on it.
  - **Status**: Gated with `#[cfg(test)]`.

- [x] Repeated `PeerHealthSummary` construction (3 nearly identical blocks) — **Refactor (duplication)**
  - **Claude**: Flags exact duplication 3×; recommends `From<&PeerHealthDump> for PeerHealthSummary` or helper.
  - **Codex**: Not mentioned.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter** (still a clear cleanup win).
  - **Status**: Deduplicated in `node/src/main.rs` via a small local macro used by all three maps.

- [x] Throughput/ETA sliding-window logic duplicated (main vs historical) — **Refactor (duplication)**
  - **Claude**: Not mentioned.
  - **Codex**: Not mentioned.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Flags duplicated `VecDeque<(Instant, u64)>` window logic in `main.rs` vs `sync/historical/mod.rs`; suggests factoring into a helper.
  - **Consensus**: **Single-reporter** (optional refactor).
  - **Status**: Re-check shows only `main.rs` has `VecDeque<(Instant, u64)>` now; no duplicate in `sync/historical/mod.rs`.

---

## `node/src/p2p/mod.rs`

- [x] `SelectorPeerId` / `PeerSelector` / `RoundRobinPeerSelector` — **Test-only in prod module**
  - **Claude**: Legacy; not used (likely test scaffolding).
  - **Codex**: Unused selector scaffolding.
  - **Gemini**: Testing/future peer selection abstractions bypassed by `PeerPool`.
  - **GPT-5.2**: Test-only abstractions living in non-test code.
  - **Consensus**: **Strong (4/4)** → move behind `#[cfg(test)]` or remove + rewrite tests.
  - **Status**: Gated with `#[cfg(test)]`.

- [x] `request_headers_chunked()` / `request_bodies_chunked_partial()` / `request_receipts_chunked_partial()` — **False positive / already correctly gated**
  - **Claude**: Identifies them as test-only wrappers.
  - **Codex**: Not mentioned.
  - **Gemini**: Notes the partial receipts helper is test-only.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: These are already `#[cfg(test)]` in current tree.

- [x] `request_receipts_legacy()` fallback path — **Not dead (but “legacy-looking”)**
  - **Claude**: Not mentioned.
  - **Codex**: Calls out legacy eth-protocol compatibility paths as “worth calling out”.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Weak** (1/4). Current tree: this fallback is actively used for non-eth/69,70 peers.
  - **Suggested action**: Keep, but document why it exists (or decide to drop older peers explicitly).
  - **Status**: Decision: **Keep for now** (no code change).

---

## `node/src/storage/mod.rs`

- [x] `LogIndexEntry` — **Stub / scope decision**
  - **Claude**: “Appears to be truly unused” (no external references beyond definition).
  - **Codex**: Implies log-index functionality is stubbed/unused.
  - **Gemini**: Notes log index not implemented; logs derived from receipts.
  - **GPT-5.2**: Calls it part of “types that imply features you don’t actually serve”.
  - **Consensus**: **Strong that it’s unused by current runtime** (but still referenced by stub APIs).
  - **Suggested action**: Either delete the log-index API surface (and this type), or feature-gate it until implemented.
  - **Status**: Removed (storage no longer exposes log-index surface).

- [x] Withdrawals/logs/log-index types in the storage model (`StoredWithdrawal*`, `StoredLogs`, `BlockBundle.withdrawals/logs`) — **Stub / scope decision**
  - **Claude**: Notes `#[allow(dead_code)]` on many storage types is overly broad; does not explicitly call out withdrawals/logs types.
  - **Codex**: Calls withdrawals/logs persistence/indexes “stub entrypoints” (not implemented).
  - **Gemini**: Calls these stubbed implementations; “NOT dead code” if future indexing is planned.
  - **GPT-5.2**: Calls these “misleading / maintenance drag” vs PRD (since logs are derived from receipts and withdrawals are always null in RPC); suggests delete or feature-gate.
  - **Consensus**: **Mixed** (intentional future surface vs current-product minimalism).
  - **Suggested action**: Make an explicit decision:
    - keep + feature-gate + add at least one real consumer, or
    - remove to match the current v0.2 product contract.
  - **Status**: Removed `StoredWithdrawal*`/`StoredLogs`/`StoredTransaction*` + `BlockBundle.{withdrawals,logs,transactions}` and the associated stub read APIs (logs stay derived from receipts; withdrawals remain `null` in RPC).

- [x] `#[allow(dead_code)]` hygiene in `storage/mod.rs` — **Refactor (cleanup)**
  - **Claude**: Many types are actually used → annotations look overly cautious.
  - **Codex**: “Suspicious / possibly superfluous `#[allow(dead_code)]`”.
  - **Gemini**: Notes `#[allow(dead_code)]` indicates “reserved for future features”.
  - **GPT-5.2**: Recommends targeted allows + possibly `#[deny(dead_code)]` in CI.
  - **Consensus**: **Moderate** (multiple reports agree it’s worth tightening). UPD: I've already removed all of them, then reapplied only to those pointed by warnings.
  - **Status**: Already addressed (per note above).

---

## `node/src/storage/sharded/wal.rs`

- [x] `read_records()` — **Confirmed-dead (prod)** / **Test-only utility**
  - **Claude**: Not mentioned.
  - **Codex**: Likely dead/debug-only (current path prefers index-based reads).
  - **Gemini**: Full WAL reader not wired to lifecycle.
  - **GPT-5.2**: Only used by tests; not used in production.
  - **Consensus**: **Strong (3/4)** → move behind `#[cfg(test)]` or into a clearly named debug module/feature.
  - **Status**: Gated with `#[cfg(test)]`.

- [x] `WalIndexEntry.record_offset` + `WalIndexEntry.payload_len` — **Confirmed-dead (unused fields)**
  - **Claude**: Not mentioned.
  - **Codex**: Fields are unused.
  - **Gemini**: Fields unused; only `block_number` used.
  - **GPT-5.2**: Fields never read anywhere.
  - **Consensus**: **Strong (3/4)** → remove fields or start using them.
  - **Status**: Removed fields from `node/src/storage/sharded/wal.rs` (kept `block_number` only).

- [x] Consolidate WAL parsing/verification paths (index-based vs full-load vs CRC tooling) — **Refactor (simplification)**
  - **Claude**: Not mentioned.
  - **Codex**: Notes multiple WAL parsing strategies co-exist; suggests keeping one “production path” and moving the rest under debug/feature flags.
  - **Gemini**: Similarly notes CRC/record readers are not wired into lifecycle.
  - **GPT-5.2**: Not mentioned as a meta-item (but lists individual unused pieces).
  - **Consensus**: **Moderate (2/4)** → optional, but reduces long-term maintenance overhead.
  - **Status**: Simplified: production uses `build_index()` + `build_slice_index()`; full record reader is test-only (`#[cfg(test)]`), and unused CRC tooling was removed.

---

## `node/src/storage/sharded/mod.rs`

- [x] `SegmentWriter.compression` field — **Confirmed-dead (unused field)**
  - **Claude**: Reserved but unused in current impl.
  - **Codex**: Not mentioned explicitly as a field.
  - **Gemini**: Not mentioned explicitly as a field.
  - **GPT-5.2**: Not mentioned explicitly as a field.
  - **Consensus**: **Single-reporter**, but verified: field is stored and not read.
  - **Status**: Removed field from `node/src/storage/sharded/mod.rs`.

- [x] `SegmentWriter::append_rows_no_commit()` + `SegmentWriter::commit()` — **Confirmed-dead**
  - **Claude**: Reserved for future optimization.
  - **Codex**: Not wired up; appears “verification / future optimization”.
  - **Gemini**: Not used; compaction commits immediately.
  - **GPT-5.2**: Reserved; no call sites.
  - **Consensus**: **Strong (4/4)** → delete or feature-gate until needed.
  - **Status**: Removed methods; `SegmentWriter` now commits on every append.

- [x] `DirtyShardInfo.sorted` + `DirtyShardInfo.complete` — **Confirmed-dead (unused fields)**
  - **Claude**: Unused fields.
  - **Codex**: Still suppressed but no longer referenced.
  - **Gemini**: Unused in `dirty_shards()` output.
  - **GPT-5.2**: Not mentioned explicitly, but implies similar “unused fields”.
  - **Consensus**: **Strong (3/4)** → remove fields or start consuming them.
  - **Status**: Removed fields; `DirtyShardInfo` now only carries `shard_start` and `wal_bytes`.

- [x] `Storage::set_last_indexed_block()` — **Test-only in prod module**
  - **Claude**: Only used in tests.
  - **Codex**: Appears unused.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: No call sites.
  - **Consensus**: **Moderate (3/4)**. Current tree: used by RPC tests only.
  - **Suggested action**: Move behind `#[cfg(test)]` (or keep as an explicit debug API if desired).
  - **Status**: Gated with `#[cfg(test)]` in `node/src/storage/sharded/mod.rs`.

- [x] `Storage::compact_all_dirty()` — **False positive (NOT dead)**
  - **Claude**: Not mentioned.
  - **Codex**: Claims unused.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Reality check**: Current tree calls this from `sync/historical/db_writer.rs` finalize path.

- [x] `records_to_map()` / `WalPayloadCrcReader` / `read_wal_bundle_record_at()` — **Confirmed-dead**
  - **Claude**: Not mentioned.
  - **Codex**: Flags as unused compaction/debug tooling.
  - **Gemini**: Flags as unused WAL verification tooling.
  - **GPT-5.2**: Flags as unused helper(s).
  - **Consensus**: **Strong (3/4)** → delete or move behind a debug feature.
  - **Status**: Removed from `node/src/storage/sharded/mod.rs`.

- [x] Stubbed read APIs: `block_transactions()`, `block_withdrawals()`, `block_logs()`, `block_logs_range()`, `log_index_by_address_range()`, `log_index_by_topic0_range()` — **Stub / scope decision**
  - **Claude**: Not mentioned directly (only notes `LogIndexEntry` appears unused).
  - **Codex**: Calls these stub entrypoints (returns `Ok(None)`/empty).
  - **Gemini**: Calls them “unused/placeholder” but acknowledges they may be future-facing.
  - **GPT-5.2**: Lists them as “not wired” / no call sites; suggests deleting or feature-gating if out of scope.
  - **Consensus**: **Moderate** (agreement on “no current consumer”; disagreement on whether to keep for future).
  - **Status**: Removed from `node/src/storage/sharded/mod.rs` (Option A).

- [x] Reduce repeated “bitset check + load readers + read row” boilerplate — **Refactor (duplication)**
  - **Claude**: Not mentioned.
  - **Codex**: Suggests a helper to consolidate repeated read patterns across storage accessors.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter** (optional, but could reduce bug surface).
  - **Status**: Added `Storage::with_readers_for_present_block()` and refactored `block_header`/`block_tx_hashes`/`block_size`/`block_receipts` to use it.

---

## `node/src/sync/historical/process.rs`

- [x] Duplicate tx-type match-arm structure in `tx_hash_fast()` and `signing_hash_fast()` — **Refactor (duplication)**
  - **Claude**: Notes identical `match TransactionSigned::{Legacy,Eip2930,Eip1559,Eip4844,Eip7702}` arm structure in both; suggests helper/macro if more ops are added.
  - **Codex**: Not mentioned.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter** (low urgency; becomes higher value if more tx-type operations are added).
  - **Status**: `signing_hash_fast()` was removed along with transaction-metadata storage; only `tx_hash_fast()` remains.

---

## `node/src/sync/historical/db_writer.rs`

- [x] `DbWriterMessage::Flush` — **Confirmed-dead (unused variant)**
  - **Claude**: Never sent; could be removed.
  - **Codex**: Variant never sent.
  - **Gemini**: Variant never sent.
  - **GPT-5.2**: Variant never sent; notes config has `db_write_flush_interval_ms`.
  - **Consensus**: **Strong (4/4)** → delete variant or wire it up (periodic flush).
  - **Status**: Removed from `node/src/sync/historical/db_writer.rs`.

- [x] Repeated `bytes_total` calculation from `DbWriteByteTotals` — **Refactor (duplication)**
  - **Claude**: Same calc repeated several times; recommends `DbWriteByteTotals::total()`.
  - **Codex**: Not mentioned.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter** (small, low-risk cleanup).
  - **Status**: Added `DbWriteByteTotals::total()` in `node/src/sync/historical/stats.rs` and used it in `db_writer.rs`.

- [x] Duplicate test helpers (`temp_dir()`, `base_config()`) across `db_writer.rs` and `rpc/mod.rs` — **Refactor (duplication)**
  - **Claude**: Calls out duplication; suggests shared `#[cfg(test)]` test util module.
  - **Codex**: Not mentioned.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter** (low-risk, improves test maintenance).
  - **Status**: Added `node/src/test_utils.rs` (test-only) and updated `rpc/mod.rs` + `db_writer.rs` tests to use it.

---

## `node/src/sync/historical/stats.rs`

- [x] `IngestBenchStats::logs_total()` — **Confirmed-dead**
  - **Claude**: Unused method.
  - **Codex**: Unused (all data read via `summary()`).
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Moderate (2/4)** → delete or start using.
  - **Status**: Removed from `node/src/sync/historical/stats.rs`.

- [x] `DbWriteByteTotals::add()` — **Confirmed-dead**
  - **Claude**: Unused.
  - **Codex**: Unused.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Moderate (2/4)** → delete or start using.
  - **Status**: Removed from `node/src/sync/historical/stats.rs`.

- [x] Duplicate “rate/sec” helper vs `metrics.rs` — **Refactor (duplication)**
  - **Claude**: Notes `metrics.rs::rate_per_sec` unused; suggests removal or use.
  - **Codex**: Not mentioned.
  - **Gemini**: Explicitly flags duplication (`metrics.rs` vs `stats.rs`).
  - **GPT-5.2**: Explicitly flags duplication.
  - **Consensus**: **Moderate (3/4)** → keep one canonical helper.
  - **Status**: Canonical helpers now live in `node/src/metrics.rs`; `stats.rs` calls into them.

- [x] Centralize percentile/math helpers used by stats (`percentile*`, etc.) — **Refactor (duplication/organization)**
  - **Claude**: Not mentioned.
  - **Codex**: Not mentioned.
  - **Gemini**: Flags ad-hoc percentile helper implementations in `stats.rs`; suggests moving math helpers into `metrics.rs`.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter** (optional; improves code locality/reuse if metrics helpers are kept).
  - **Status**: Moved `rate_per_sec` + `percentile*` helpers into `node/src/metrics.rs`; `stats.rs` imports them.

---

## `node/src/sync/historical/scheduler.rs`

- [x] `SchedulerConfig.max_lookahead_blocks` — **Confirmed-dead (unused knob)**
  - **Claude**: Not mentioned.
  - **Codex**: Calls out as unused after refactors.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter**, but verified: stored/configured, not used in logic.
  - **Status**: Removed (incl. CLI `fast_sync_max_lookahead_blocks`).

- [x] `PeerHealthConfig.partial_threshold_multiplier` + `PeerHealthConfig.partial_ban_duration` — **Confirmed-dead (unused knobs)**
  - **Claude**: Not mentioned.
  - **Codex**: Calls out as unused.
  - **Gemini**: Calls out as unused.
  - **GPT-5.2**: Calls out as unused.
  - **Consensus**: **Strong (3/4)** → remove or implement partial-based banning.
  - **Status**: Removed.

- [x] `PeerHealth::ban_remaining()` — **Confirmed-dead**
  - **Claude**: Not mentioned.
  - **Codex**: Calls out as unused.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Mentions an unused ban-remaining helper.
  - **Consensus**: **Moderate (2/4)** → remove.
  - **Status**: Removed.

- [x] `PeerWorkScheduler.low_watermark` — **Confirmed-dead (unused field)**
  - **Claude**: Not mentioned.
  - **Codex**: Calls out as unused after refactors.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Calls out as stored, never read.
  - **Consensus**: **Moderate (2/4)** → remove field or make it functional.
  - **Status**: Removed.

- [x] `PeerWorkScheduler::{completed_count, failed_count}` — **False positive (NOT dead)**
  - **Claude**: Not mentioned.
  - **Codex**: Claims unused.
  - **Gemini**: Claims unused.
  - **GPT-5.2**: Claims unused.
  - **Reality check**: Current tree uses these in `sync/historical/mod.rs` (progress) and in tests.

- [x] “Peer health dump age fields are never consumed” — **False positive**
  - **Claude**: Not mentioned.
  - **Codex**: Not mentioned.
  - **Gemini**: Not mentioned.
  - **GPT-5.2**: Claims `last_*_age_ms` fields are never consumed.
  - **Reality check**: Current tree uses them in `main.rs` peer health summaries and `follow.rs` logs.

---

## `node/src/rpc/mod.rs`

- [x] `RpcWithdrawal` (withdrawals always `None` in RPC responses) — **Stub / scope decision**
  - **Claude**: Not mentioned.
  - **Codex**: Not mentioned.
  - **Gemini**: Notes `RpcWithdrawal` exists but `eth_getBlockByNumber` hardcodes `withdrawals: None`.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Single-reporter**. This is “dead-ish” only if withdrawals are permanently out of scope.
  - **Status**: Removed `RpcWithdrawal`; `RpcBlock.withdrawals` remains present but always `null` (documented in code).

- [x] Duplicate test helpers (`temp_dir()`, `base_config()`) also appear here — **Refactor (duplication)**
  - **Claude**: Flags duplication with `db_writer.rs`.
  - **Others**: Not mentioned.
  - **Status**: Handled via shared `node/src/test_utils.rs` (test-only).

---

## `node/tests/test_strategy.rs`

- [x] `#[ignore]` placeholders (`rpc_contract_probe`, `reorg_rollback_integration`) — **Stub / scope decision**
  - **Claude**: Notes they are placeholders.
  - **Codex**: Not mentioned.
  - **Gemini**: Notes they are empty + ignored.
  - **GPT-5.2**: Not mentioned.
  - **Consensus**: **Moderate (2/4)** → either implement or delete to reduce noise.
  - **Status**: Decision: **keep for now**; added ROADMAP item to implement/un-ignore later.

---

## Report discrepancies / already-removed items (keep as resolved for traceability)

- [x] `sink.rs` deleted but referenced by `sync/historical/AGENTS.md` (Claude) — **Already resolved**
  - Current tree: no `sink.rs`, and no matches for `run_probe_sink`/`fetch_probe_batch`/`ProbeRecord` in `node/`.

- [x] `AGENTS.md` references missing symbols like `BenchmarkMode`, `BlockPayloadSource`, `MultiPeerBlockPayloadSource`, `request_receipt_counts` (Claude) — **Already resolved**
  - Current tree: `node/src/*/AGENTS.md` files list current modules/types; no such references found.

- [x] “Probe mode leftovers” (`ProcessingMode`, `FetchedBlock`, `process_probe`, `ProbeStats::*`, `ProbeRecord.peer_id`) (Codex/Gemini/GPT-5.2) — **Already removed**
  - Current tree: these symbols do not exist under `node/`.

- [x] Repo junk files/dirs (`__MACOSX/`, `node/src/.DS_Store`, `node/src/chain/`) (Gemini/GPT-5.2) — **Not present**
  - Current tree: none found.

