Below is a minimal, engineering-focused **spec primer** for a “receipt-availability harness” that measures whether you can fetch **headers + receipts** over **EL devp2p (RLPx + `eth/*`)** on **Ethereum mainnet** and **Base**, and optionally verifies `receiptsRoot`.

---

## 0) Goal and success criteria

### Primary question

For a set of target block numbers (recent + older), **what fraction of peers will serve receipts**, and with what latency/failure modes?

### Secondary question

When receipts are returned, can we **validate them against the header’s `receiptsRoot`** (cheap correctness knob, no EVM/state)?

### Non-goals (for harness)

* No block execution, no state, no snap sync.
* No persistent chain DB.
* Not building a full “history client” yet—just measure and log availability.

---

## 1) Inputs

### Chains

* **ETH mainnet**: `networkId=1`, `genesisHash=0xd4e56740...` (hardcode)
* **Base mainnet**: `chainId=8453` (EVM), but for devp2p you mostly care about the **genesis hash + network id** used by that L2’s execution client.

  * Don’t assume these values: fetch from peer `Status` and *accept only peers matching your expected genesis hash* (you must supply expected genesis hash for Base in config).

### Peer seeds

You need *some* way to get peers:

* ETH: known bootnodes (static list) + discv4/discv5
* Base: whatever bootnodes/ENRs Base publishes; if not published, start with **1–3 known peers** (ENR/enode) and use their peer table to expand.

### Sample blocks

Define block numbers relative to head + a few absolute anchors.

Example (ETH):

* `head - 1_000`
* `head - 100_000`
* `head - 1_000_000`
* fixed post-merge anchor (e.g. ~16–18M)
* fixed pre-merge anchors (e.g. 14M, 12M, 10M)

Example (Base):

* `head - 1_000`
* `head - 100_000`
* `head - 1_000_000` (if chain old enough)
* early anchor: block 1 / 10_000 / 100_000

---

## 2) Outputs / metrics

Produce a JSONL (or SQLite) report with:

### Per peer session

* peer id (pubkey), client name/version if available (via devp2p `Hello` caps, not always precise)
* handshake success/failure reason
* `Status` fields: head hash, forkid, genesis hash, network id
* round-trip stats + disconnect reasons

### Per (peer, block) probe

* block number requested
* header fetch: ok/missing/timeout
* receipts fetch: ok/missing/timeout/throttled/disconnect
* bytes received
* latency: header RTT, receipts RTT
* receiptsRoot check: pass/fail/not-run
* if fail: mismatch type (decode, trie root mismatch, empty receipts vs non-empty root, etc.)

### Aggregate

* availability rate by block (e.g. % peers served receipts)
* availability vs age curve (roughly)
* “best peer” list for backfill (highest success + speed)

---

## 3) Harness architecture

Two layers:

### A) Networking core

* discv4/discv5 (or static peers)
* RLPx session management (ECIES handshake, framing, optional snappy)
* devp2p capability negotiation
* `eth` subprotocol handler (at least: Status, GetBlockHeaders, BlockHeaders, GetReceipts, Receipts)

### B) Probe runner

* connect to N peers (e.g. 20–100)
* for each peer, run the same probe schedule with rate limits
* record results

---

## 4) Protocol flow (per peer)

### Step 1: Discovery / connect

* obtain peer endpoint (IP:port + pubkey via ENR/enode)
* open TCP
* run RLPx handshake
* devp2p `Hello`:

  * require capability `eth` with some version you support (recommend supporting `eth/66` and `eth/67`; many peers will do one)
  * ignore other caps

### Step 2: `eth` handshake

* exchange `Status` message.
* validate:

  * genesis hash matches expected chain
  * network id matches expected chain (if you have it)
  * forkid present/parsable (don’t hard-fail unless it’s wildly wrong)

If mismatch => disconnect and record `status_mismatch`.

### Step 3: Determine head number

From `Status` you get `headHash` but not head number.

* Request header by hash:

  * `GetBlockHeaders` with `origin = headHash`, `amount=1`, `skip=0`, `reverse=false`
* Parse returned header, get `headNumber`.
* Now you can compute `head - delta` sample numbers.

### Step 4: For each sample block number `n`

**Header fetch**

* `GetBlockHeaders` by number: `origin=n`, `amount=1`.
* Require exactly one header and `header.number == n`.
* Record `header.hash`, `header.receiptsRoot`.

**Receipt fetch**

* `GetReceipts` with `[header.hash]`.
* Expect response: receipts list for that block.

**Optional: receiptsRoot validation**
Compute trie root of returned receipts and compare to `header.receiptsRoot`.

---

## 5) Receipts decoding + receiptsRoot validation

### Receipt encoding reality (must handle this)

* Legacy receipts (type 0) are plain RLP list.
* Typed receipts (EIP-2718) are `typeByte || rlp(payload)` **as raw bytes**; trie value is those raw bytes (type prefix included).

So your decoder must accept:

* first byte < 0xc0 (likely typed): treat it as `type`, remainder is RLP payload
* else treat as legacy RLP

### Trie root computation

* Build a **secure MPT** (hexary trie, keccak-hashed nodes).
* Insert key/value pairs in **tx index order**:

  * key = `RLP(txIndex)` where `txIndex` is the transaction index starting at 0
  * value = the **exact receipt encoding bytes** as above
* Root hash is `receiptsRoot`.

### Edge cases

* Empty block: receipts list should be empty.

  * empty receipts trie root is the standard empty trie root:
    `0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421`
* Some peers might return malformed/partial receipts or disconnect—treat as failure.

---

## 6) Sampling strategy

### Keep it cheap and representative

You don’t need 10k blocks. You need a few “ages” + a few repeats.

Recommended:

* per chain: ~10–20 sample blocks total
* per peer: probe all samples, but with a per-peer timeout and per-message timeout.

### Suggested ETH set

* `head - 1k`, `head - 100k`, `head - 1M`
* fixed: 18,000,000; 17,000,000; 16,000,000
* pre-merge: 15,000,000; 14,000,000; 12,000,000; 10,000,000

### Suggested Base set

* `head - 1k`, `head - 100k`
* early anchors: 100,000; 1,000,000 (if exists); maybe 5,000,000

---

## 7) Concurrency + rate limits (avoid getting dropped)

Per peer:

* sequential requests (header then receipts)
* sleep jitter 50–150ms between requests
* hard cap: e.g. 20–50 requests per peer
* timeouts: connect 3s, handshake 3s, per request 2–5s

Across peers:

* connect concurrently to e.g. 32 peers max
* total run should finish in minutes, not hours

---

## 8) Canonicality / eclipse resistance (minimal)

For a harness, you can accept “peer says header/receipt” as long as receiptsRoot matches that header. But you can cheaply harden:

* For each block number, collect `(headerHash, receiptsRoot)` from multiple peers.
* Treat “canonical” as **majority header hash**.
* Only count receipt success if it matches the majority header hash.

This avoids measuring an attacker’s private fork.

---

## 9) Implementation notes (minimal effort)

### Don’t reimplement RLPx from scratch unless you really want to

Use an existing stack and implement only the probe logic.

Good options:

* **Rust**: leverage `reth-network` / `discv5` + reuse their eth protocol codec; then your harness is mostly “connect → status → request → measure”.
* **Go**: reuse geth’s `p2p` + `eth/protocols/eth` packages (same idea).
* **TS**: `@ethereumjs/devp2p` exists, but you’ll fight performance and subtle encoding details.

### Separate “protocol correctness” from “measurement”

Implement a deterministic state machine per peer; record every transition + errors.

---

## 10) What “success” looks like after you run it

You should be able to answer, for ETH and Base:

* Up to what block age do random peers reliably serve receipts?
* Is there a cliff (e.g. pre-merge)?
* Which clients/peers are most reliable?
* Do receiptsRoot checks usually pass when data is returned?

That directly tells you whether:

* pure P2P backfill from 2021 is viable, or
* you need `era1`/Portal/seeders for backfill, while P2P handles the tail.

---

If you tell me your preferred language (Rust/Go/TS) and whether you’re okay with using an existing p2p stack (reth/go-ethereum), I can refine this into:

* concrete module boundaries,
* exact message structs you need,
* and a minimal test plan (unit tests for typed receipts + receiptsRoot).

---

If you want to build the harness on top of “Reth stack” without accidentally embedding a whole node, think of it as:

* **Reuse Reth’s networking + codecs + primitives**
* **Write your own orchestrator (probe logic) + verification + reporting**

Below is the clean split.

---

## What you can take from Reth (recommended)

### 1) Discovery + peer management

Use Reth’s discovery and peer stores rather than rolling your own ENR parsing / discv4/5.

* **discv4/discv5 integration** (find peers, keep a peer table, filter by forkid/genesis/network):

  * Use whatever Reth exposes for discovery (often wraps `discv5` crate + Reth peer records).
* **Peer scoring / banning / backoff utilities** (optional but useful).

**Why reuse:** Discovery edge cases + timeouts + NAT weirdness is a time sink.

**Fallback for Base:** you may not find many public bootnodes / ENRs. In practice, start with **static peers** (enode/enr) and then crawl their peer tables.

---

### 2) RLPx / devp2p transport

Reth has the whole RLPx session machinery:

* ECIES handshake
* framing
* snappy compression negotiation
* devp2p `Hello` / disconnect handling
* multiplexing subprotocol streams (`eth`, `snap`, …)

**Why reuse:** Handshake correctness and disconnect reasons are painful to debug if you reimplement.

---

### 3) `eth/*` wire protocol message types + encoding/decoding

You want to reuse the exact message structs and codecs for:

* `Status`
* `GetBlockHeaders` / `BlockHeaders`
* `GetReceipts` / `Receipts`
  (and optionally `GetBlockBodies` / `BlockBodies`)

Reth has these types in its “eth-wire” layer (message enums + RLP). Use them.

**Why reuse:** You’ll avoid subtle RLP and typed-receipt encoding mistakes.

---

### 4) Primitives / hashing

Reuse:

* `Header`, `BlockHash`, `B256`, `U256`, etc.
* Keccak implementation

**Why reuse:** Consistency across decoding and trie construction.

---

## What you should write yourself (minimal harness core)

### A) Chain config + acceptance rules

You must define per-chain:

* expected `genesisHash`
* expected `networkId` (if used by that chain’s EL p2p)
* forkid validation policy (strict/lenient)

For ETH: easy/hardcoded.
For Base: you likely need a Base chainspec / genesis hash from a trusted source (or your own Base node once, then hardcode).

---

### B) Probe orchestrator (the real harness)

This is not in Reth because Reth’s sync pipeline is not “measure availability”.

You write:

* peer selection (N peers)
* probe plan (block heights)
* per-peer sequencing (header→receipts)
* concurrency limits (tokio tasks)
* timeouts and retry policy
* metrics aggregation + JSONL output

---

### C) ReceiptsRoot verifier

Reth gives you receipts decoding. It won’t hand you a “verify receiptsRoot for arbitrary receipt bytes” function in a stable public API.

Write a small verifier module that:

* takes `Vec<ReceiptEnvelope>` (typed/legacy)
* re-encodes each receipt exactly as it appears in trie value:

  * legacy: RLP(receipt)
  * typed: `typeByte || RLP(payload)`
* inserts into an MPT with keys `RLP(txIndex)`
* compares root to `header.receipts_root`

You can optionally use a trie utility crate (Reth’s trie or a small standalone triehash library). Either way, *the harness owns this logic*.

---

### D) Canonicality heuristic (optional, but good)

To avoid measuring an eclipse fork:

* For each probed block number, record `(hash, receiptsRoot)` from peers.
* Pick the majority header hash as canonical for that run.
* Count receipt success only if it matches that hash.

---

### E) Persistence/reporting

* JSONL line per probe result
* plus summary stats per block age

Keep it stupid-simple; the harness is disposable.

---

## Two viable integration styles with Reth

### Option 1 (recommended): “Use Reth networking as a library”

You use Reth’s network manager / session layer, and you implement a tiny “eth request client” wrapper that can send `GetBlockHeaders` and `GetReceipts` and await responses.

**Pros:** least code, most robust
**Cons:** Reth’s internal networking APIs can change between versions—pin a git tag.

### Option 2: “Low-level RLPx session + eth-wire only”

You still reuse Reth’s RLPx/session crate + eth-wire messages, but you manage:

* connecting sockets
* session lifecycle
* request IDs + response matching

**Pros:** fewer dependencies on Reth internals
**Cons:** more glue code, still some complexity

Given your “bundle up” goal, Option 1 is typically faster, but you *must* pin versions.

---

## Minimal module breakdown (what I’d implement)

1. `chain.rs`

* `ChainConfig { name, genesis_hash, network_id, bootnodes/static_peers }`

2. `peer_source.rs`

* discovery or static peers
* yields `PeerAddr { enr/enode, socket_addr }`

3. `eth_client.rs`

* `connect(peer) -> Session`
* `status() -> Status`
* `get_header_by_hash(hash) -> Header`
* `get_header_by_number(n) -> Header`
* `get_receipts(block_hash) -> Vec<Receipt>`

4. `verify.rs`

* `compute_receipts_root(receipts) -> B256`
* `verify_receipts_root(header, receipts) -> bool`

5. `runner.rs`

* per-peer probe loop
* block sampling strategy
* rate limiting / timeouts

6. `report.rs`

* JSONL writer + summary aggregation

---

## Practical gotchas (so you don’t waste a week)

* **Peer availability bias:** Many peers accept RLPx but drop requests quickly. Track disconnect reason codes.
* **GetReceipts often fails before GetBlockHeaders does.** Treat them separately.
* **Base peer discovery may be thin.** Expect to start with static peers (from your own node once, or public lists).
* **Avoid looking like a DoS client:** slow down, randomize peers, cap requests.

---

## What you’ll *not* need from Reth

* DB / storage layers
* execution / EVM
* sync pipeline
* RPC server
* txpool

Keep your dependency surface to: networking + wire + primitives (+ trie helper).

---

If you tell me which direction you prefer:

* **Option 1 (network manager style)** or **Option 2 (low-level session)**,

I can outline the exact “call graph” you’ll implement (connect → Hello → Status → request/response), and the minimal dependency set (crate list + features) to keep the binary small and compilation sane.

---

