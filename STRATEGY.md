# Strategy: Stateless History Node in the Ethereum Roadmap

> Last updated: 2025-02 (based on analysis of [The Ethereum Stateless Book](https://stateless.fyi/) and SIC call notes through early 2025)

## Where We Fit

Ethereum's roadmap ("The Verge") is unbundling three concerns that full nodes currently handle together:

| Role | What it stores | Who does it today | Who does it tomorrow |
|------|---------------|-------------------|---------------------|
| **Consensus Verification** | Current state (balances, storage) | Full nodes (Geth, Reth) | Stateless clients (verify via witnesses) |
| **History Availability** | Headers, receipts, logs, txs | Full nodes | **Dedicated history nodes (us)**, Portal Network, Era archives |
| **Block Building** | Mempool, state | Full nodes | Specialized builders (PBS) |

**We are building category 2.** As EL clients go stateless and drop history (EIP-4444), dedicated history infrastructure becomes essential — not optional.

## The Tailwinds

### EIP-4444: History Expiry (Our biggest driver)
Full nodes will stop serving historical data beyond a rolling window. Indexers (The Graph, Ponder, Goldsky, etc.) that depend on `eth_getLogs` and `eth_getBlockByNumber` will need a new backend. That's us.

### Stateless Clients Can't Serve History
The Stateless Book explicitly states that stateless clients "can't be block builders" and won't serve historical queries efficiently. They optimize for *verification*, not *data serving*. The more successful statelessness becomes, the more our node is needed.

### Portal Network Synergy
Portal Network is being developed as a decentralized history distribution layer. We can serve as a **Portal Network seeder** — a reliable, high-performance source that feeds the P2P history network while also serving direct JSON-RPC to indexers.

## The Landscape Shift

### Binary Trees Replace Verkle Trees
The stateless community is pivoting from Verkle Trees to **Binary State Trees** (EIP-7864), driven by:
- Better SNARK proving performance
- Quantum resistance (Verkle uses Elliptic Curves)
- Likely hash function: Blake3 or Poseidon2

**Impact on us:** Minimal. We don't verify state roots. But the hash function choice affects ecosystem tooling.

### State Conversion via Overlay (EIP-7612/7748)
The transition uses an overlay approach: old MPT becomes read-only, new writes go to the new tree, background process converts.

**Impact on us:** None. This is purely state-side. We index history, not state.

### SSZ Migration
The push toward statelessness is coupled with moving from **RLP to SSZ** (Simple Serialize) for wire formats and potentially execution data.

**Impact on us:** Medium-term. Reth handles P2P wire format upstream, but we should ensure our types can handle SSZ-encoded data when it arrives. Our internal storage (bincode) is unaffected.

### Execution Witnesses in Block Bodies
Blocks will include **Execution Witnesses** (state proofs needed for stateless verification). Median witness size: ~300-700KB per block.

**Impact on us:** Our P2P decoder must handle or strip witness fields from BlockBody payloads. We don't need to store or verify witnesses, but we can't crash on them.

## What We Should Do

### High Priority
| Action | Rationale |
|--------|-----------|
| Monitor eth wire protocol changes for witness fields | BlockBody payloads will grow; our P2P layer must gracefully handle new fields |
| Track SSZ migration timeline | Wire format changes affect our data ingestion pipeline |
| Position as "History Availability Layer" | Not just an indexer backend — a fundamental piece of post-4444 infrastructure |

### Medium Priority
| Action | Rationale |
|--------|-----------|
| Investigate Portal Network integration | Become a Portal Network bridge/seeder for history data |
| Track gas cost changes (EIP-4762) | New gas schedule affects receipt/log sizes and query patterns |
| Consider Era file export | Standard archive format for historical data distribution |

### Low Priority / Monitor
| Action | Rationale |
|--------|-----------|
| Binary Tree vs Verkle debate outcome | Doesn't affect history, but affects ecosystem direction |
| State expiry mechanics | Affects state trie, not receipt/log storage |
| Preimage distribution | Only relevant if we ever verify state roots (we don't) |

## What We Can Ignore

- **Overlay Tree logic (EIP-7612/7748)** — purely state-side
- **Code chunking (31/32-byte chunkers)** — EVM internals
- **Preimage distribution** — only for state verification
- **Block hash system contract (EIP-2935)** — changes EVM access to block hashes, not P2P header format

## Key EIPs Reference

| EIP | Topic | Relevance | Status |
|-----|-------|-----------|--------|
| **EIP-4444** | History Expiry | **Critical** — our raison d'etre | Active Discussion |
| **EIP-7864** | Binary State Tree | Low — doesn't affect history | Rising (replacing Verkle) |
| **EIP-4762** | Gas Cost Changes (Stateless) | Medium — affects receipt sizes | Draft |
| **EIP-7612** | Overlay Tree Transition | None — state-side only | Draft |
| **EIP-7748** | State Conversion Scheduling | None — state-side only | Draft |
| **EIP-2935** | Block Hash in State | Low — EVM internal change | Included in Pectra |
| **EIP-6800** | Verkle State Tree | Low — being superseded by 7864 | Superseded |

## Strategic Questions

1. **Indexer backend vs. History Availability Layer?** Should we just serve JSON-RPC, or also feed Portal Network, export Era files, and serve light clients?

2. **Multi-chain timing.** EIP-4444 discussions mention L2s too. OP Stack and other L2s face the same history problem. When do we expand beyond Ethereum L1?

3. **Witness storage.** When blocks include execution witnesses, do we store them (valuable for proof-of-inclusion use cases) or strip them (save storage)?

4. **Revenue model.** If we become critical infrastructure, how does this sustain itself? Indexer SLAs? Portal Network incentives? Grant funding?

## Sources

- [The Ethereum Stateless Book](https://stateless.fyi/) ([repo](https://github.com/stateless-consensus/eth-stateless-book))
- Stateless Implementers Call (SIC) notes through early 2025
- EIP-4444, EIP-4762, EIP-7612, EIP-7748, EIP-7864, EIP-2935
- Detailed Gemini analysis: `docs/spec/stateless-book-analysis/gemini-analysis.md`
- Repomix pack: `docs/spec/stateless-book-analysis/repomix-eth-stateless-book.xml`
