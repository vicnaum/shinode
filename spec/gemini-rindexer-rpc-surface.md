# Gemini Notes: rindexer RPC Surface

Captured: 2026-01-18

Source: Gemini analysis of the `joshstevens19/rindexer` codebase (core indexing flow).

## Gemini answer (raw)

```text
Based on the analysis of the `rindexer` codebase (specifically `core/`), here is the breakdown of the Ethereum JSON-RPC method surface required for operation.

### Summary
*   **Indexing Strategy:** `rindexer` is a **polling-based** indexer. It does not use WebSockets (`eth_subscribe`) for live indexing. It relies on a loop that polls `eth_getBlockByNumber` (latest) and `eth_getLogs`.
*   **Minimal Surface:** The core event indexing loop strictly requires only **4 methods**: `eth_chainId`, `eth_blockNumber`, `eth_getLogs`, and `eth_getBlockByNumber`.
*   **Timestamp Handling:** `rindexer` **does not strictly require** the non-standard `blockTimestamp` field in `eth_getLogs` responses. It checks for it; if missing, it falls back to a "BlockClock" mechanism which fetches headers via `eth_getBlockByNumber` to interpolate timestamps.
*   **Reorg Handling:** Reorgs are detected by comparing the `logsBloom` and block numbers from the RPC head against the internal state during the polling loop.
*   **Native Transfers:** If native transfers (ETH) indexing is enabled, it requires full transaction objects in blocks (`eth_getBlockByNumber` with `full=true`), `trace_block`, or `debug_traceBlockByNumber`.

---

### RPC Method Surface Table

| RPC Method | Required? | Purpose | Call Sites | Required Response Fields |
| :--- | :--- | :--- | :--- | :--- |
| **`eth_chainId`** | **Yes** | Validates that the connected RPC node matches the `chain_id` defined in `rindexer.yaml` on startup. | `core/src/provider.rs`<br>`create_client` | `result` (hex string) |
| **`eth_blockNumber`** | **Yes** | Determines the start/end blocks for indexing if not manually specified, and tracks progress. | `core/src/provider.rs`<br>`core/src/indexer/start.rs` | `result` (hex string) |
| **`eth_getLogs`** | **Yes** | The primary mechanism for fetching contract events (historical and live). | `core/src/provider.rs`<br>`core/src/indexer/fetch_logs.rs` | **Log Object:**<br>- `address`<br>- `topics` (array)<br>- `data`<br>- `blockNumber`<br>- `blockHash`<br>- `transactionHash`<br>- `transactionIndex`<br>- `logIndex`<br>- `blockTimestamp` (Optional optimization) |
| **`eth_getBlockByNumber`** | **Yes** | 1. **Live Head:** Polls `latest` to detect new blocks.<br>2. **Bloom Check:** optimization to skip blocks without relevant logs.<br>3. **Timestamps:** Fallback to fetch time if not in logs.<br>4. **Native Transfers:** Fetches full txs if native indexing is on. | `core/src/provider.rs`<br>`core/src/blockclock/fetcher.rs`<br>`core/src/indexer/fetch_logs.rs` | **Block Object:**<br>- `number`<br>- `timestamp`<br>- `hash`<br>- `logsBloom` (Optional if `disable_logs_bloom_checks` is true)<br>- `transactions` (Only if native transfers enabled) |
| **`debug_traceBlockByNumber`** | No | Feature: Native Transfers. Used if configured as the extraction method. | `core/src/provider.rs`<br>`core/src/indexer/native_transfer.rs` | Trace objects compatible with Geth `callTracer` |
| **`trace_block`** | No | Feature: Native Transfers. Used if configured as the extraction method (Parity/OpenEthereum style). | `core/src/provider.rs`<br>`core/src/indexer/native_transfer.rs` | Trace objects (Parity style) |
| **`eth_getTransactionReceipt`** | No | Exposed in provider but not used in the core `fetch_logs` loop. Available for custom Rust handlers. | `core/src/provider.rs` | Receipt fields |
| **`zks_L1ChainId`** | No | Auto-detection: Checks if the chain is a ZK-rollup (specifically ZKsync) to adjust trace logic. | `core/src/provider.rs`<br>`create_client` | `result` (if successful) |

---

### Minimum RPC Spec (v0.1)

To support `rindexer` for standard contract event indexing (no-code or Rust projects), an RPC provider **must** implement:

1.  **`eth_chainId`**
2.  **`eth_blockNumber`**
3.  **`eth_getLogs`**
    *   *Constraint:* Must support range queries. `rindexer` has retry logic for "block range too wide" errors, so it adapts to provider limits, but the method must work.
    *   *Optimization:* Including `blockTimestamp` in the log object avoids additional `eth_getBlockByNumber` calls.
4.  **`eth_getBlockByNumber`**
    *   *Inputs:* Must accept `latest` and specific hex block numbers.
    *   *Output:* Must return `timestamp` and `number`. `logsBloom` is highly recommended (rindexer checks this to avoid querying logs for blocks that don't contain the contract address).

### Important Implementation Details

**1. Live Indexing Mechanism**
`rindexer` uses a polling loop in `core/src/indexer/fetch_logs.rs` (`live_indexing_stream`).
*   It calls `get_latest_block` (which calls `eth_getBlockByNumber("latest")`).
*   It maintains a local LRU cache of block timestamps (`block_times`).
*   It waits `target_iteration_duration` (default 200ms) or uses the `block_poll_frequency` defined in YAML.
*   **Confirmation:** It does **not** use `eth_subscribe` or WebSockets.

**2. Timestamp Fallback Strategy**
Located in `core/src/blockclock/mod.rs` (`attach_log_timestamps`):
1.  **Check Logs:** Checks if `log.block_timestamp` exists. If yes, returns.
2.  **Fixed Interval:** If the chain is known to have fixed spacing (e.g., Base, Optimism), it calculates the timestamp locally.
3.  **RPC Fetch:** If neither above works, it calls `eth_getBlockByNumber` (batched) to fetch headers for the specific blocks mentioned in the logs.

**3. Bloom Filters**
Located in `core/src/helpers/evm_log.rs` (`is_relevant_block`):
*   `rindexer` fetches the block header first.
*   It checks the `logsBloom` against the contract address and topic signature.
*   If the bloom check fails, it skips `eth_getLogs` for that block entirely to save RPC credits.
*   *Note:* This can be disabled via `disable_logs_bloom_checks: true` in `rindexer.yaml`.

### Optional Features to Ignore

For a minimal implementation of an RPC for `rindexer`, you can ignore:
*   `eth_subscribe` / WebSockets.
*   `debug_trace*` and `trace_*` methods (unless supporting Native Transfers feature).
*   `eth_getTransactionReceipt` (unless custom user code requests it).
*   `zks_*` methods (unless supporting ZKsync specifically).
```

