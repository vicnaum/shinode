# Reth Context Files for Stateless History Node

Generated context XML files for building a production stateless history node.

---

## Questions to Answer

### 1. Storage
- How to store block headers, block data, and receipts?
- Which database does reth use?
- What format does it use?
- Does it compress data?

### 2. Fetching
- How to optimally request headers and receipts from other nodes?
- How to queue requests efficiently?
- How to fill holes in data?
- How to handle retries when nodes refuse or timeout?
- How does reth mark which ranges were fetched vs. missing?

### 3. Node Handling
- How does reth regulate sync flow and connection count?
  - Observation: 10 peers → 1150 blocks/sec, 20 peers → 800 blocks/sec, 50 peers → 500 blocks/sec
  - Does reth auto-scale based on throughput? Or is this custom?
- How does it filter, ban, or blacklist misbehaving nodes?
- Does it persist peer information for reuse on restart?

### 4. Keeping Up to Date
- How is historical sync vs. live sync organized?
- What's the algorithm for catching up after historical sync completes?
  - (Sync history → sync recent gap → smaller gap → head → follow tip)
- How does reth know which block is the current head?
- How does it fetch newly produced blocks?
- You mentioned it uses CL for latest blocks - how? Does it have it's own CL node?

### 5. RPC
- What's needed to implement RPC for a limited node (events/logs, block info only)?
- How to make it compatible with indexers like Ponder or rindexer?
- How to protect RPC (localhost only, Docker internal)?
- What auth/binding options does reth provide?

### 6. Serving
- How can we serve data to other P2P nodes?
- Which components handle incoming peer requests (GetBlockHeaders, GetReceipts, etc.)?
- How does reth respond to these requests?

---

## Summary

| File | Tokens | Focus |
|------|--------|-------|
| `context-storage.xml` | 387K | Database, compression, static files |
| `context-sync.xml` | 263K | Stages, pipeline, downloaders, config |
| `context-network.xml` | 375K | P2P stack, discovery, peers, serving |
| `context-rpc.xml` | 216K | RPC server, eth API, filters/logs |
| `context-engine.xml` | 176K | Engine tree, live sync, consensus |
| **Total** | **~1.4M** | |

---

## Question → Context File Mapping

### 1. Storage (Database, Format, Compression)
**Primary:** `context-storage.xml`

Key components:
- `crates/storage/db/` - **MDBX** database implementation (Lightning Memory-Mapped Database fork)
- `crates/storage/db-api/` - Database traits and table definitions
- `crates/storage/nippy-jar/` - **NippyJar** compression format (zstd/lz4)
- `crates/storage/codecs/` - Encoding/decoding (Compact encoding scheme)
- `crates/static-file/` - Static file storage for immutable historical data (headers, receipts, transactions)
- `docs/crates/db.md` - Database documentation

Key files to read:
- `crates/storage/db-api/src/tables/mod.rs` - All database table definitions
- `crates/storage/nippy-jar/src/lib.rs` - Compression implementation
- `crates/static-file/types/src/segment.rs` - Static file segments (Headers, Receipts, Transactions)

### 2. Fetching (Queueing, Hole Filling, Retries)
**Primary:** `context-sync.xml` + `context-network.xml`

Key components:
- `crates/net/downloaders/src/headers/reverse_headers.rs` - **ReverseHeadersDownloader** (key algorithm!)
- `crates/net/downloaders/src/bodies/bodies.rs` - Bodies downloader with batching
- `crates/stages/stages/src/stages/headers.rs` - Headers stage orchestration
- `crates/stages/stages/src/stages/bodies.rs` - Bodies stage orchestration
- `crates/stages/api/src/pipeline/mod.rs` - Pipeline that runs all stages in sequence
- `crates/net/network/src/fetch/mod.rs` - Fetch client for network requests

Key files to read:
- `crates/net/downloaders/src/headers/reverse_headers.rs` - Shows gap filling, buffering, validation
- `crates/stages/types/src/checkpoints.rs` - How progress/checkpoints are tracked

### 3. Node Handling (Speed, Banning, Persistence)
**Primary:** `context-network.xml`

Key components:
- `crates/net/network/src/peers.rs` - **PeersManager** - central peer management (28K tokens!)
- `crates/net/network-types/src/peers/reputation.rs` - Reputation scoring system
- `crates/net/network-types/src/peers/config.rs` - Peer configuration (max connections, backoff, etc.)
- `crates/config/src/config.rs` - Node configuration including peer persistence

Key files to read:
- `crates/net/network/src/peers.rs` - Peer scoring, banning, connection limits
- `crates/net/network-types/src/peers/reputation.rs` - Reputation values (TIMEOUT, BAD_MESSAGE, etc.)
- `crates/config/src/config.rs` - `PeersConfig` for limits and persistence

### 4. Keeping Up to Date (Historical vs Live Sync)
**Primary:** `context-sync.xml` + `context-engine.xml`

Key components:
- `crates/stages/api/src/pipeline/mod.rs` - Pipeline runs historical sync stages
- `crates/engine/tree/src/tree/mod.rs` - **EngineTree** handles live sync with consensus
- `crates/engine/tree/src/backfill.rs` - Backfill coordination
- `crates/engine/tree/src/download.rs` - Download coordination for live blocks
- `crates/chain-state/src/in_memory.rs` - In-memory chain state for recent blocks

Key concepts:
1. **Historical sync**: Pipeline runs stages (Headers → Bodies → etc.) until tip
2. **Live sync**: Engine takes over, receives blocks from consensus layer
3. **Backfill**: When a gap is detected, engine triggers backfill via pipeline

Key files to read:
- `crates/engine/tree/src/tree/mod.rs` - Main engine logic (fork choice, block insertion)
- `crates/engine/tree/src/backfill.rs` - Backfill state machine
- `crates/stages/stages/src/sets.rs` - Stage set definitions

### 5. RPC (Server, eth_getLogs, Protection)
**Primary:** `context-rpc.xml`

Key components:
- `crates/rpc/rpc-builder/src/lib.rs` - **RpcModuleBuilder** - builds RPC server
- `crates/rpc/rpc/src/eth/filter.rs` - `eth_getLogs` implementation (15K tokens!)
- `crates/rpc/rpc-eth-api/src/core.rs` - Eth API trait definitions
- `crates/node/core/src/args/rpc_server.rs` - CLI args for RPC (host binding, auth, CORS)
- `crates/rpc/rpc-builder/src/auth.rs` - JWT authentication for engine API

Key files to read:
- `crates/rpc/rpc/src/eth/filter.rs` - Filter implementation, log matching
- `crates/rpc/rpc-eth-types/src/logs_utils.rs` - Log matching utilities
- `crates/node/core/src/args/rpc_server.rs` - Host binding options (localhost protection)

### 6. Serving (Responding to Peer Requests)
**Primary:** `context-network.xml`

Key components:
- `crates/net/network/src/eth_requests.rs` - **EthRequestHandler** - handles GetBlockHeaders, GetBlockBodies, GetReceipts, etc.
- `crates/net/eth-wire-types/src/message.rs` - All ETH protocol messages
- `crates/net/network/src/session/active.rs` - Active session handling

Key files to read:
- `crates/net/network/src/eth_requests.rs` - How incoming requests are processed
- `crates/net/eth-wire-types/src/receipts.rs` - Receipts message encoding

---

## Regenerating Context Files

```bash
cd reth

# Storage context (387K tokens)
repomix \
    --include "crates/storage/db/src/**,crates/storage/db/Cargo.toml,crates/storage/db-api/src/**,crates/storage/db-api/Cargo.toml,crates/storage/provider/src/**,crates/storage/provider/Cargo.toml,crates/storage/nippy-jar/src/**,crates/storage/nippy-jar/Cargo.toml,crates/storage/codecs/src/**,crates/storage/codecs/Cargo.toml,crates/storage/storage-api/src/**,crates/storage/storage-api/Cargo.toml,crates/static-file/types/src/**,crates/static-file/static-file/src/**,docs/crates/db.md" \
    --ignore "**/tests/**,**/benches/**,**/testdata/**" \
    -o context-storage.xml

# Sync context (263K tokens)
repomix \
    --include "crates/stages/stages/src/**,crates/stages/api/src/**,crates/stages/types/src/**,crates/net/downloaders/src/**,crates/net/network/src/peers.rs,crates/net/network/src/fetch/**,crates/net/network/src/eth_requests.rs,crates/net/network-types/src/peers/**,crates/chain-state/src/**,crates/config/src/**,docs/crates/stages.md" \
    --ignore "**/tests/**,**/benches/**,**/test_utils/**" \
    -o context-sync.xml

# Network context (375K tokens)
repomix \
    --include "crates/net/network/src/**,crates/net/network-api/src/**,crates/net/network-types/src/**,crates/net/dns/src/**,crates/net/discv4/src/**,crates/net/discv5/src/**,crates/net/eth-wire/src/**,crates/net/eth-wire-types/src/**,crates/net/p2p/src/**,examples/manual-p2p/**,docs/crates/network.md,docs/crates/eth-wire.md" \
    --ignore "**/tests/**,**/benches/**,**/testdata/**,crates/net/network/src/transactions/**" \
    -o context-network.xml

# RPC context (216K tokens)
repomix \
    --include "crates/rpc/rpc-builder/src/**,crates/rpc/rpc/src/eth/**,crates/rpc/rpc-eth-api/src/**,crates/rpc/rpc-eth-types/src/**,crates/rpc/rpc-server-types/src/**,crates/rpc/rpc-api/src/**,crates/node/core/src/args/rpc_server.rs" \
    --ignore "**/tests/**,**/benches/**" \
    -o context-rpc.xml

# Engine context (176K tokens)
repomix \
    --include "crates/engine/tree/src/**,crates/engine/service/src/**,crates/engine/primitives/src/**,crates/consensus/consensus/src/**,crates/consensus/common/src/**" \
    --ignore "**/tests/**,**/benches/**,**/test_utils/**" \
    -o context-engine.xml
```

---

## Quick Answers to Your Questions

### 1. Storage - Which database?
**MDBX** (fork of LMDB). Immutable historical data (headers, receipts) goes into **static files** using **NippyJar** format with zstd compression. This separates hot data (recent blocks, state) from cold data (historical).

### 2. Fetching - How does it track ranges?
Uses **checkpoints** stored in database (`StageCheckpoint`). Each stage tracks its progress block number. The `ReverseHeadersDownloader` downloads headers backwards from tip, buffering and validating as it goes.

### 3. Node handling - Speed optimization?
`PeersManager` in `peers.rs` uses **reputation scoring**. Peers get penalized for timeouts, bad messages, slow responses. There's `max_outbound_peers` limit. Reth doesn't auto-scale based on throughput - that's custom optimization you'd need to implement. Peer info is persisted to `known-peers.json`.

### 4. Historical vs Live sync?
1. **Pipeline** runs stages sequentially for historical sync
2. When near tip, **Engine** takes over via consensus layer messages
3. If gap detected, engine triggers **backfill** which runs pipeline for missing range
4. For your case: you'd use pipeline for history, then switch to listening for new block announcements via P2P

### 5. RPC localhost protection?
`--http.addr 127.0.0.1` (default). See `crates/node/core/src/args/rpc_server.rs` for all options. JWT auth is for Engine API only.

### 6. Serving to other nodes?
`EthRequestHandler` in `eth_requests.rs` handles incoming GetBlockHeaders/GetBlockBodies/GetReceipts requests. It reads from local storage and responds.
