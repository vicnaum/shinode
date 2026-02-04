## Repo hygiene (non-code artifacts)

* `__MACOSX/` (entire dir) — macOS zip junk.
* `node/src/.DS_Store` — macOS Finder artifact.
* `node/src/chain/` — empty dir (looks like a removed/abandoned module placeholder).

---

## Obvious dead code (no non-test call sites; effectively unused)

These either have `#[allow(dead_code)]` *and* no references, or are clearly “reserved/future” stubs.

### `node/src/metrics.rs`

* `lag_to_head(...)` (line ~6) — never referenced.
* `rate_per_sec(...)` (line ~14) — never referenced; also duplicated by `sync/historical/stats.rs::rate_per_sec` (see “repeating” section).

### `node/src/main.rs`

* `total_blocks_to_head(...)` (line ~2269) — never referenced.

### `node/src/sync/historical/process.rs`

* `process_probe(...)` (line ~21) — never referenced (probe path builds `ProbeRecord` directly in `fetch.rs`).

### `node/src/sync/historical/types.rs`

* `ProcessingMode` (line ~11) — never referenced.
* `FetchedBlock` (line ~63) — never referenced (was likely for an earlier “fetch→process” probe pipeline).
* `ProbeRecord.peer_id` field (line ~79) — written in `fetch_probe_batch`, but never read (sink ignores it).

### `node/src/sync/historical/db_writer.rs`

* `DbWriterMessage::Flush` (line ~16) — never sent/handled in the live path (looks like a planned periodic flush feature; config has `db_write_flush_interval_ms`).

### `node/src/sync/historical/scheduler.rs`

* Config knobs stored but unused:

  * `partial_threshold_multiplier` (line ~46)
  * `partial_ban_duration` (line ~48)
* `PeerHealthDump::ban_remaining()` (line ~115) — unused (callers only use `ban_remaining_ms`).
* `PeerHealthDump` “age” fields are computed/stored but never consumed:

  * `last_assigned_age_ms` / `last_success_age_ms` / `last_failure_age_ms` / `last_partial_age_ms` (lines ~147–158)
* `PeerWorkScheduler.low_watermark` field (line ~456) — stored, never read.

### `node/src/storage/sharded/wal.rs`

* `read_records(...)` (line ~56) — only used by tests; not used in production.
* `WalIndexEntry.record_offset` and `payload_len` fields (lines ~20–24) — never read anywhere (no `.record_offset` usages).

### `node/src/p2p/mod.rs`

Test-only scheduling abstractions sitting in non-test code:

* `trait PeerSelector`, `struct RoundRobinPeerSelector`, `struct SelectorPeerId` (lines ~1110–1140)

  * Only referenced by the test module at the end of the file.

### `node/src/storage/sharded/mod.rs`

A cluster of “not wired” storage APIs and debug helpers:

* `SegmentWriter::append_rows_no_commit(...)` (line ~218) — “reserved for future use”; no call sites.
* `SegmentWriter::commit(...)` (line ~224) — reserved; no call sites.
* `Storage::set_last_indexed_block(...)` (line ~462) — no call sites.
* Transaction / withdrawal / logs / log-index readers (all no call sites):

  * `block_transactions(...)` (~1168)
  * `block_withdrawals(...)` (~1188)
  * `block_logs(...)` (~1234)
  * `block_logs_range(...)` (~1278)
  * `log_index_by_address_range(...)` (~1286)
  * `log_index_by_topic0_range(...)` (~1295)
* Unused helper(s):

  * `records_to_map(...)` (~1711)
  * `WalPayloadCrcReader` + its impls (~1734–1766)
  * `read_wal_bundle_record_at(...)` (~1780)

---

## Likely-unused / misleading (present, but not used by the *current* v0.1 feature contract)

These aren’t necessarily unreachable, but they don’t contribute to the PRD/README “must work” surface and create maintenance drag.

### Storage types that imply features you don’t actually serve

In `node/src/storage/mod.rs`:

* `StoredWithdrawal`, `StoredWithdrawals` + `BlockBundle.withdrawals`
* `StoredLogs` + `BlockBundle.logs`
* `LogIndexEntry` (+ the related “log index by address/topic0” APIs in sharded storage)

Why this matters:

* PRD/README scope is `eth_getLogs` + minimal block/tx-hash support; your `getLogs` path walks receipts and synthesizes log metadata. There’s no read-path using `StoredWithdrawals`/`StoredLogs`, and the sharded backend doesn’t encode a standalone logs segment or log index.
* Net effect: you carry types/fields that are almost always `None`/empty, plus byte accounting that suggests you’re writing data you aren’t.

Action:

* Either **delete** these types/fields, or **feature-gate** them (`features = ["withdrawals", "log_index"]`) so the default build matches the PRD.

### Peer health dump has unused fields

`PeerHealthDump` contains multiple “age” fields and a `ban_remaining()` method that are never used by any consumer (SIGUSR1 output uses only `ban_remaining_ms`, plus score/sample/batch/inflight).

Action:

* Remove the unused fields/method, or start consuming them (e.g., emit them on SIGUSR1 / metrics).

---

## Repeating / legacy-looking code

### Duplicate “rate per second” helpers

* `node/src/metrics.rs::rate_per_sec`
* `node/src/sync/historical/stats.rs::rate_per_sec`

They don’t share code and one appears unused. Pick one location (I’d keep it near the stats/telemetry that actually uses it) and delete the other.

### Duplicate throughput “sliding window” logic

* `main.rs` has a `VecDeque<(Instant, u64)>` window for speed/ETA.
* `sync/historical/mod.rs` has a similar window (and an escalation window).

Action:

* Factor into a small reusable helper (ironically, `metrics.rs` is the natural place), so speed/ETA behavior is consistent across modes.

### Heavy use of `#[allow(dead_code)]` as a blanket

You’ve got `#[allow(dead_code)]` on many structs/types in `storage/mod.rs` and elsewhere. Some are genuinely dead (above), others are “future contract” placeholders.

Action:

* Replace blanket allows with **targeted** ones:

  * move test-only helpers behind `#[cfg(test)]`
  * use `#[allow(dead_code)]` on *specific* functions/fields that are intentionally staged
  * consider `#[deny(dead_code)]` in CI to stop new drift (you can carve out intentional modules with `#[allow]`).

---

## Suggested cleanup plan (lowest-risk first)

1. Delete repo junk: `__MACOSX/`, `node/src/.DS_Store`, empty `node/src/chain/`.
2. Move test-only selectors in `p2p/mod.rs` behind `#[cfg(test)]`.
3. Delete truly unused probe pipeline remnants: `ProcessingMode`, `FetchedBlock`, `process_probe`.
4. Remove unused scheduler knobs + unused peer dump fields/methods (or wire them properly).
5. Decide whether withdrawals/log-index/logs-segment are **in-scope**:

   * if *not*: delete `StoredWithdrawal(s)`, `StoredLogs`, `LogIndexEntry`, the dead storage readers, and remove `BlockBundle.withdrawals/logs` + corresponding byte accounting.
   * if *yes (later)*: feature-gate and add at least one real consumer so it’s not “zombie API”.

If you want, I can also produce a patch-style set of deletions/edits (file-by-file) to remove the dead items while keeping the current feature contract intact.
