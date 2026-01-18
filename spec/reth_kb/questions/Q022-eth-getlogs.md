# Q022 - eth_getLogs

Status: verified

## Question
How is eth_getLogs implemented?

## Context Packs
- `spec/reth_repo_context/context-rpc.xml`
- (optional) `spec/reth_repo_context/context-storage.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-rpc.xml

Question (Q022):
How is eth_getLogs implemented?
I want the query pipeline, filtering limits, and how storage access is performed (DB vs static files).

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Trace the main call path from RPC handler to provider/storage.
3. Describe limits and safeguards (max blocks, max logs, pagination).
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Algorithm flow (step-by-step)
4. Key abstractions/types (with file paths)
5. "Where to look next"
6. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-rpc.xml` pack, here is the architecture of the `eth_getLogs` implementation in Reth.

### 1. Summary
*   **Two-Stage Filtering**: The implementation first filters blocks using **Bloom Filters** (in memory or via header access) to identify candidate blocks, then fetches **Receipts** only for those blocks to perform exact log matching.
*   **Hybrid Execution Mode**: To optimize performance, the filter logic switches between a **Cached Mode** (for recent blocks likely in memory/LRU cache) and a **Range Mode** (for historical blocks requiring disk I/O).
*   **Parallel Receipt Fetching**: When processing large historical ranges, the system spawns parallel blocking tasks to fetch receipts for multiple candidate blocks concurrently.
*   **Strict Resource Limits**: Queries are bounded by `max_blocks_per_filter` (range size) and `max_logs_per_response` (result size) to prevent DoS attacks.
*   **Storage Agnostic**: The RPC layer calls `provider.receipts_by_block()`. Whether this reads from MDBX or Static Files (flat files) is determined by the Provider implementation (not in this pack), making the RPC layer backend-agnostic.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/rpc/rpc/src/eth/filter.rs` | `EthFilter::logs` | The public RPC handler for `eth_getLogs`. Validates input and delegates to internal logic. |
| `crates/rpc/rpc/src/eth/filter.rs` | `get_logs_in_block_range` | The coordination function that checks block limits and spawns the blocking task for processing. |
| `crates/rpc/rpc/src/eth/filter.rs` | `get_logs_in_block_range_inner` | The core loop that iterates headers, checks blooms, and fetches receipts. |
| `crates/rpc/rpc-eth-types/src/logs_utils.rs` | `append_matching_block_logs` | The matcher logic that compares a specific log against the filter topics/addresses. |

---

### 3. Algorithm Flow

#### Step 1: Input Resolution & Validation
When `eth_getLogs` is called, `logs_for_filter` resolves the block range.
*   **Logic**: It converts `fromBlock`/`toBlock` tags (e.g., "latest", "safe") to block numbers.
*   **Limit Check**: It checks if `to_block - from_block` exceeds `max_blocks_per_filter`.
*   **File**: `crates/rpc/rpc/src/eth/filter.rs` -> `logs_for_filter`.

#### Step 2: Bloom Filter Scanning (`get_logs_in_block_range_inner`)
The request is spawned onto a `blocking_task` (to avoid stalling the async runtime).
1.  **Iterate Headers**: It iterates over headers in the range in chunks (`MAX_HEADERS_RANGE = 1000`).
2.  **Bloom Check**: For each header, it calls `filter.matches_bloom(header.logs_bloom())`.
3.  **Candidate Collection**: If the bloom matches, the header is added to `matching_headers`.
*   **Optimization**: Non-matching headers are skipped entirely, saving DB lookups for receipts.

#### Step 3: Receipt Fetching (Cached vs. Range Mode)
The system initializes a `RangeMode` to process the matching headers.
*   **Cached Mode**: If the range is small and close to the tip, it uses `CachedMode`. It queries `eth_cache.get_receipts_and_maybe_block`, hitting the LRU cache first.
*   **Range Mode**: For older/larger ranges, it uses `RangeBlockMode`.
    *   **Parallelism**: If the number of matching headers exceeds `PARALLEL_PROCESSING_THRESHOLD` (1000), it chunks them and spawns parallel `tokio::task::spawn_blocking` jobs to fetch receipts from the provider.
*   **File**: `crates/rpc/rpc/src/eth/filter.rs` -> `RangeBlockMode` struct.

#### Step 4: Log Matching
Once receipts are loaded, `append_matching_block_logs` is called for every receipt.
*   It iterates `receipt.logs()`.
*   It matches the log address and topics against the user's filter.
*   Matches are appended to `all_logs`.
*   **Safety**: If `all_logs.len()` exceeds `max_logs_per_response`, the query aborts early with an error.
*   **File**: `crates/rpc/rpc-eth-types/src/logs_utils.rs`.

---

### 4. Limits and Safeguards

Defined in `crates/rpc/rpc-eth-types/src/builder/config.rs` and enforced in `crates/rpc/rpc/src/eth/filter.rs`.

1.  **Block Range Limit**:
    *   **Config**: `max_blocks_per_filter` (default `100,000`).
    *   **Check**: In `get_logs_in_block_range`. Returns `EthFilterError::QueryExceedsMaxBlocks`.
2.  **Response Size Limit**:
    *   **Config**: `max_logs_per_response` (default `20,000` logs).
    *   **Check**: In `get_logs_in_block_range_inner`. Returns `EthFilterError::QueryExceedsMaxResults`.
3.  **Parallel Concurrency**:
    *   **Constant**: `DEFAULT_PARALLEL_CONCURRENCY` (4 tasks) used in `RangeBlockMode` to prevent one query from saturating all I/O threads.

---

### 5. Storage Access (DB vs. Static Files)

*   **Abstraction**: The RPC layer does **not** know about Static Files or MDBX directly. It interacts purely via the `Provider` trait.
*   **Call Site**: `self.filter_inner.provider().receipts_by_block(header.hash().into())?` inside `RangeBlockMode`.
*   **Implication**: The logic determining whether to read from the "Hot" database (MDBX) or "Cold" history (Static Files/NippyJar) resides in the `reth-provider` crate (not in this pack), specifically in `DatabaseProvider::receipts_by_block`.

---

### 6. Where to Look Next
1.  **`crates/rpc/rpc/src/eth/filter.rs`**: Read `get_logs_in_block_range_inner` to see the exact loop structure and how `RangeMode` handles the complexity of switching strategies.
2.  **`crates/rpc/rpc-eth-types/src/cache/mod.rs`**: To understand how `EthStateCache` serves recent receipts to `CachedMode` to avoid disk I/O.
3.  **`crates/rpc/rpc-eth-types/src/logs_utils.rs`**: For the low-level matching logic (`matching_block_logs_with_tx_hashes`).

### 7. Suggested Follow-up Questions
*   "How does the `EthStateCache` decide what to evict, and does `eth_getLogs` cause cache churn for other RPC methods?"
*   "Where is the implementation of `receipts_by_block` that routes between MDBX and Static Files?" (Requires access to `reth-provider` pack).
*   "How does `eth_getLogs` handle reorgs if a query is running while the chain updates?"

## Verification
- `eth_getLogs` handler forwards to `logs_for_filter`, which delegates to `EthFilterInner::logs_for_filter`.```409:412:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
async fn logs(&self, filter: Filter) -> RpcResult<Vec<Log>> {
    Ok(self.logs_for_filter(filter, self.inner.query_limits).await?)
}
```
```462:569:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
async fn logs_for_filter(self: Arc<Self>, filter: Filter, limits: QueryLimits) -> Result<Vec<Log>, EthFilterError> { ... }
```
- Range queries resolve block tags, reject ranges past head, and call `get_logs_in_block_range`.```540:568:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
let from = from_block.map(|num| self.provider().convert_block_number(num)).transpose()?.flatten();
let to = to_block.map(|num| self.provider().convert_block_number(num)).transpose()?.flatten();
...
if let Some(t) = to && t > info.best_number { return Err(EthFilterError::BlockRangeExceedsHead); }
...
let (from_block_number, to_block_number) = logs_utils::get_filter_block_range(from, to, start_block, info)?;
self.get_logs_in_block_range(filter, from_block_number, to_block_number, limits).await
```
- `get_logs_in_block_range` checks range limits and spawns a blocking task for the inner loop.```603:631:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
if let Some(max_blocks_per_filter) = limits.max_blocks_per_filter.filter(|limit| to_block - from_block > *limit) {
    return Err(EthFilterError::QueryExceedsMaxBlocks(max_blocks_per_filter))
}
...
self.task_spawner.spawn_blocking(Box::pin(async move {
    let res = this.get_logs_in_block_range_inner(&filter, from_block, to_block, limits).await;
    let _ = tx.send(res);
}));
```
- Header bloom filtering happens in `get_logs_in_block_range_inner` before receipt fetching.```655:682:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
let headers = self.provider().headers_range(from..=to)?;
...
if !filter.matches_bloom(header.logs_bloom()) { continue; }
matching_headers.push(SealedHeader::new(header, block_hash));
```
- Cached vs range mode is chosen by `RangeMode::new` and `should_use_cached_mode`.```1002:1022:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
if use_cached_mode && !sealed_headers.is_empty() {
    Self::Cached(CachedMode { filter_inner, headers_iter: sealed_headers.into_iter() })
} else {
    Self::Range(RangeBlockMode { ... })
}
```
```1031:1044:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
const fn should_use_cached_mode(...) -> bool { ... block_count <= adjusted_threshold && distance_from_tip <= adjusted_threshold }
```
- Range mode uses cached receipts when available and falls back to provider receipts.```1201:1217:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
let (maybe_block, maybe_receipts) = self.filter_inner.eth_cache().maybe_cached_block_and_receipts(header.hash()).await?;
...
match self.filter_inner.provider().receipts_by_block(header.hash().into())? { Some(receipts) => Arc::new(receipts), None => continue }
```
- Parallel receipt fetching uses `tokio::task::spawn_blocking` for large remaining header sets.```1179:1289:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
if remaining_headers >= PARALLEL_PROCESSING_THRESHOLD { self.spawn_parallel_tasks(range_headers); }
...
let chunk_task = tokio::task::spawn_blocking(move || { ... provider().receipts_by_block(...) ... });
```
- Matching logs are appended using `append_matching_block_logs`, which checks topics/addresses and fetches tx hashes when needed.```70:146:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-types/src/logs_utils.rs
pub fn append_matching_block_logs<P>(...){
    for (receipt_idx, receipt) in receipts.iter().enumerate() {
        for log in receipt.logs() {
            if filter.matches(log) { ... provider.transaction_by_id(...) ... }
        }
    }
}
```
- Response size limit is enforced for multi-block ranges only.```712:731:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc/src/eth/filter.rs
let is_multi_block_range = from_block != to_block;
if let Some(max_logs_per_response) = limits.max_logs_per_response && is_multi_block_range && all_logs.len() > max_logs_per_response { ... }
```
- Default limits come from `EthConfig`, which uses `DEFAULT_MAX_BLOCKS_PER_FILTER` and `DEFAULT_MAX_LOGS_PER_RESPONSE`.```122:133:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-eth-types/src/builder/config.rs
max_blocks_per_filter: DEFAULT_MAX_BLOCKS_PER_FILTER,
max_logs_per_response: DEFAULT_MAX_LOGS_PER_RESPONSE,
```
```12:16:/Users/vicnaum/github/stateless-history-node/reth/crates/rpc/rpc-server-types/src/constants.rs
pub const DEFAULT_MAX_BLOCKS_PER_FILTER: u64 = 100_000;
pub const DEFAULT_MAX_LOGS_PER_RESPONSE: usize = 20_000;
```

## Corrections / Caveats
- `max_blocks_per_filter` is checked in `get_logs_in_block_range`, not in `logs_for_filter`.
- The bloom filter stage is skipped for `FilterBlockOption::AtBlockHash` and for the pending-block shortcut; those paths go straight to receipts.
- `max_logs_per_response` is only enforced for multi-block ranges, so single-block queries can exceed it without error.
- Range mode still consults the cache (`maybe_cached_block_and_receipts`) before falling back to provider reads.
- Parallel receipt fetching uses `tokio::task::spawn_blocking` inside `RangeBlockMode`, not the generic task spawner used by `get_logs_in_block_range`.

## Actionable Pointers
- Core handler and range flow: `reth/crates/rpc/rpc/src/eth/filter.rs` (`logs_for_filter`, `get_logs_in_block_range`, `get_logs_in_block_range_inner`).
- Cache integration: `reth/crates/rpc/rpc-eth-types/src/cache/mod.rs` (`get_receipts_and_maybe_block`, `maybe_cached_block_and_receipts`).
- Log matching and tx hash lookup: `reth/crates/rpc/rpc-eth-types/src/logs_utils.rs` (`append_matching_block_logs`).
