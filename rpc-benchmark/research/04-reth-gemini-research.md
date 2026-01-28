Based on the provided codebase, here is a detailed analysis of Reth's RPC and storage caching patterns.

### Summary of Key Findings
1.  **Mmap Reuse**: Reth avoids repeated file system calls by mapping static files (`NippyJar`) once and wrapping the memory-mapped handles in `Arc`. These are stored in a `DashMap` keyed by the file's identifier (block range end), preventing re-opening files on every read.
2.  **Two-Layer Caching Strategy**:
    *   **Storage Layer (`StaticFileProvider`)**: Caches open file handles/mmaps indefinitely (until invalidated by reconfiguration). It does not cache the *data* content, but rather the *access mechanism* to the data.
    *   **RPC Layer (`EthStateCache`)**: An LRU cache that stores deserialized/processed objects (Blocks, Receipts) to serve high-frequency RPC requests.
3.  **Thundering Herd Protection**: The `EthStateCache` implements a "MultiConsumer" pattern. If multiple RPCs request the same missing block simultaneously, only one DB fetch is performed, and the result is broadcast to all waiting listeners.
4.  **Smart Filtering**: `eth_getLogs` switches strategies based on the query size. Recent/small queries hit the LRU cache. Large/historical queries bypass the cache (to avoid evicting hot items) and use parallelized, bloom-filtered fetching directly from the `StaticFileProvider`.
5.  **Per-Column Compression**: The low-level `NippyJar` format uses Zstd with per-column dictionaries, and cursors maintain an internal reuse buffer to minimize allocation during decompression.

---

### Key Files & Modules

| Module | File Path | Responsibility |
| :--- | :--- | :--- |
| **Manager** | `crates/storage/provider/src/providers/static_file/manager.rs` | Manages the lifecycle of static files and the `DashMap` cache of open handles. |
| **Jar Provider** | `crates/storage/provider/src/providers/static_file/jar.rs` | The specific provider instance for a single open static file (wraps `LoadedJar`). |
| **RPC Cache** | `crates/rpc/rpc-eth-types/src/cache/mod.rs` | The `EthStateCache` service handling LRU logic for blocks and receipts. |
| **Log Filter** | `crates/rpc/rpc/src/eth/filter.rs` | Logic for `eth_getLogs`, bloom filtering, and processing modes. |
| **NippyJar** | `crates/storage/nippy-jar/src/lib.rs` | The core file format reader utilizing memory-mapped files. |

---

### 1. StaticFileProvider Caching (Storage Layer)

This layer solves your "315 file opens" problem. It caches the *accessors* to the files, not the data itself.

**How segment files are cached:**
Reth uses a `DashMap` to hold open references to static files (wrapped in `LoadedJar`).
*   **File:** `crates/storage/provider/src/providers/static_file/manager.rs`
*   **Code:** `map: DashMap<(BlockNumber, StaticFileSegment), LoadedJar>` (Line 160)
*   **Mechanism:** When data is requested, `get_or_create_jar_provider` (Line 746) checks this map.
    *   **Hit:** Returns a lightweight `StaticFileJarProvider` wrapping the existing `LoadedJar` (Line 757).
    *   **Miss:** Loads the file via `NippyJar::load`, wraps it, inserts it into the map, and returns the provider (Line 761).

**Cache Key Structure:**
The key is a tuple: `(BlockNumber, StaticFileSegment)`.
*   The `BlockNumber` here specifically represents the **end of the block range** for that specific file (Line 746, `fixed_block_range.end()`). This ensures unique identification of the file segment.

**Invalidation & Eviction:**
*   **Invalidation:** Handled via `update_index` (Line 829). If a file is truncated or modified (e.g., during a reorg or append), the old entry is removed or updated.
*   **Eviction:** There is **no automatic LRU eviction** for these file handles in the current code.
    *   *Reference:* Line 743 explicitly states: `// TODO(joshie): we should check the size and pop N if there's too many.`
    *   *Implication:* Since static files are large (covering thousands of blocks) and finite, holding open handles to all of them is generally acceptable for a server node compared to the cost of constantly opening/closing them.

---

### 2. EthStateCache LRU (RPC Layer)

This layer sits above storage to speed up repeated RPC calls (e.g., `eth_getBlockByNumber`).

**How it works:**
It is an async service (`EthStateCacheService`) communicating via channels.
*   **File:** `crates/rpc/rpc-eth-types/src/cache/mod.rs`
*   **Caches:** It maintains three distinct LRU caches:
    1.  `full_block_cache` (Blocks)
    2.  `receipts_cache` (Receipts)
    3.  `headers_cache` (Headers)

**Concurrent Requests (Thundering Herd):**
*   **File:** `crates/rpc/rpc-eth-types/src/cache/multi_consumer.rs`
*   **Mechanism:** The `MultiConsumerLruCache` struct maintains a `queued` map (`HashMap<K, Vec<S>>`).
*   **Logic:**
    1.  When a request comes in (`queue` function, Line 50), it checks if the key is already being fetched.
    2.  If yes, it adds the response sender (`oneshot::Sender`) to the vector in `queued`.
    3.  If no, it returns `true`, triggering the `EthStateCacheService` (Line 245 of `mod.rs`) to spawn a fetch task.
    4.  When the fetch completes, the result is sent to **all** consumers in the queue (Line 207 of `mod.rs`).

**Default Sizes:**
*   **File:** `crates/rpc/rpc-eth-types/src/cache/config.rs`
    *   **Blocks:** 5,000 (`DEFAULT_BLOCK_CACHE_MAX_LEN`)
    *   **Receipts:** 2,000 (`DEFAULT_RECEIPT_CACHE_MAX_LEN`)
    *   **Headers:** 1,000 (`DEFAULT_HEADER_CACHE_MAX_LEN`)

---

### 3. eth_getLogs Optimizations

Reth employs a complex decision tree to optimize log filtering.

**Bloom Filter Pre-filtering:**
*   **File:** `crates/rpc/rpc/src/eth/filter.rs`
*   **Logic:** Before fetching receipts, it iterates headers in the range. It calls `filter.matches_bloom(header.logs_bloom())` (Line 637).
*   **Result:** It builds a list of `matching_headers`. Only blocks with potential matches are queried for receipts.

**RangeMode vs. CachedMode:**
*   **Decision Logic:** `RangeMode::new` (Line 924) determines the strategy.
*   **Heuristic:** `should_use_cached_mode` (Line 950).
    *   It calculates an `adjusted_threshold` based on how many bloom matches occurred.
    *   If `block_count` < Threshold (default 250) AND the data is close to the chain tip, it uses **CachedMode**.
*   **CachedMode:** Uses the `EthStateCache` (LRU) to fetch receipts. This is fast for recent blocks usually in memory.
*   **RangeMode:** Bypasses the LRU cache and fetches directly from `StaticFileProvider` (Line 1109). This prevents massive historical log queries from flushing valuable recent data out of the cache.

**Parallel Receipt Fetching:**
*   **Threshold:** If `remaining_headers >= PARALLEL_PROCESSING_THRESHOLD` (1000) (Line 1070).
*   **Implementation:**
    1.  Chunks the headers (`spawn_parallel_tasks`, Line 1129).
    2.  Spawns `tokio::task::spawn_blocking` for each chunk (Line 1140).
    3.  Fetches receipts directly from the provider in parallel.

---

### 4. NippyJar and Mmap (Low-Level Access)

**Mmap Handles:**
*   **File:** `crates/storage/nippy-jar/src/lib.rs`
*   **Struct:** `DataReader` holds `data_mmap: Mmap` and `offset_mmap: Mmap` (Line 285).
*   **Creation:** `unsafe { Mmap::map(&data_file)? }` (Line 294).
*   **Sharing:** The `StaticFileProvider` wraps `DataReader` in an `Arc`. When a `NippyJarCursor` is created, it clones this `Arc`, incrementing the reference count without duplicating the file descriptor or mapping.

**Cursor Pattern:**
*   **File:** `crates/storage/nippy-jar/src/cursor.rs`
*   **Struct:** `NippyJarCursor` (Line 9).
*   **Buffer Reuse:** It holds `internal_buffer: Vec<u8>` (Line 14).
*   **Flow:**
    1.  `read_value` (Line 101) finds the offset via `self.reader.offset`.
    2.  It slices the mmap (`self.reader.data(range)`).
    3.  If compressed, it decompresses into `internal_buffer` (clearing it first, not reallocating) (Line 124).
    4.  It returns a reference to the data, either pointing to the mmap (if uncompressed) or the internal buffer.

---

### 5. Actionable Recommendations

To solve your "315 file opens" issue and match Reth's performance:

1.  **Implement a Manager with Arc-ed Mmaps:**
    Do not let your query logic open files directly. Create a `StaticFileManager` singleton/actor.
    *   **Structure:** `DashMap<SegmentID, Arc<MmapHandle>>`.
    *   **Behavior:** On query, check the map. If present, clone the `Arc`. If absent, open, map, store in `Arc`, and return.

2.  **Separate Metadata from Data:**
    Reth stores offsets in a separate `.off` file (mapped separately) or header structure. This allows calculating data ranges without touching the main data file.

3.  **Use Per-Cursor Buffers:**
    If you use compression (like Zstd), ensure your cursor/iterator owns a reusable `Vec<u8>` buffer for decompression to avoid `malloc` per block.

4.  **Adopt Multi-Consumer Queues for RPC:**
    If your system serves RPCs, verify you aren't fetching the same "hot" block (e.g., latest) multiple times concurrently. Use the `queued` sender pattern found in `multi_consumer.rs`.

5.  **Differentiate "Hot" vs "Cold" Access:**
    Copy the `eth_getLogs` logic:
    *   **Small/Recent Query:** Go through an LRU Cache.
    *   **Large/Deep Query:** Go straight to the Mmap manager. Do **not** populate the LRU cache with these results to avoid cache pollution.

### Missed/Additional Optimizations

1.  **Zstd Dictionaries:** Reth trains Zstd dictionaries for specific columns (`crates/storage/nippy-jar/src/compression/zstd.rs`). This significantly improves compression ratios and decompression speed for small, repetitive data (like headers or receipts).
2.  **Binary Search on ChangeSets:** In `StaticFileProvider::get_account_before_block` (Line 1315 in `manager.rs`), Reth performs a binary search over the mmapped changeset file to find specific account updates without reading the whole block's history.
3.  **PhantomData for Types:** The `StaticFileProvider` is generic over `NodePrimitives`. This allows the same caching logic to work for different chain specs (e.g., Optimism vs Mainnet) without runtime overhead.