# Gemini Notes: Ponder RPC Surface

Captured: 2026-01-18

Source: Gemini analysis of the `ponder-sh/ponder` codebase (pack: `spec/repomix-output.ponder.filtered.xml`).

Important: Ponder baseline requires **`eth_call`** (stateful), so it is **not supported in v0.1** of our stateless node. This is kept as reference for later.

## Gemini answer (raw)

```text
Based on an analysis of the provided Ponder codebase (v0.16.1), here is the detailed breakdown of Ethereum JSON-RPC usage.

### Summary
*   **Core Reliance:** Ponder relies heavily on `viem` for RPC interaction but implements its own caching and request orchestration layers.
*   **Baseline Requirements:** A "normal" Ponder app (indexing log events) only strictly requires **`eth_chainId`**, **`eth_getLogs`**, **`eth_getBlockByNumber`**, **`eth_getBlockByHash`**, and **`eth_call`**.
*   **Live Indexing:** Ponder prefers **`eth_subscribe`** (WebSockets) for realtime updates but automatically falls back to polling **`eth_getBlockByNumber`** if no WebSocket URL is provided or if the subscription fails.
*   **Reorg Handling:** Ponder requires **`eth_getBlockByHash`** to traverse the `parentHash` chain when reconciling reorgs in the realtime sync service.
*   **Receipts & Traces:** Methods like `eth_getBlockReceipts` and `debug_traceBlockBy*` are only invoked if specific configuration options (`includeTransactionReceipts`, `includeCallTraces`) or filter types (`trace`, `transfer`) are used.

---

### RPC Methods Analysis

| Method | Status | Purpose | Call Sites (File Paths) | Minimum Response Fields Read |
| :--- | :--- | :--- | :--- | :--- |
| **`eth_chainId`** | **REQUIRED** | Validates config vs RPC response at startup. | `core/src/build/index.ts` (rpcDiagnostic) | Result (Hex) |
| **`eth_getLogs`** | **REQUIRED** | Fetches historical event logs for indexing. | `core/src/rpc/actions.ts`<br>`core/src/sync-historical/index.ts`<br>`core/src/sync-realtime/index.ts` | `blockNumber`, `logIndex`, `blockHash`, `address`, `topics`, `data`, `transactionHash`, `transactionIndex` |
| **`eth_getBlockByNumber`** | **REQUIRED** | Fetches blocks for historical backfill, realtime polling, and finality checks. | `core/src/rpc/actions.ts`<br>`core/src/sync-historical/index.ts`<br>`core/src/sync-realtime/index.ts`<br>`core/src/runtime/index.ts` | `hash`, `number`, `timestamp`, `logsBloom`, `parentHash`, `transactions` (array of objects)* |
| **`eth_getBlockByHash`** | **REQUIRED** | Fetches blocks during realtime reorg reconciliation (walking up `parentHash`). | `core/src/rpc/actions.ts`<br>`core/src/sync-realtime/index.ts` | Same as `eth_getBlockByNumber` |
| **`eth_call`** | **REQUIRED** | Used for factory contract resolution, `multicall3` aggregation, and user-land `readContract`. | `core/src/indexing/client.ts`<br>`core/src/rpc/index.ts` | Result (Hex) |
| **`eth_subscribe`** | Optional | Preferred method for realtime block ingestion ("newHeads"). | `core/src/rpc/index.ts` | `hash`, `number`, `parentHash`, `logsBloom`, `timestamp` |
| **`eth_getBlockReceipts`** | Optional | Fetches receipts in batch. Used if `includeTransactionReceipts` is true. | `core/src/rpc/actions.ts`<br>`core/src/sync-realtime/index.ts` | `transactionHash`, `transactionIndex`, `blockHash`, `blockNumber`, `from`, `to`, `status`, `logsBloom` |
| **`eth_getTransactionReceipt`** | Optional | Fallback for receipts if `eth_getBlockReceipts` fails or is unsupported. | `core/src/rpc/actions.ts`<br>`core/src/sync-realtime/index.ts` | Same as `eth_getBlockReceipts` |
| **`debug_traceBlockByNumber`** | Optional | Fetches traces. Used if `includeCallTraces` is true or `trace`/`transfer` filters used. | `core/src/rpc/actions.ts`<br>`core/src/sync-historical/index.ts` | `txHash`, `result` (frame: `from`, `to`, `input`, `output`, `value`, `gasUsed`, `error`, `calls`) |
| **`debug_traceBlockByHash`** | Optional | Same as above, used in realtime sync logic. | `core/src/rpc/actions.ts`<br>`core/src/sync-realtime/index.ts` | Same as `debug_traceBlockByNumber` |

*\*Note on Blocks: Ponder validates consistency between the Block header and the Transaction objects inside it. It strictly requires transaction objects (not just hashes) in block responses unless specifically configured otherwise, though it handles some `eth_subscribe` payloads that only return headers.*

---

### Minimum RPC Spec for Ponder Baseline Event Indexing

To support a standard Ponder application (indexing contract events, reading contract state, no tracing, no extra receipt data), an RPC provider must support:

**1. General**
- [x] **`eth_chainId`**: Must match the chain ID in `ponder.config.ts`.

**2. Logs**
- [x] **`eth_getLogs`**:
    - Must support `fromBlock` and `toBlock`.
    - Must support `address` (single or array).
    - Must support `topics` filtering.
    - Ponder has built-in retry logic to handle rate limits and block range limits (e.g., splitting 10k block ranges into smaller chunks), so strict range limits are acceptable but must be communicated via standard error messages (e.g., "limit exceeded").

**3. Blocks (Full)**
- [x] **`eth_getBlockByNumber`**:
    - Must return full transaction objects (`hydrated: true`).
    - **Crucial:** The `transactions` array must contain objects with `hash`, `transactionIndex`, `from`, `to`, `input`, `value`, `nonce`.
    - **Crucial:** `logsBloom` is required for Ponder's optimization logic (it skips `getLogs` if the bloom filter doesn't match).
    - `timestamp`, `number`, `hash`, `parentHash` are strict requirements.
- [x] **`eth_getBlockByHash`**:
    - Used during reorgs to walk the chain. Same response requirements as `getByNumber`.

**4. Contract Interaction**
- [x] **`eth_call`**:
    - Ponder heavily relies on `multicall3`. The RPC must support `eth_call` to the Multicall3 contract address for the specific chain.
    - Used for resolving Factory child contracts and user-land `context.client.readContract` calls.

---

### Optional Features & RPC Implications

If you are building a specialized RPC for Ponder, you can omit these methods if you accept reduced functionality:

1.  **WebSockets (`eth_subscribe`)**:
    - Impact: Live indexing will use polling (`eth_getBlockByNumber` every ~1s) instead of pushing.

2.  **`eth_getBlockReceipts` / `eth_getTransactionReceipt`**:
    - Impact: Users cannot use `includeTransactionReceipts: true`.

3.  **`debug_traceBlockBy*`**:
    - Impact: Users cannot use `includeCallTraces: true` or index internal transfers/calls.

4.  **`eth_getStorageAt`, `eth_getBalance`, `eth_getCode`**:
    - Impact: Minimal. Exposed to user via `context.client` but not required by core indexing engine.
```

