# PRD: Stateless History Node (v0.1 MVP)

## Context
We already have a working **receipt availability harness** (`harness/`) that uses Reth as a library to do devp2p discovery, dialing, `eth` handshake, and `GetBlockHeaders`/`GetReceipts*` probing. The next product is a **long-running stateless history node**: it ingests history artifacts and serves an indexer-compatible RPC subset.

References:
- Project goals: `spec/main_idea.md`
- Verified Reth knowledge: `spec/reth_kb/INDEX.md` (especially Q015/Q016/Q021/Q022/Q034/Q039)
- Reth crate responsibility map: `reth/crates/**/AGENTS.md`

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
Required:
- `eth_chainId`
- `net_version`
- `web3_clientVersion`
- `eth_blockNumber`
- `eth_getBlockByNumber` (at least header + tx hashes)
- `eth_getBlockByHash` (at least header + tx hashes)
- `eth_getLogs`

Optional (only if we retain enough data):
- `eth_getTransactionReceipt`
- `eth_getTransactionByHash`

Explicitly unsupported (return standard JSON-RPC “method not found” or a clear “not supported” error):
- stateful and trace methods (`eth_call`, balances, storage, `debug_trace*`, etc.)

### Reorg semantics (v0.1)
- Maintain a configurable **rollback window** (e.g., 64 blocks default).
- When a reorg is detected:
  - roll back canonical blocks/logs above the common ancestor
  - make “removed logs” visible to RPC clients (how depends on our chosen internal log model; see storage section)

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
   - Recommended baseline: MDBX (reth-style) with explicit schema/versioning.
   - Must support:
     - idempotent writes
     - range queries for logs
     - rollback on reorg

5. **RPC server**
   - Minimal JSON-RPC server with safe defaults (localhost bind, request limits).
   - Query limits should mirror reth’s intent (Q022/Q023): max blocks per filter, max logs per response.

### Suggested crate/module layout (in this repo)
- `harness/` (keep as a separate tool)
- `node/` (new): the v0.1 service binary + internal modules:
  - `p2p/`
  - `sync/`
  - `chain/`
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
  5. persist atomically (idempotent)

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

### Unit tests
- receipt decoding + eth version variants (`eth/68`, `eth/69`, `eth/70`)
- receipt→log derivation correctness (tx index/log index, removed flag rules)
- reorg common ancestor + rollback math
- storage idempotency: “write same block twice” yields same DB state

### Integration tests (local)
- spin up the service, ingest a short range, then query via RPC and compare against known fixtures
- simulate a reorg by injecting an alternate header chain and assert rollback behavior

### RPC conformance checks
- Use a minimal “indexer probe” script/test that:
  - calls `eth_blockNumber`
  - fetches blocks and logs
  - validates response shape matches expectations for the chosen indexer

## Milestone plan (mapped to ROADMAP.md)

### v0.1.0 Product contract + skeleton
Deliverables:
- CLI/config scaffolding for: chain, start block, retention, DB path, RPC bind
- storage schema draft + migrations/versioning approach
- basic process lifecycle (shutdown, logs)
- explicit “unsupported RPC list”

Verification:
- service starts/stops, creates DB, exposes health/version endpoints (or `web3_clientVersion`)

### v0.1.1 Sync + ingest loop
Deliverables:
- range sync backfill `start_block..head`
- canonical header chain tracking and basic reorg rollback
- receipts + body ingestion and log derivation

Verification:
- ingest a small range and confirm `eth_getBlockByNumber` and `eth_getLogs` return expected results

### v0.1.2 Persistence hardening
Deliverables:
- idempotent writes + checkpoint/resume
- indexes for logs
- storage rollback correctness

Verification:
- restart mid-sync and ensure progress resumes correctly
- induce reorg and ensure DB returns to consistent canonical chain

### v0.1.3 RPC server
Deliverables:
- required RPC endpoints implemented
- query limits (max blocks per filter, max logs per response)
- localhost-by-default binding and basic request limits

Verification:
- run an indexer against it for a small range

### v0.1.4 Operator experience
Deliverables:
- structured logs/metrics (lag to head, throughput, reorg count)
- stable config file format (optional)

Verification:
- unattended run for N hours with stable memory and bounded queues

## Open questions (for you)
These don’t block writing code, but they affect the “default” product contract:
1. Which indexer is the “must pass” target first: Ponder, rindexer, or a specific custom pipeline?
2. Confirm the exact RPC method surface those indexers call (we can repo-scan Ponder + rindexer and update the v0.1 RPC list accordingly).

