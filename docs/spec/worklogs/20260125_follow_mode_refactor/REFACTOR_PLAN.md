# Refactor Plan: Benchmark Ingest As Default (Temporary)

Branch: `refactor/bench-default` (created from `ac5ad06`)

## Goal
Make the default `stateless-history-node` execution path run the exact same pipeline as
`--benchmark ingest` (fast-sync ingest pipeline), with the only difference being:
- When `--benchmark` is **not** set: do **not** write benchmark JSON artifacts
  (events/logs/trace/report files).
- For now: after ingest finishes, **exit** (do not start follow mode / RPC).

## Non-goals (for this refactor)
- No algorithm changes to ingest/compaction.
- No follow-mode behavior changes (follow mode is temporarily disabled).

## Implementation Steps
1. **Unify ingest runner in `node/src/main.rs`**
   - Execute the same steps as the existing `--benchmark ingest` path:
     - Start P2P.
     - Wait for peers warmup (`--benchmark-min-peers`, default 5).
     - Determine `head_at_startup` via `wait_for_peer_head(...)`.
     - Compute `range = compute_target_range(...)`.
     - Run `sync::historical::run_ingest_pipeline(... head_cap_override=None ...)`.
     - Flush peer cache (+ persist peer limits).
2. **Artifact gating**
   - Only create `BenchmarkRunContext` and file writers when `--benchmark` is
     `probe` or `ingest`.
   - Ignore `--benchmark-events/--benchmark-trace` when `--benchmark` is disabled.
3. **Default mode behavior**
   - Running without `--benchmark` should execute the unified ingest runner and
     exit when done (no follow loop, no RPC).
4. **Testing / Validation**
   - Run the same CLI args with and without `--benchmark ingest` and compare:
     - blocks/sec, peer warmup behavior, and end-to-end time.
     - no JSON artifacts created when `--benchmark` is not set.

## Later (after this refactor is validated)
- Reintroduce follow mode by starting RPC only when follow starts, and ensure the
  backfill stage uses the same ingest pipeline as benchmark mode (benchmark flag
  only toggles instrumentation + “exit vs follow”).

