# Stateless History Node SPEC

This repo currently contains a working **receipt availability harness** (see Appendix A). The next product is a **stateless history node**: a long-running service that ingests history artifacts from the Ethereum Execution Layer (EL) P2P network and serves an indexer-compatible RPC subset.

## Current status (v0.1.4)
- Range backfill (single run) using P2P headers/bodies/receipts.
- Static-file persistence (NippyJar) for headers, tx hashes, tx metadata (no calldata;
  signature + signing hash stored for sender recovery), receipts, and block size.
- Logs are derived on demand; withdrawals are not stored.
- Minimal RPC subset with query limits (`eth_chainId`, `eth_blockNumber`,
  `eth_getBlockByNumber`, `eth_getLogs`).
- `eth_getBlockByNumber` returns full block shape; `totalDifficulty` is mocked to `0x0`.
- Operator basics: CLI config, verbosity flags, progress bar, graceful shutdown.
- Benchmark modes: probe (headers/receipts only) and ingest (full pipeline timing).
- Not yet: follow mode, live reorg handling, extra RPC methods, metrics export.

## Goals
- **Ship a usable v0.1 MVP quickly**: backfill → follow head → persist → serve RPC.
- **Stay stateless**: no EVM execution, no state trie/database, no archive-state growth.
- **Future-proof architecture from day 1**: clear boundaries, stable storage contract, minimal rewrites.
- **Reuse Reth components and patterns** wherever feasible:
  - Networking (`reth/crates/net/**`)
  - Proven download/concurrency patterns (see `spec/reth_kb` Q034/Q035)
  - Storage primitives (static-file patterns) and safety knobs
  - RPC safety defaults and query limits (see `spec/reth_kb` Q021/Q022/Q023)

## Non-goals (for v0.1)
- Full execution/state, archive state, or EVM traces.
- Transaction pool or mempool APIs.
- “Full node” correctness guarantees (the trust model is intentionally pragmatic in early versions).

## MVP v0.1: “Synced History RPC for Indexing”

### What “end-to-end usable” means
- **Backfill** from a configured `start_block` to a moving “head”.
- **Follow** new blocks continuously.
- **Persist** data restart-safely and idempotently (no duplicates, no corruption).
- **Serve RPC** sufficient for the v0.1 target indexer (**rindexer**) to run end-to-end:
  - Required: `eth_chainId`
  - Required: `eth_blockNumber`
  - Required: `eth_getBlockByNumber` (including `"latest"`)
  - Required: `eth_getLogs`
  - Optional (v0.2+): additional endpoints for other indexers (see note below)
- **Handle reorgs** within a configured rollback window (v0.1 default: delete-on-rollback).

### Note: Ponder baseline requires stateful RPC
Ponder’s baseline “normal app” mode includes **`eth_call`** (multicall3 / read-contract / some factory flows) and uses **`eth_getBlockByHash`** for realtime reorg traversal. Since `eth_call` requires EVM state, a fully stateless history node cannot serve it without either:
- running execution/state (out of scope), or
- proxying `eth_call` to an upstream RPC (optional future integration), or
- a stateless execution approach (e.g., Reth’s RESS subsystem) if feasible for the required calls.

### Data we must ingest (minimum set)
To serve `eth_getLogs` correctly (including `transactionHash`/`transactionIndex`/`logIndex`), we need:
- **Headers** (canonical chain): number/hash/parent/timestamp + `logsBloom`
- **Tx hashes per block** (from block bodies or equivalent)
- **Receipts** per block (from `GetReceipts*`), then derive logs with full metadata

### Retention modes (persisted settings)
Retention is a product-contract decision and must be explicit up-front to avoid DB redesign:
- **v0.1 (simple default)**: store headers + tx hashes + tx metadata (no calldata) +
  full receipts; logs derived at query time; withdrawals not stored
- **Later (v0.2+)**:
  - optional “filtered logs only” retention
  - optional tx calldata retention (`input`)

### Canonicality and head source (trust model)
Reth itself relies on an external CL via Engine API for head/finalization (see `spec/reth_kb` Q039).
For our v0.1 stateless history node we can start with a pragmatic model:
- **MVP default**: best-effort head tracking from the P2P view + reorg rollback window
- **Later upgrade**: integrate a local CL (or beacon API) to get `safe`/`finalized` head signals and stronger canonicality

## Architecture outline (inspired by Reth)
High-level components (mirrors reth’s separation of concerns):
1. **P2P**: run devp2p networking using `reth-network`, subscribe to peer events, and issue `GetBlockHeaders` / `GetBlockBodies` / `GetReceipts*` requests.
2. **Sync/ingest orchestrator**: a coordinator loop that decides “what to fetch next” and manages concurrency and retries (informed by reth’s engine download coordination patterns, Q034).
3. **Chain tracker**: maintains the canonical header chain view, detects reorgs, and provides rollback points.
4. **Storage**: a durable store with:
  - schema + versioning
  - append-only static files with rollback via tail pruning
  - logs derived from receipts at query time
5. **RPC**: a minimal JSON-RPC server with safe defaults (localhost bind by default, query limits) and only the namespaces we support.

## Historical fast sync (v0.1.5)
For blocks **older than the reorg window**, we use a fast backfill mode:
- split the historical range into fixed-size chunks (default 32)
- fetch chunks concurrently across peers with bounded in-flight requests
- buffer out-of-order chunks and write in block order
- persist each chunk in a single static-file append batch

For blocks **inside the reorg window**, use the existing safe sequential path.

## References
- Reth knowledge base index: `spec/reth_kb/INDEX.md`
- Reth “what lives where” map: `reth/crates/**/AGENTS.md`

---

## Appendix A: Receipt Availability Harness Spec (Updated)

### Goal
Measure the availability and latency of Ethereum L1 block headers and receipts served over devp2p `eth` without executing transactions or maintaining state. Persist only what is needed for receipts availability analysis and indexing.

### Non-goals
- Full execution / state verification.
- Archive-style state or EVM traces.
- Receipts root validation (explicitly skipped in v1).

### High-level architecture
1. **P2P network layer (Reth as library)**
   - Discovery, dialing, and `eth` handshake.
   - `PeerRequestSender` for headers/receipts.
2. **Scheduler**
   - Shared work queue of target blocks.
   - Per-peer assignment and batching.
   - Soft ban on peers with repeated failures.
3. **Probing pipeline**
   - Header batch → receipt batch per block hash.
   - Eth protocol version-aware receipts requests (`eth/68` and below, `eth/69`, `eth/70`).
4. **Persistence**
   - JSONL logs for requests, probes, stats, and known blocks.

### Fetch flow

#### Peer head resolution
When a peer connects, their `status` message includes their head block hash. We send `GetBlockHeaders` for that hash (limit=1) to resolve the head block number. This determines the maximum block number we can request from that peer.

#### Block batch probing
For each assigned batch of blocks (default 32 contiguous):

```
┌─────────────────────────────────────────────────────────────────┐
│  1. Request Headers                                             │
│     GetBlockHeaders { start: Number(N), limit: 32, skip: 0 }    │
│     → Vec<Header>                                               │
│                                                                 │
│  2. Compute Block Hashes                                        │
│     For each header: blockHash = keccak256(RLP(header))         │
│     (via SealedHeader::seal_slow)                               │
│                                                                 │
│  3. Request Receipts (in chunks of 16)                          │
│     GetReceipts([blockHash1, blockHash2, ...])                  │
│     → Vec<Vec<Receipt>>  (all receipts for each block)          │
│                                                                 │
│  4. Mark Success                                                │
│     If receipts returned for block → mark_known_block()         │
└─────────────────────────────────────────────────────────────────┘
```

#### What we fetch
| Request           | Input                | Output                          |
|-------------------|----------------------|---------------------------------|
| GetBlockHeaders   | block number + limit | Vec\<Header\>                   |
| GetReceipts       | Vec\<blockHash\>     | Vec\<Vec\<Receipt\>\> per block |

#### What we DON'T fetch
- **Block bodies** (transactions, uncles) — not needed for receipts.
- **State** — no execution.
- We do NOT verify `receipts_root` — we only count receipts received.

#### Protocol variants
- **Legacy (eth/68 and below)**: `GetReceipts(Vec<B256>)` → `Receipts(Vec<Vec<Receipt>>)`
- **Eth69**: Same request, different response encoding (`Receipts69`)
- **Eth70**: `GetReceipts70 { first_block_receipt_index, block_hashes }` → `Receipts70`

#### Why block hash is required
The `GetReceipts` message requires **block hashes**, not block numbers. The only way to obtain a block's hash is to fetch its header and compute `keccak256(RLP(header))`. This is why we must fetch headers first.

#### Definition of "known"
A block is marked "known" **only when receipts are successfully received**:
- Header fetch fails → block stays in queue (or escalation)
- Header OK but receipts fail → block stays pending
- Both succeed → entry written to `known_blocks.jsonl`

### Target selection
Targets are constructed from a fixed set of DeFi deployment anchors plus a contiguous window:
- Uniswap V2: 10000835
- Aave V2: 11362579
- Uniswap V3: 12369621
- Aave V3: 16291127
- Uniswap V4: 21688329

The default window is 10,000 blocks per anchor. Set with `--anchor-window`.

### Data stored
Minimal data to enable downstream analysis:
- **Headers:** only for probing (not persisted as canonical chain state).
- **Receipts:** only counts and timing in probe logs.
- **JSONL output:**
  - `requests.jsonl`: request events + timing.
  - `probes.jsonl`: per-block probe results.
  - `known_blocks.jsonl`: blocks proven available.
  - `missing_blocks.jsonl`: blocks not fetched (written on shutdown if any remain).
  - `stats.jsonl`: window stats + summary.

### Retry and error handling
Two-tier retry strategy:
1. **Normal retries**
   - Failed blocks are requeued up to `--max-attempts`.
2. **Escalation pass**
   - Once the main queue is drained, blocks that failed all retries are tried again across distinct peers (one attempt per peer).
   - If every known peer fails, the block is marked unavailable.

All failures are recorded with reason tags (`header_timeout`, `receipts_batch`, `escalation_*`, etc).

### Peer handling
- Track peer health and ban temporarily after consecutive failures.
- Ban duration and failure threshold are configurable.
- Sessions are counted; peer assignment is based on availability and queue.

### Completion criteria
The harness exits when:
1. All targets are completed or failed, and
2. Both normal and escalation queues are empty, and
3. No in-flight work remains.

`--run-secs` provides a hard time limit override.

### CLI behavior
Console output is a real-time progress bar (10Hz) showing:
- % complete and processed/total
- elapsed time, ETA (1-second rolling window)
- speed (blocks/sec, 1-second rolling window)
- peer status (active/total sessions)
- queue length, in-flight count
- failed blocks count

During escalation, a separate red progress bar shows:
- failed remaining/total
- peers tried/total
- escalation queue length

Verbose levels:
- (default): progress bar only, no log output
- `-v`: progress + periodic stats to logs
- `-vv`: adds WARN-level logs (peer failures, timeouts)
- `-vvv`: includes per-request and per-probe JSON logs

### Known limitations
- No receipts root validation.
- Network-only data availability is impacted by EIP-4444 pruning.
- No L2 support yet (Base/Optimism planned).
