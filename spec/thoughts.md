I’ll propose a concrete phased plan for ETH L1 and Base that minimizes storage and paid RPC: quick experiments to validate whether P2P receipts are still fetchable for your target ranges, then a robust design with fallbacks (Portal + era1 + community archives) and optional stronger validation.


You can make this *not* depressive by changing one assumption:

**You don’t need to store “chain receipts” to index a few contracts.**
You only need to *fetch* full receipts to extract logs and (optionally) check `receiptsRoot`. After that you can **discard most of the receipt payload** (esp. `logsBloom`) and keep only:

* canonical headers (for timestamps + chain structure),
* tx-hash list (or txIndex→txHash),
* the logs you care about (+ minimal per-log metadata).

That turns storage from “~200GB receipts” into “size proportional to your contracts’ event volume”.

---

## Reality check: P2P history availability on Ethereum L1 (post EIP-4444 direction)

EIP-4444’s long-term goal is: peers stop serving old bodies/receipts on p2p. The EIP itself says clients must stop serving historical headers/bodies/receipts older than one year on p2p, and may prune locally. ([Ethereum Improvement Proposals][1])

But what’s *actually deployed right now* is **partial history expiry**: dropping **pre-merge** block data to save ~300–500GB. The Ethereum Foundation post explicitly frames “partial history expiry” as pre-merge pruning; “full, rolling history expiry is ongoing.” ([Ethereum Foundation Blog][2])

Implication:

* Anything **pre-merge** (<= 2022-09-15) will become increasingly unavailable via random peers.
* Post-merge history is still widely served *for now*, but you should assume it will get harder over time.

So: **P2P is great for the rolling head / recent window. For backfill (especially pre-merge), plan on archives.**

---

## How big are `era1` archives?

`era1` is specifically **pre-merge history** (genesis → merge block 15,537,393). It’s chunked (e.g. block 0–8191 per file). ([ethpandaops.io][3])
There’s a commonly referenced **~427GB** torrent / set of era files for pre-merge history. ([Ethereum Research][4])

Key point for you: because it’s **range-addressable**, you can download *only the files covering your startBlock…endBlock*, not 427GB.

---

# What to try first (ETH L1)

### Experiment A: P2P receipts availability vs block age

Write a quick harness that:

1. finds peers (DNS discovery),
2. does RLPx handshake + `eth/66`,
3. sends `GetReceipts` for a few sample blocks at different ages:

   * head-1k
   * head-100k
   * head-1M
   * a known pre-merge block (e.g. 14M, 12M, 10M)
4. records: success rate, latency, and peer client fingerprint.

This will tell you immediately whether “pure p2p backfill from 2021” is realistic (it’s trending toward “no”).

### Experiment B: receiptsRoot validation cost

Implement receipts trie root recomputation and confirm it matches header `receiptsRoot`.

* You’ll need full receipts for the block.
* You do **not** need state or execution.

This is the strongest “stateless correctness” knob you can turn without EVM.

### Minimum viable ETH architecture (works even if old p2p dies)

**Ingest pipeline**

* Follow canonical chain via headers.
* For each canonical block in your target range:

  * fetch receipts (and bodies only if you need tx hashes / from/to/value)
  * optionally verify `receiptsRoot`
  * extract logs relevant to your watched contracts/topics
  * persist:

    * header
    * tx-hash list (or txIndex→txHash)
    * relevant logs (+ blockNumber, txIndex, logIndex, address, topics, data)

**Storage**

* Headers are tiny.
* Tx hashes are manageable.
* Logs are the main volume, but only for your contracts.
* You can *discard* raw receipts after extraction/verification.

**Backfill sources (in priority order)**

1. P2P receipts/bodies (if available for your range)
2. `era1` for pre-merge ranges (selective download by block range) ([ethpandaops.io][3])
3. Portal as a “missing block finder” (only for holes, not primary throughput path)

---

# What to try first (Base)

Base is younger, so your “start at Uniswap deploy” might be within 2023–2026 only, which helps.

### Experiment A (Base): does op-reth/op-geth p2p serve receipts?

Run the same harness against Base network peers:

* Try `GetReceipts` for blocks at:

  * head-1k
  * head-100k
  * early chain (near genesis-ish)
* Measure hit rates.

If it works: you can do the exact same stateless pipeline as Ethereum (headers + receipts, optional bodies).

### If Base p2p history is insufficient: your realistic backups

Base’s official snapshots are indeed huge. Their docs point to:

* Mainnet **geth full** snapshot
* Mainnet **reth archive** snapshot (recommended for archive)
  and note geth archive snapshots aren’t supported. ([docs.base.org][5])
  A Base repo issue mentions a mainnet snapshot around **3.4TB zipped** (and much larger extracted). ([GitHub][6])

So you want a fallback that *doesn’t require every user to unpack multi-TB*:

**Backup 1: “Receipts-only seeders” (pragmatic, low engineering risk)**

* You (or a small group) run a heavy node *once* (or continuously).
* You export **(headers, tx hashes, logs)** into your compact format.
* Everyone else syncs the compact log DB + only uses p2p for the tail.

This is basically “Portal for Base”, but centralized/permissioned at first. Given your “stage-1 pragmatic trust”, this is the simplest way to avoid forcing multi-TB bootstraps.

**Backup 2: “Range snapshots”**
Instead of one huge snapshot, publish **range-indexed chunks**:

* e.g. 100k-block chunks containing:

  * headers
  * tx hash lists
  * extracted logs for watched contracts (or even full receipts if you insist)
    served over HTTP + checksums / torrent.

This matches how `era1` works conceptually, but for Base.

**Backup 3: Derivation + execution (last resort)**
If you truly must be trust-minimized on Base without relying on peers serving receipts, you end up running:

* op-node derivation (from L1 batch data)
* execution client
  to generate receipts yourself.

You can still keep *your* final artifact stateless (just logs DB), but bootstrapping becomes heavier.

---

# Recommended phased plan (ETH + Base)

### Phase 0: pick your “stateless artifact”

Default to:

* headers
* tx hashes (per block)
* extracted logs for watched contracts/topics
* optional “proof meta”: `receiptsRoot matched` boolean + peer set / source IDs

Avoid storing full receipts long-term unless you’re intentionally trying to be a general-purpose receipts server.

### Phase 1: implement P2P tail ingestion (both chains)

* Headers sync
* Receipts fetch
* Reorg handling
* Optional receiptsRoot check

### Phase 2: backfill from deploy block

* Try p2p first
* If missing:

  * ETH pre-merge: `era1` range download
  * Base: your own range snapshots / seeder

### Phase 3: make it “useful peer” without storing everything

Two options:

* Serve **only your retained range** (since Uniswap deploy) and only the message types you support.
* Or serve **your extracted log DB** via a custom protocol/API (more useful for indexers than raw receipts anyway).

---

## Concrete “next step” I’d do this week

1. Build the **receipt-availability harness** (ETH mainnet + Base).
2. Build the **receiptRoot verifier** (it’s your key correctness knob).
3. Decide your retention format (raw receipts vs extracted logs). I’d start with *extracted logs + tx hashes*, because it makes the whole project viable on sane disks.

If you want, paste:

* target chain (ETH/Base),
* target contract addresses (or “Uniswap v4 core”),
  and I’ll specify exactly what you must store to answer the indexer queries you care about (and what you can safely drop).

[1]: https://eips.ethereum.org/EIPS/eip-4444?utm_source=chatgpt.com "EIP-4444: Bound Historical Data in Execution Clients"
[2]: https://blog.ethereum.org/2025/07/08/partial-history-exp "Partial history expiry announcement | Ethereum Foundation Blog"
[3]: https://ethpandaops.io/data/history/ "Ethereum History Endpoints | ethPandaOps"
[4]: https://ethresear.ch/t/torrents-and-eip-4444/19788?utm_source=chatgpt.com "Torrents and EIP-4444 - Execution Layer Research"
[5]: https://docs.base.org/base-chain/node-operators/snapshots "Node Snapshots - Base Documentation"
[6]: https://github.com/base-org/node/issues/256?utm_source=chatgpt.com "Suggest right disk size requirement for base node #256"
