## Dead code / legacy surface — decisions needed

This document collects the remaining items from `deadcode/DEAD_CODE_REPORT.md` that **require an explicit product/engineering decision** (not just mechanical cleanup). For each case: what it is, why it exists, what it buys, and options with pros/cons.

---

## 1) Storage “future surface” (logs / log indexes / withdrawals / transactions)

### What exists today

- **Types (model layer)** in `node/src/storage/mod.rs`:
  - `LogIndexEntry`
  - `StoredWithdrawal`, `StoredWithdrawals`
  - `StoredLogs`
  - `BlockBundle` includes `withdrawals`/`logs` fields (currently not meaningfully used for RPC)
- **Stubbed read APIs (backend layer)** in `node/src/storage/sharded/mod.rs`:
  - `Storage::block_transactions()`
  - `Storage::block_withdrawals()`
  - `Storage::block_logs()`
  - `Storage::block_logs_range()`
  - `Storage::log_index_by_address_range()`
  - `Storage::log_index_by_topic0_range()`

Today’s actual runtime behavior (per current implementation and PRD/SPEC):
- The node stores **headers, tx hashes/meta, receipts, block sizes**.
- RPC `eth_getLogs` is derived from **stored receipts** on-demand.
- RPC `eth_getBlockByNumber` returns `withdrawals: null` (and withdrawals are not stored).

### Why this code exists

This looks like an **anticipated v0.3+ direction** (precomputed logs/indexes, maybe richer block responses), but the v0.2 runtime doesn’t use it. Keeping these stubs can make future work slightly easier, but it also expands the code surface with “zombie APIs”.

### Options

#### Option A — Remove the unused surface now (shrink to v0.2)
- **Do**: Delete the unused types (`LogIndexEntry`, `StoredWithdrawal*`, `StoredLogs`, and the unused `BlockBundle` fields if they’re always empty/None) and remove the stubbed storage APIs.
- **Pros**:
  - Smaller API surface → less confusion and less “dead code drift”.
  - Less maintenance and fewer accidental invariants (e.g. “why is logs empty?”).
  - More honest representation of current product.
- **Cons**:
  - Future work will reintroduce these types/APIs (but in a more deliberate design).

#### Option B — Keep stubs, but explicitly mark them as “future” (no behavior change)
- **Do**: Keep the types + stub methods, but make the intent explicit:
  - Add clear doc-comments: “reserved for future indexed logs”, “not used in v0.2”.
  - Add a small “roadmap note” in the relevant `AGENTS.md` / `SPEC.md` if desired.
- **Pros**:
  - Minimal code churn; preserves potential future direction.
  - Still makes it harder to misinterpret as “implemented”.
- **Cons**:
  - Still carries non-functional surface area.

#### Option C — Feature-gate this surface (default off)
- **Do**: Keep the code, but put it behind crate features like `log_index` / `withdrawals`.
- **Pros**:
  - Default build matches v0.2; optional builds keep scaffolding for future.
  - Makes “in scope vs not” concrete.
- **Cons**:
  - Adds feature complexity (Cargo features, conditional compilation, test matrix).

### Practical guidance (what I’d do by default)

If v0.2/v0.3 is aiming for **minimalism and low maintenance**, Option A is the cleanest.
If you want to preserve direction without committing, Option B is a middle ground.

---

## 2) RPC withdrawals representation (`RpcWithdrawal`)

### What exists today
- `node/src/rpc/mod.rs` defines `RpcWithdrawal` and includes `withdrawals: Option<Vec<RpcWithdrawal>>` in the block response shape.
- But the implementation returns `withdrawals: None` for blocks (consistent with “not stored”).

### Why it exists
- To match the “modern” Ethereum block response schema, even if withdrawals are not served.

### Options
- **Keep**: It keeps response schema aligned, even if always null.
- **Remove**: If you want “strict minimal” response shape (but then you may diverge from common clients/tools expectations).
- **Feature-gate**: Only include withdrawals fields when a `withdrawals` feature is enabled.

---

## 3) DB writer “Flush” message (`DbWriterMessage::Flush`)

### What exists today
- `DbWriterMessage::Flush` is handled in `node/src/sync/historical/db_writer.rs`, but nothing in the pipeline sends it.
- There is a config knob `db_write_flush_interval_ms`, suggesting a design where a timer triggers flushes.

### Why it exists
- Likely intended for **periodic flushing** in fast-sync (and possibly follow), to bound memory and reduce tail latency when blocks arrive slowly or in bursts.

### Options

#### Option A — Remove the variant
- **Pros**: Simplifies the DB writer protocol and reduces dead branches.
- **Cons**: If you want periodic flushing later, you’ll re-add it.

#### Option B — Implement periodic flushing (wire it up)
- **Do**: Add a timer in the orchestration layer to send `Flush` at the configured interval.
- **Pros**:
  - More stable memory usage for low-throughput periods.
  - More predictable “progress” and WAL persistence timing.
- **Cons**:
  - Adds complexity and more scheduler/write wakeups.
  - Needs careful interaction with compaction triggers and follow reorder buffer.

#### Option C — Keep but document as TODO
- **Pros**: Minimal churn.
- **Cons**: Dead branch remains.

---

## 4) Scheduler “unused knobs / fields”

### What exists today (in `node/src/sync/historical/scheduler.rs`)
- `SchedulerConfig.max_lookahead_blocks` is configured but not used.
- `PeerHealthConfig.partial_threshold_multiplier` / `partial_ban_duration` are stored but not used.
- `PeerHealth::ban_remaining()` exists but is unused.
- `PeerWorkScheduler.low_watermark` exists but is unused.

### Why it exists
- Likely remnants of earlier scheduling experiments (lookahead throttling, partial-response based bans, explicit “progress watermark”).

### Options

#### Option A — Remove unused knobs/fields now
- **Pros**: Less cognitive load; fewer “why doesn’t this do anything?” moments.
- **Cons**: If you wanted them later, you re-add.

#### Option B — Implement them properly
- **max_lookahead_blocks**: enforce an upper bound on how far ahead the scheduler can enqueue/assign beyond current progress.
- **partial_* knobs**: define what constitutes a “partial”, then ban/decrease batch sizes accordingly.
- **low_watermark**: use it to avoid re-scheduling below some floor or to drive UI metrics.
- **Pros**: you actually get the advertised behavior.
- **Cons**: needs careful design + tests; behavior changes.

#### Option C — Keep but rename/comment as “reserved”
- **Pros**: Minimal churn, avoids deleting potentially useful future hooks.
- **Cons**: Still dead surface.

---

## 5) Legacy receipts path (`request_receipts_legacy`)

### What exists today
- In `node/src/p2p/mod.rs`, receipts fetching falls back to a legacy path for peers not supporting newer eth versions (eth/69/70).

### Why it exists
- To maximize peer compatibility and not restrict the node to only the latest protocol variants.

### Options
- **Keep fallback** (status quo): more peers available, potentially more robustness.
- **Drop legacy**: simplify and reduce code paths; may reduce peer pool or require tighter peer selection.
- **Instrument/decide later**: add logging/metrics about how often legacy path is taken, then decide.

---

## 6) Ignored integration test placeholders (`node/tests/test_strategy.rs`)

### What exists today
- Two `#[ignore]` tests (`rpc_contract_probe`, `reorg_rollback_integration`) that are empty placeholders.

### Why it exists
- A reminder / scaffolding for future integration coverage.

### Options
- **Keep**: they’re harmless and communicate intent.
- **Remove**: reduces noise in the repo; avoids “tests are ignored” smell.
- **Implement**: best, but requires actual environment + fixtures + longer runtime.

---

## Optional refactors (no decision required, just “do you want it?”)

These are the remaining unchecked items that are purely refactor/cleanup (behavior-preserving), but not strictly “obvious must-do”:
- Consolidate WAL parsing/verification paths into one “production” path.
- Reduce repeated “bitset check + load readers + read row” boilerplate in sharded storage.
- Deduplicate tx-type match arms in `tx_hash_fast()` / `signing_hash_fast()` (likely via helper/macro).
- Centralize percentile/math helpers used by `stats.rs`.
- Deduplicate test helper functions (`temp_dir()`, `base_config()`) between `rpc/mod.rs` and `sync/historical/db_writer.rs`.

