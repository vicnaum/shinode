# PRD: Stateless History Node (v0.1 MVP)

## Status update (v0.2)
The v0.1 product contract is implemented. v0.2 focuses on reliability/performance hardening:
- Benchmark warmup gating: `--benchmark-min-peers` (default: 5).
- Peer pool warmup: head probes performed asynchronously (avoid blocking peer session watcher).
- Fast-sync WAL batching (out-of-order) to reduce per-block WAL syscall overhead.
- Benchmark events include compaction + sealing duration instrumentation for attribution.
- Compaction memory hardening: stream WAL during compaction + serialize compactions to avoid OOM-scale peaks.
- Optional allocator knobs for benchmarking: `MALLOC_ARENA_MAX=2` (Linux/glibc) or `--features jemalloc` build.
- Pipeline parity and resume: default ingest uses the benchmark ingest pipeline; resume skips already-present blocks and recompacts dirty shards before follow.

## Context
We already have a working **receipt availability harness** (`harness/`) that uses Reth as a library to do devp2p discovery, dialing, `eth` handshake, and `GetBlockHeaders`/`GetReceipts*` probing. The next product is a **long-running stateless history node**: it ingests history artifacts and serves an indexer-compatible RPC subset.

References:
- Project goals: `spec/main_idea.md`
- Verified Reth knowledge: `spec/reth_kb/INDEX.md` (especially Q015/Q016/Q021/Q022/Q034/Q039)
- Reth crate responsibility map: `spec/reth_kb/agents-export/reth/crates/**/AGENTS.md` (committed snapshot)

## Product goal
Ship **v0.1** as soon as possible:
- **Backfill** from `start_block` to head.
- **Follow** head continuously.
- **Persist** retained history (restart-safe).
- **Serve RPC** sufficient for Uniswap-style event indexing without paid RPC.

## Non-goals (v0.1)
- EVM execution / state / archive state.
- Traces (`debug_*/trace_*`) and stateful methods (`eth_call`, balances, storage).
- “Full node” canonicality guarantees: v0.1 uses a pragmatic trust model with reorg rollback.

## MVP user stories
- As an indexer operator, I can point Ponder/rindexer to `http://127.0.0.1:<port>` and index Uniswap contracts end-to-end.
- As an operator, I can restart the node and it resumes without duplicating logs or corrupting the DB.
- As an operator, I can tune resource limits and restart safely.

## v0.1 scope (contract)

### Networks
- Ethereum mainnet only.

### Data retained (minimum)
To serve `eth_getLogs` correctly, we must retain enough to produce:
- block metadata: number/hash/parent/timestamp
- tx hashes per block (to attach `transactionHash`/`transactionIndex`)
- logs with metadata (`logIndex`, `removed` on reorg)

### Retention modes (explicit from day 1)
- **v0.1 default**:
  - store headers + tx hashes + full receipts/logs for retained ranges
- **Deferred to later versions**:
  - filtered logs retention
  - minimal tx metadata retention (`from`, `to`, `value`)
  - calldata retention (`input`)

### RPC surface (v0.1)
Required (v0.1 target: rindexer event indexing):
- `eth_chainId`
- `eth_blockNumber`
- `eth_getBlockByNumber` (must support `"latest"`; must include `number`, `timestamp`, `hash`, `logsBloom`)
- `eth_getLogs` (must include standard log fields: `blockHash`, `blockNumber`, `transactionHash`, `transactionIndex`, `logIndex`; `blockTimestamp` is optional)

#### v0.1 RPC contract (rindexer-minimum)
This section is the concrete “contract” we implement for v0.1. It is intentionally minimal and chosen to satisfy rindexer’s default event indexing mode (polling).

**1) `eth_chainId`**
- **Params**: `[]`
- **Result**: hex quantity chain id (e.g. `"0x1"`)

**2) `eth_blockNumber`**
- **Params**: `[]`
- **Result**: hex quantity block number.
- **Semantics**: returns the **highest block number present in storage** (`max_present_block`).
  This may be non-contiguous (holes are possible); clients must handle gaps explicitly (see `eth_getLogs` behavior below).

**3) `eth_getBlockByNumber`**
- **Params**: `[block: "latest" | hex_quantity, full_transactions: boolean]`
- **Supported in v0.1**:
  - `block`: `"latest"` and explicit hex block numbers
  - `full_transactions`: **false only** (if `true`, return a clear “not supported” error)
- **Result (minimum fields guaranteed)**:
  - `number`, `hash`, `parentHash`, `timestamp`, `logsBloom`
  - `transactions`: array of **tx hashes** (since `full_transactions=false`)

**4) `eth_getLogs`**
- **Params**: `[filter: { fromBlock?, toBlock?, blockHash?, address?, topics? }]`
- **Supported in v0.1**:
  - range queries via `fromBlock` + `toBlock` (hex quantities or `"latest"`)
  - `blockHash`: **not supported** in v0.1 (range queries only)
  - `address`: single value or array
  - `topics`: **topic0 only** (event signature). Supports `null` wildcards and OR-lists for topic0 (e.g. `["0x…"]` or `[["0x…","0x…"]]`).
- **Result (minimum fields guaranteed for each log)**:
  - `address`, `topics`, `data`
  - `blockNumber`, `blockHash`
  - `transactionHash`, `transactionIndex`
  - `logIndex`
  - `removed`: always `false` in v0.1 (`eth_getLogs` returns canonical logs only)
- **Ordering**:
  - logs are sorted ascending by `(blockNumber, transactionIndex, logIndex)` (rindexer assumes `logs.last()` corresponds to the highest block)
- **Limits**:
  - enforce a configurable `max_blocks_per_filter` and `max_logs_per_response`
  - **On limit violation**: return JSON-RPC **invalid params** (`-32602`) with:
    - block range too wide: `block range exceeds max_blocks_per_filter`
    - too many logs: `response exceeds max_logs_per_response`
  - **rindexer retry behavior**: rindexer will shrink the requested range on *any* `eth_getLogs` error.
- **Availability semantics (all-or-nothing)**:
  - If **any** block in `[fromBlock..=toBlock]` is missing in storage, return a JSON-RPC **server error**:
    - code: `-32001`
    - message: `missing block in requested range`
    - data: `{ "missing_block": "0x..." }` (first missing block)

Optional / deferred (v0.2+):
- **Ponder baseline compatibility adds**:
  - `eth_getBlockByHash` (reorg traversal)
  - `eth_call` (multicall3 / readContract; requires state and is not supported by a fully stateless node unless proxied or supported via a stateless execution approach like RESS)
  - optional WS: `eth_subscribe` (`newHeads`) with polling fallback
  - optional receipts: `eth_getBlockReceipts` / `eth_getTransactionReceipt`
  - optional call traces: `debug_traceBlockByNumber` / `debug_traceBlockByHash`
- Other deferred items:
  - `net_version`, `web3_clientVersion`
  - `eth_getTransactionByHash` (if we decide to support it)
  - non-standard chain detection methods (e.g., `zks_L1ChainId`)

### Reorg semantics (v0.1)
- Maintain a configurable **rollback window** (e.g., 64 blocks default).
- When a reorg is detected:
  - roll back canonical blocks/logs above the common ancestor
  - v0.1 default: delete-on-rollback (no tombstones); `eth_getLogs` continues to return canonical logs only (`removed=false`)

### Head tracking / trust model (v0.1)
Reth uses an external CL for canonical head/finalization (Q039). For v0.1, we start pragmatic:
- Track head from the P2P view (peer status / header announcements) and treat it as “unsafe head”.
- Design the head source behind an interface so we can add a CL/beacon mode later (v0.2+).

## Architecture (v0.1)

### High-level components
1. **P2P subsystem**
   - Reuse: `reth-network`, `reth-network-api`, `reth-eth-wire`, `reth-eth-wire-types`, `reth-network-types`
   - Responsibilities:
     - discovery + dialing
     - session lifecycle + peer health
     - request/response plumbing for headers/bodies/receipts

2. **Sync / ingest orchestrator**
   - Responsibilities:
     - decide “what to fetch next” (backfill, catch-up, follow)
     - manage concurrency / retries / backpressure
   - Informed by: reth engine download coordination patterns (Q034) and peer management (Q015/Q016).

3. **Chain tracker**
   - Responsibilities:
     - canonical header chain representation
     - reorg detection + common ancestor computation
     - checkpoint: “last fully indexed block”

4. **Storage**
   - Baseline: sharded static-file storage (schema v2) built on per-shard NippyJar segments, with
     per-shard presence bitsets, staging WAL, and compaction into canonical sorted shards.
   - Must support:
     - idempotent writes
     - range queries for logs
     - rollback on reorg

5. **RPC server**
   - Minimal JSON-RPC server with safe defaults (localhost bind, request limits).
   - Query limits should mirror reth’s intent (Q022/Q023): max blocks per filter, max logs per response.

### Suggested crate/module layout (in this repo)
- `harness/` (keep as a separate tool)
- `node/`: the v0.1 service binary + internal modules:
  - `p2p/`
  - `sync/` (includes chain tracking + reorg handling today)
  - `storage/`
  - `rpc/`
  - `cli/`

This mirrors reth’s internal boundaries and keeps later refactors small.

## Storage design (v0.1)

### Storage requirements
- Fast `eth_getLogs` over large ranges (indexer workloads).
- Deterministic rollback for reorgs.
- Easy “what do we have?” introspection (progress, head, retention settings).
- Versioning/migrations.

### Proposed logical schema (implementation-agnostic)
Minimum tables/collections:
- **`blocks`**
  - key: `block_number`
  - fields: `hash`, `parent_hash`, `timestamp`, `logs_bloom`, `canonical` (or implied by canonical set)
- **`block_txs`**
  - key: `(block_number, tx_index)`
  - fields: `tx_hash`
- **`block_receipts`**
  - key: `(block_number, tx_index)`
  - fields: receipt envelope needed to build `eth_getTransactionReceipt`
- **`logs`**
  - key: `(block_number, log_index)` (and/or `(tx_hash, log_index)`)
  - fields: `address`, `topics[0..4]`, `data`, `tx_hash`, `tx_index`, `block_hash`, `removed`
  - indexes: `(block_number)`, `(address, block_number)`, `(topic0, block_number)`, composite indexes depending on MVP needs
- **`meta`**
  - `chain_id`, `start_block`, `retention_mode`, `last_indexed_block`, `head_seen`, `rollback_window`, schema version

### Reorg handling in storage
Two viable approaches:
1. **Delete-on-rollback (simple)**:
   - when reorg happens, delete blocks/logs/receipts above common ancestor
   - RPC never returns `removed=true` for historical queries (only for streaming/subscriptions, which are optional in v0.1)
2. **Tombstone-on-rollback (more correct)**:
   - mark logs from rolled-back blocks as `removed=true` and keep them for a bounded time window
   - allows an indexer to reconcile removals even if queried slightly later

Recommendation:
- v0.1: **delete-on-rollback** with a clear rollback window contract.
- v0.2: add tombstones if needed (or if we add WS subscriptions).

## Sync / ingest algorithm (v0.1)

### Backfill
- Determine a moving “head” block number (best-effort) and sync `start_block..=head`.
- For each block:
  1. fetch header
  2. fetch body (or at least tx hashes)
  3. fetch receipts
  4. derive logs with tx hashes + indices
  5. persist atomically (idempotent), storing tx signature + signing hash for deferred sender recovery

### Historical fast sync (v0.1.5)
- For blocks older than the reorg window, use a fast backfill mode:
  - split the historical range into fixed-size chunks (default 16)
  - fetch chunks concurrently across peers with bounded in-flight requests
  - buffer out-of-order chunks and write in block order
  - persist each chunk in a single static-file append batch
- For blocks inside the reorg window, use the existing safe sequential path.

### Follow (live)
- Listen for peer sessions and head updates (peer status head, announcements).
- Maintain a “target head” and keep indexing forward.
- On parent mismatch with our canonical head:
  - treat as reorg, find common ancestor, rollback, then re-sync forward.

### Concurrency & peer selection
- Reuse reth’s peer/session management and reputation/backoff behavior (Q015/Q016).
- Keep concurrency bounded (global and per-peer).
- Prefer “pipeline overlap” as a later optimization (Q022/Q034 style separation of fast path vs disk path).

## Testing plan (v0.1)

### Approach (parallel + contract-first)
- Write tests **in parallel** with implementation. Treat the PRD “v0.1 RPC contract” as **executable acceptance criteria**.
- Use **TDD where the spec is crisp** (RPC response shape/errors, log ordering, storage idempotency, rollback semantics).
- For evolving areas (P2P heuristics, peer selection/orchestration), prefer **invariants + regression tests**: codify “must never happen” and add a test whenever we fix a bug/edge case.
- Keep fixtures **small and deterministic** so `cargo test` is fast and reliable.

### Unit tests
- receipt decoding + eth version variants (`eth/68`, `eth/69`, `eth/70`)
- receipt→log derivation correctness (tx index/log index; `removed` is always `false` in v0.1)
- reorg common ancestor + rollback math
- storage idempotency: “write same block twice” yields same DB state

### Integration tests (local)
- spin up the service, ingest a short range, then query via RPC and compare against known fixtures
- simulate a reorg by injecting an alternate header chain and assert rollback behavior

### RPC conformance checks (contract tests)
- Implement a small “indexer probe” integration test that validates:
  - `eth_chainId` returns the configured chain id
  - `eth_blockNumber` returns “last fully indexed block”
  - `eth_getBlockByNumber("latest", false)` returns minimum fields: `number`, `hash`, `parentHash`, `timestamp`, `logsBloom`, `transactions` (hashes)
  - `eth_getBlockByNumber(_, true)` returns a clear “not supported” error
  - `eth_getLogs` returns:
    - required fields and **ascending** ordering by `(blockNumber, transactionIndex, logIndex)`
    - topic/address filter semantics used by indexers (address as single or array; topic wildcards)
    - limit errors as `-32602` with deterministic reth-style messages (see RPC contract section)
- Where possible, reuse/refer to reth’s `eth_getLogs` compatibility vectors as test inputs:
  - `reth/crates/rpc/rpc-e2e-tests/testdata/rpc-compat/eth_getLogs/` (if you have a local `reth/` checkout)
- Acceptance test (manual or automated later): run `rindexer` against the node for a small range and confirm end-to-end indexing works.

## Milestone plan (mapped to ROADMAP.md)

### v0.1.0 Product contract + skeleton
Deliverables:
- CLI/config scaffolding for: chain, start block, retention, DB path, RPC bind
- storage schema draft + migrations/versioning approach
- basic process lifecycle (shutdown, logs)
- explicit “unsupported RPC list”

Verification:
- service starts/stops, creates DB, exposes a minimal “identity” RPC (e.g., `eth_chainId`)

Test gate (must pass):
- `cargo test` passes
- RPC smoke: `eth_chainId` responds with configured chain id

### v0.1.1 Sync + ingest loop
Deliverables:
- range sync backfill `start_block..head`
- canonical header chain tracking and basic reorg rollback
- receipts + body ingestion and log derivation

Verification:
- ingest a small range and confirm `eth_getBlockByNumber` and `eth_getLogs` return expected results

Test gate (must pass):
- Unit tests for receipt→log derivation and rollback math
- Integration: ingest a small fixture range and validate `eth_getLogs` ordering + required fields

### v0.1.2 Persistence hardening
Deliverables:
- idempotent writes + checkpoint/resume
- indexes for logs
- storage rollback correctness

Verification:
- restart mid-sync and ensure progress resumes correctly
- induce reorg and ensure DB returns to consistent canonical chain

Test gate (must pass):
- Idempotency: “write same block twice” yields same DB state
- Restart: resume from checkpoint without duplicates
- Reorg: rollback deletes data above common ancestor (v0.1 delete-on-rollback)

### v0.1.3 RPC server
Deliverables:
- required RPC endpoints implemented
- query limits (max blocks per filter, max logs per response)
- localhost-by-default binding and basic request limits

Verification:
- run an indexer against it for a small range

Test gate (must pass):
- RPC contract tests (“indexer probe”) for v0.1 RPC surface:
  - response shapes, log ordering, and `-32602` limit errors/messages
- Compatibility vectors: reth `eth_getLogs` topic/address wildcard fixtures (where applicable)

### v0.1.4 Operator experience
Deliverables:
- structured logs/metrics (lag to head, throughput, reorg count)
- stable config file format (optional)
- ingest benchmark mode with per-stage timing (fetch/process/db)

Verification:
- unattended run for N hours with stable memory and bounded queues

Test gate (must pass):
- (Manual) N-hour soak run: bounded memory/queues and continued forward progress

### v0.1.5 Historical sync speedup
Deliverables:
- historical fast sync mode (chunked + concurrent downloads + ordered writes)
- bounded in-flight concurrency and buffering
- per-chunk atomic static-file append batches + checkpoints
- safe boundary switch to the slow path near the reorg window

Verification:
- ingest a large historical range and report throughput vs baseline

Test gate (must pass):
- chunk planner tests (range → chunks)
- storage batch write tests (single append batch per chunk)

### v0.1.6 Live sync + reorg resilience
Deliverables:
- follow mode (keep up with new heads)
- live reorg handling with rollback window

## Open questions (for you)
These don’t block writing code, but they affect the “default” product contract:
1. If/when Ponder support becomes a goal: decide the `eth_call` strategy (proxy to upstream vs. stateless execution via RESS vs. running execution/state).
