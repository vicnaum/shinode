# Q009 - Static Files + NippyJar Architecture and Data Split

Status: verified

## Question
How do static files and NippyJar work in Reth, and how is data split between MDBX and static files?

## Context Packs
- `spec/reth_repo_context/context-storage.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-storage.xml

Question:
How do static files and NippyJar work in Reth, and how is data split between MDBX and static files?
I want the architecture, file layout, read/write flow, and how providers decide which backend to use.

Requirements:
1. Cite file paths and type names for every major claim.
2. Identify entry points for static file initialization and read/write paths.
3. Explain how NippyJar segments are named and how indices/offsets/configs are used.
4. Describe how provider code routes reads/writes between MDBX and static files.
5. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Static files lifecycle flow (step-by-step)
4. Data split rules (what goes to MDBX vs static files)
5. Key abstractions/types (with file paths)
6. “Where to look next”
7. Suggested follow-up questions
```

## Gemini Answer
Based on the provided XML pack, here is a detailed architectural map of how Reth uses Static Files (`NippyJar`) alongside MDBX.

### 1. Summary
*   **Hybrid Storage Model**: Reth uses MDBX for "hot" random-access data (state, trie, tip of chain) and Static Files for "cold" immutable data (headers, transactions, receipts).
*   **NippyJar Format**: Custom immutable file format consisting of three parts: Data (`.dat`), Offsets (`.off`), and Configuration (`.conf`). It supports column-based compression (Zstd/LZ4).
*   **Segmented Data**: Data is split into `StaticFileSegment`s (e.g., `Headers`, `Transactions`, `Receipts`), stored in fixed-size block ranges (default 500k blocks per file).
*   **Dual-Threaded Writing**: When saving blocks, `DatabaseProvider` spawns parallel threads: one to write immutable data to Static Files and one to write state/indices to MDBX.
*   **Abstracted Access**: The `EitherReader` and `EitherWriter` enums dynamically route I/O to either MDBX or Static Files based on `StorageSettings` and data availability.
*   **Memory-Mapped Reads**: Reading from static files relies on memory-mapped files (`mmap`) managed by the `DataReader` struct for high-performance zero-copy access.

---

### 2. Entry Points

| File Path | Type | Purpose |
| :--- | :--- | :--- |
| `crates/storage/provider/src/providers/static_file/manager.rs` | `StaticFileProvider` | The central manager that holds indices of all available static files and routes queries to the correct file segment. |
| `crates/storage/nippy-jar/src/lib.rs` | `NippyJar` | The core struct representing the file format, handling serialization of configuration and file path management. |
| `crates/storage/provider/src/either_writer.rs` | `EitherWriter` | The routing logic that decides whether to write data (like Receipts) to MDBX cursors or Static File writers. |
| `crates/storage/provider/src/providers/database/provider.rs` | `DatabaseProvider::save_blocks` | The high-level function that orchestrates writing a block, splitting work between MDBX and Static Files. |
| `crates/static-file/static-file/src/static_file_producer.rs` | `StaticFileProducer` | A background process (not fully detailed in this pack, but referenced) that moves data from MDBX to static files for "legacy" node configurations. |

---

### 3. Static Files Lifecycle & Architecture

#### A. File Layout & Naming
Static files are stored in the `static_files/` directory. A single segment range produces three files.
*   **Naming Convention**: Defined in `crates/static-file/types/src/segment.rs`.
    *   Format: `static_file_{segment}_{start}_{end}`
    *   Example: `static_file_headers_0_499999`
*   **Extensions** (`crates/storage/nippy-jar/src/lib.rs`):
    1.  `.dat`: The compressed raw data (columnar).
    2.  `.off`: Offsets (64-bit integers pointing to row starts in the `.dat` file).
    3.  `.conf`: Bincode-serialized `NippyJar` struct (metadata, compression dictionaries, version).

#### B. The NippyJar Format
*   **Structure**: A `NippyJar` holds a `user_header` (metadata like `SegmentHeader`) and supports multiple columns.
*   **Writing**: Handled by `NippyJarWriter` (`crates/storage/nippy-jar/src/writer.rs`).
    *   It appends data to the `.dat` file.
    *   It keeps offsets in memory and flushes them to `.off` only on `commit()`.
    *   It ensures consistency; if a write is interrupted, it can heal (truncate) the files on the next open using `NippyJarChecker`.
*   **Reading**: Handled by `NippyJarCursor` (`crates/storage/nippy-jar/src/cursor.rs`).
    *   Uses `DataReader` to `mmap` the `.dat` and `.off` files.
    *   Reads the offset, slices the byte range from the mmap, and decompresses it using `Zstd` or `Lz4`.

#### C. Lifecycle Flow (Write -> Read)
1.  **Creation**: `StaticFileProvider::get_writer` creates a `StaticFileProviderRW`.
2.  **Append**: Data is appended (e.g., `append_header`) via `NippyJarWriter`.
3.  **Commit**: calling `commit()` flushes offsets to disk and updates the `indexes` in `StaticFileProviderInner` to make the new data visible.
4.  **Read Query**: A read request (e.g., `header_by_number`) hits `StaticFileProvider`.
5.  **Index Lookup**: `find_fixed_range` calculates which file holds the block.
6.  **Cursor Access**: A `StaticFileJarProvider` creates a cursor over the specific `.dat` file and retrieves the row.

---

### 4. Data Split & Routing (MDBX vs. Static Files)

Reth decides where to put data based on **Storage Settings** and the **Segment** type.

#### The `EitherWriter` Pattern
Located in `crates/storage/provider/src/either_writer.rs`, this enum abstracts the destination.

```rust
pub enum EitherWriter<'a, CURSOR, N> {
    Database(CURSOR),
    StaticFile(StaticFileProviderRWRefMut<'a, N>),
    // ...
}
```

*   **Receipts**: `EitherWriter::new_receipts` checks `StorageSettings`.
    *   If `receipts_in_static_files` is true: Returns `EitherWriter::StaticFile`.
    *   Otherwise: Returns `EitherWriter::Database` (MDBX).
*   **Senders**: Similar logic via `EitherWriter::new_senders`.
*   **Headers & Transactions**: These are almost exclusively written to Static Files in modern Reth configurations (Edge), bypassing MDBX storage logic for bodies entirely in `DatabaseProvider::save_blocks`.

#### The `save_blocks` Flow
Located in `crates/storage/provider/src/providers/database/provider.rs`.
When a block is inserted:
1.  **Parallel Execution**: `thread::scope` is used to spawn threads.
    *   **Thread 1 (Static Files)**: Writes Headers, Transactions, and (optionally) Receipts/Senders to static files.
    *   **Thread 2 (MDBX)**: Writes Block Indices (`HeaderNumbers`, `BlockBodyIndices`) and State (Trie updates, History) to MDBX.
2.  **Commit**: Both providers are committed. MDBX commit creates the transaction boundary; Static File commit flushes offsets.

---

### 5. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `StaticFileSegment` | `crates/static-file/types/src/segment.rs` | Enum identifying data types (Headers, Transactions, Receipts, AccountChangeSets). |
| `SegmentHeader` | `crates/static-file/types/src/segment.rs` | Metadata stored in the `.conf` file, tracking block ranges and transaction ranges included in the jar. |
| `StaticFileProvider` | `crates/storage/provider/src/providers/static_file/manager.rs` | Thread-safe manager that maintains indices (`blocks_per_file`, `map` of loaded jars) and allows searching for data across files. |
| `NippyJar` | `crates/storage/nippy-jar/src/lib.rs` | The struct representing the file on disk. Handles loading/saving the `.conf` file. |
| `NippyJarCursor` | `crates/storage/nippy-jar/src/cursor.rs` | Reads data. Supports `row_by_number_with_cols` to read specific columns (e.g., just the Header, avoiding the Hash). |
| `StorageSettings` | `crates/storage/db-api/src/models/metadata.rs` | Config struct determining if Receipts/Senders go to MDBX or Static Files. |

---

### 6. "Where to look next"

1.  **`crates/storage/provider/src/providers/static_file/manager.rs`**: Look at `get_segment_provider_for_block`. This is the core routing logic that translates a block number into a specific file handle.
2.  **`crates/storage/nippy-jar/src/compression/zstd.rs`**: To understand how Reth uses Zstd dictionaries. Static files train a dictionary based on the first N rows to compress subsequent rows efficiently.
3.  **`crates/storage/provider/src/providers/database/provider.rs`**: Specifically the `save_blocks` method. This is the critical junction where data forks between the "Hot" path (MDBX) and "Cold" path (Static Files).

---

### 7. Suggested Follow-up Questions

*   "How does `NippyJarWriter` handle crash recovery if the node shuts down while appending to the `.dat` file but before writing the `.off` file?"
*   "How does `StaticFileProvider` handle overlapping block ranges if a static file was generated for blocks 0-1000 but MDBX still has blocks 900-1000?"
*   "How are Zstd dictionaries trained and stored within the `NippyJar` `.conf` file?"
*   "What is the specific role of `SegmentRangeInclusive` in mapping transaction numbers to block numbers within the `StaticFileProvider` indices?"

## Verification
- Static file segments and naming: `StaticFileSegment::filename` uses `static_file_{segment}_{start}_{end}`; default blocks per file is 500,000.```106:112:/Users/vicnaum/github/stateless-history-node/reth/crates/static-file/types/src/segment.rs
pub fn filename(&self, block_range: &SegmentRangeInclusive) -> String {
    format!("static_file_{}_{}_{}", self.as_str(), block_range.start(), block_range.end())
}
```
```27:28:/Users/vicnaum/github/stateless-history-node/reth/crates/static-file/types/src/lib.rs
pub const DEFAULT_BLOCKS_PER_STATIC_FILE: u64 = 500_000;
```
- `NippyJar` supports column-based compression and reads via `mmap`; `DataReader` maps data and offsets files.```82:86:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/nippy-jar/src/lib.rs
/// Data is organized into a columnar format, enabling column-based compression. Data retrieval
/// entails consulting an offset list and fetching the data from file via `mmap`.
```
```332:355:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/nippy-jar/src/lib.rs
pub struct DataReader {
    data_mmap: Mmap,
    offset_mmap: Mmap,
}
...
let data_mmap = unsafe { Mmap::map(&data_file)? };
let offset_mmap = unsafe { Mmap::map(&offset_file)? };
```
- NippyJar file layout uses base data file + `.idx`, `.off`, `.conf` sidecars (no `.dat` extension).```223:241:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/nippy-jar/src/lib.rs
pub fn data_path(&self) -> &Path { self.path.as_ref() }
pub fn index_path(&self) -> PathBuf { self.path.with_extension(INDEX_FILE_EXTENSION) }
pub fn offsets_path(&self) -> PathBuf { self.path.with_extension(OFFSETS_FILE_EXTENSION) }
pub fn config_path(&self) -> PathBuf { self.path.with_extension(CONFIG_FILE_EXTENSION) }
```
- `NippyJarWriter` keeps offsets in memory and flushes/commits them on `commit()`.```348:364:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/nippy-jar/src/writer.rs
/// Commits configuration and offsets to disk. It drains the internal offset list.
pub fn commit(&mut self) -> Result<(), NippyJarError> {
    self.sync_all()?;
    self.finalize()?;
    Ok(())
}
```
- `StaticFileProvider` manages static files and supports per‑segment `blocks_per_file`.```103:156:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/static_file/manager.rs
/// [`StaticFileProvider`] manages all existing [`StaticFileJarProvider`].
...
/// Each static file segment is stored across multiple files, and each of these files contains
/// up to the specified number of blocks of data.
```
- Static file writes are parallel per segment (headers, txs, senders, receipts, account changesets).```646:699:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/static_file/manager.rs
/// Writes all static file data for multiple blocks in parallel per-segment.
...
let h_headers = self.spawn_segment_writer(... StaticFileSegment::Headers, ...);
let h_txs = self.spawn_segment_writer(... StaticFileSegment::Transactions, ...);
let h_senders = ctx.write_senders.then(|| { ... StaticFileSegment::TransactionSenders ... });
let h_receipts = ctx.write_receipts.then(|| { ... StaticFileSegment::Receipts ... });
let h_account_changesets = ctx.write_account_changesets.then(|| { ... StaticFileSegment::AccountChangeSets ... });
```
- `DatabaseProvider::save_blocks` parallelizes static file writes with MDBX writes.```416:490:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/provider.rs
/// This method parallelizes static file (SF) writes with MDBX writes.
...
thread::scope(|s| {
    let sf_handle = s.spawn(|| { sf_provider.write_blocks_data(&blocks, &tx_nums, sf_ctx)?; ... });
    ...
    // MDBX writes happen on the main thread
})
```
- `EitherWriter` routes receipts/senders/account changesets based on `StorageSettings` and prune modes.```109:167:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/either_writer.rs
pub fn new_receipts<P>(...) -> ProviderResult<...> {
    if Self::receipts_destination(provider).is_static_file() { ... } else { ... }
}
pub fn new_senders<P>(...) -> ProviderResult<...> { ... }
pub fn new_account_changesets<P>(...) -> ProviderResult<...> { ... }
```
- `StorageSettings` defines flags for receipts/senders/account changesets in static files.```13:33:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/models/metadata.rs
pub struct StorageSettings {
    pub receipts_in_static_files: bool,
    pub transaction_senders_in_static_files: bool,
    pub account_changesets_in_static_files: bool,
    ...
}
```
- `StaticFileProducer` exists and copies data from DB to static files in parallel; it does not delete DB data.```108:116:/Users/vicnaum/github/stateless-history-node/reth/crates/static-file/static-file/src/static_file_producer.rs
/// For each target ... runs it ... All segments are run in parallel.
///
/// NOTE: it doesn't delete the data from database, and the actual deleting (aka pruning) logic
/// lives in the `prune` crate.
```

## Corrections / Caveats
- NippyJar data files do not use a `.dat` extension; the data file is the base path, with `.idx`, `.off`, `.conf` sidecars.
- The `static_files/` directory path is not hardcoded here; it depends on the configured `StaticFileProvider` path.
- “EitherReader” routing is not shown in this pack; `EitherWriter` is the proven routing logic.
- `StaticFileProducer` is a utility for copying DB data to static files; it is not explicitly described as “legacy-only”.

## Actionable Pointers
- Static file routing + indexing: `reth/crates/storage/provider/src/providers/static_file/manager.rs`.
- Static file segments and naming: `reth/crates/static-file/types/src/segment.rs` and `reth/crates/static-file/types/src/lib.rs`.
- NippyJar format + mmap readers: `reth/crates/storage/nippy-jar/src/lib.rs`, `.../writer.rs`, `.../cursor.rs`.
- Storage settings routing: `reth/crates/storage/provider/src/either_writer.rs`, `reth/crates/storage/db-api/src/models/metadata.rs`.
- Save blocks split: `reth/crates/storage/provider/src/providers/database/provider.rs` (`save_blocks`).
