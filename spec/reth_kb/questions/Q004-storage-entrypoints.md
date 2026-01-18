# Q004 - Storage Entry Points (DB, Providers, Static Files)

Status: verified

## Question
Where are the storage entry points (DB open, provider layer, static files), and how do they connect?

## Context Packs
- `spec/reth_repo_context/context-storage.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-storage.xml

Question:
Where are the storage entry points (DB open, provider layer, static files), and how do they connect?
I want a map of: MDBX open/init, db-api traits, provider layer, static files, and how callers use them.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Identify entry points (open env, provider factory, static file builder).
3. Explain read/write transaction flow and where migrations/versioning live.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Storage flow (open -> provider -> read/write -> commit)
4. Key abstractions/types (with file paths)
5. “Where to look next” (top files to read)
6. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-storage.xml` file, here is the architecture map of Reth’s storage layer.

### 1. Summary
*   **Hybrid Backend Strategy**: Reth uses **MDBX** (key-value) for "hot" data like state and trie nodes, and **Static Files** (immutable "Nippy Jars") for "cold" data like historical headers, transactions, and receipts.
*   **The Coordinator**: `ProviderFactory` is the primary entry point that holds handles to MDBX, the Static File Provider, and an auxiliary RocksDB instance.
*   **Trait-Driven API**: The `reth-storage-api` crate defines chain-agnostic traits (e.g., `HeaderProvider`, `StateWriter`) which are implemented by the `DatabaseProvider`.
*   **Transactional Integrity**: All writes occur through `DatabaseProviderRW`, which coordinates a database transaction and multiple static file writers.
*   **Smart Routing**: The `EitherWriter` and `EitherReader` abstractions decide at runtime whether to access data from MDBX or Static Files based on the node's `StorageSettings`.
*   **Immutable Snapshots**: Historical data is stored in segments (e.g., `Headers`, `Receipts`) within `.dat` and `.idx` files, managed by the `NippyJar` format.
*   **Strict Versioning**: Reth tracks a `database.version` file and prevents startup if the version on disk doesn't match the code.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/storage/db/src/mdbx.rs` | `init_db` | The low-level entry point to open MDBX, create tables, and check the version file. |
| `crates/storage/provider/src/providers/database/mod.rs` | `ProviderFactory` | The central struct used by the rest of the node to spawn read or write providers. |
| `crates/storage/provider/src/providers/database/builder.rs` | `ProviderFactoryBuilder` | A high-level builder to configure MDBX, Static Files, and RocksDB paths in one flow. |
| `crates/storage/provider/src/providers/static_file/manager.rs` | `StaticFileProviderBuilder` | Entry point for configuring the immutable data store (block ranges per file, compression). |

---

### 3. Storage Flow

#### Phase 1: Open & Initialization
1.  **Version Check**: `create_db` calls `check_db_version_file` (`crates/storage/db/src/version.rs`). If `database.version` != `DB_VERSION` (currently 2), it fails.
2.  **MDBX Startup**: `init_db` creates the environment and opens the 250+ potential tables defined in `tables!` macro (`crates/storage/db-api/src/tables/mod.rs`).
3.  **Static File Indexing**: `StaticFileProvider::initialize_index` scans the data directory for existing `.dat` files and maps their block/transaction ranges (`crates/storage/provider/src/providers/static_file/manager.rs`).
4.  **Factory Creation**: These are bundled into a `ProviderFactory`.

#### Phase 2: Provider Creation
*   **Read-Only**: `factory.provider()` returns a `DatabaseProviderRO`, which holds a long-lived MDBX read transaction.
*   **Read-Write**: `factory.provider_rw()` returns a `DatabaseProviderRW`, giving the caller access to `DbTxMut` methods.

#### Phase 3: Writing (The Save Loop)
When the node saves a block, `save_blocks` is called (`crates/storage/provider/src/providers/database/provider.rs`):
1.  **Parallel Write**: It spawns a scoped thread to write headers/transactions to **Static Files** while the main thread writes state changes to **MDBX**.
2.  **Routing**: `EitherWriter::new_receipts` checks `StorageSettings`. If `receipts_in_static_files` is true, it writes to a NippyJar; otherwise, it uses an MDBX cursor.
3.  **Consistency**: `insert_block_mdbx_only` ensures that even if data is in static files, the MDBX indices (hash-to-number) are updated.

#### Phase 4: Commit
The `commit()` function in `DatabaseProvider` (`crates/storage/provider/src/providers/database/provider.rs`) performs a "Two-Phase-ish" commit:
1.  **Static Files**: Calls `finalize()` to flush NippyJar buffers and update the file-system headers.
2.  **MDBX**: Calls `tx.commit()` to finalize the key-value changes.

---

### 4. Key Abstractions / Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `Table` | `crates/storage/db-api/src/table.rs` | A trait defining how keys (`Encode/Decode`) and values (`Compress/Decompress`) are handled. |
| `DatabaseProvider` | `crates/storage/provider/src/providers/database/provider.rs` | The implementation core that implements dozens of traits for reading/writing. |
| `NippyJar` | `crates/storage/nippy-jar/src/lib.rs` | The format for immutable columnar storage. Supports LZ4 and Zstd compression. |
| `EitherWriter` | `crates/storage/provider/src/either_writer.rs` | A runtime switch between `Database(CURSOR)` and `StaticFile(WRITER)`. |
| `StorageSettings` | `crates/storage/db-api/src/models/metadata.rs` | Persisted config that tells the node whether to act as a "Legacy" or "Edge" node. |

---

### 5. "Where to Look Next"
1.  **`crates/storage/db-api/src/tables/mod.rs`**: To see the full list of MDBX tables (Schema).
2.  **`crates/storage/provider/src/providers/database/provider.rs`**: This is the most important file for understanding how block execution results are actually persisted.
3.  **`crates/storage/nippy-jar/src/writer.rs`**: To understand the internal disk layout of the static files (offsets vs data).
4.  **`crates/storage/provider/src/providers/state/historical.rs`**: To see how Reth reconstructs the state of an account at an arbitrary block in the past by looking at changesets.

---

### 6. Suggested Follow-up Questions
*   "How does the `StaticFileAccountChangesetWalker` avoid loading an entire segment into memory during a range query?"
*   "What happens during an `unwind_trie_state_from` call if the data has already been moved to static files?"
*   "How does MDBX's `handle_slow_readers` function prevent the database from growing indefinitely due to long read transactions?"
*   "How are the `RocksDB` consistency checks used to heal indices that get out of sync with MDBX checkpoints?"

## Verification
- `create_db` checks/creates `database.version`; `init_db` creates tables via `create_and_track_tables_for`.```10:47:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/mdbx.rs
/// Creates a new database at the specified path if it doesn't exist. Does NOT create tables. Check
/// [`init_db`].
pub fn create_db<P: AsRef<Path>>(path: P, args: DatabaseArguments) -> eyre::Result<DatabaseEnv> {
    use crate::version::{check_db_version_file, create_db_version_file, DatabaseVersionError};
    ...
    Ok(DatabaseEnv::open(rpath, DatabaseEnvKind::RW, args)?)
}
...
pub fn init_db_for<P: AsRef<Path>, TS: TableSet>(
    path: P,
    args: DatabaseArguments,
) -> eyre::Result<DatabaseEnv> {
    let mut db = create_db(path, args)?;
    db.create_and_track_tables_for::<TS>()?;
    db.record_client_version(client_version)?;
    Ok(db)
}
```
```8:12:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/version.rs
pub const DB_VERSION_FILE_NAME: &str = "database.version";
pub const DB_VERSION: u64 = 2;
```
- `ProviderFactory` is the central entry point and holds MDBX, static files, storage settings, and RocksDB.```60:79:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/mod.rs
pub struct ProviderFactory<N: NodeTypesWithDB> {
    /// Database instance
    db: N::DB,
    /// Static File Provider
    static_file_provider: StaticFileProvider<N::Primitives>,
    /// Storage configuration settings for this node
    storage_settings: Arc<RwLock<StorageSettings>>,
    /// `RocksDB` provider
    rocksdb_provider: RocksDBProvider,
    ...
}
```
- `ProviderFactoryBuilder` wires DB + static files + RocksDB (read-only path shown).```99:113:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/builder.rs
    pub fn open_read_only(
        self,
        chainspec: Arc<N::ChainSpec>,
        config: impl Into<ReadOnlyConfig>,
    ) -> eyre::Result<ProviderFactory<NodeTypesWithDBAdapter<N, Arc<DatabaseEnv>>>>
    ...
        self.db(Arc::new(open_db_read_only(db_dir, db_args)?))
            .chainspec(chainspec)
            .static_file(StaticFileProvider::read_only(static_files_dir, watch_static_files)?)
            .rocksdb_provider(RocksDBProvider::builder(&rocksdb_dir).with_default_tables().build()?)
            .build_provider_factory()
```
- `StaticFileProviderBuilder` configures blocks-per-file and access mode.```120:180:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/static_file/manager.rs
pub struct StaticFileProviderBuilder<P> {
    access: StaticFileAccess,
    blocks_per_file: StaticFileMap<u64>,
    path: P,
    ...
}
...
pub fn with_blocks_per_file(mut self, blocks_per_file: u64) -> Self {
    for segment in StaticFileSegment::iter() {
        self.blocks_per_file.insert(segment, blocks_per_file);
    }
    self
}
```
- `EitherWriter::new_receipts` uses cached storage settings and prune modes to choose DB vs static files.```109:195:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/either_writer.rs
pub fn new_receipts<P>(
    provider: &'a P,
    block_number: BlockNumber,
) -> ProviderResult<...> {
    if Self::receipts_destination(provider).is_static_file() {
        Ok(EitherWriter::StaticFile(
            provider.get_static_file_writer(block_number, StaticFileSegment::Receipts)?,
        ))
    } else {
        Ok(EitherWriter::Database(
            provider.tx_ref().cursor_write::<tables::Receipts<ReceiptTy<P::Primitives>>>()?,
        ))
    }
}
...
pub fn receipts_destination<P: DBProvider + StorageSettingsCache>(provider: &P) -> EitherWriterDestination {
    let receipts_in_static_files = provider.cached_storage_settings().receipts_in_static_files;
    let prune_modes = provider.prune_modes_ref();
    ...
}
```
- `save_blocks` explicitly parallelizes static‑file writes and MDBX writes.```416:475:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/provider.rs
/// This method parallelizes static file (SF) writes with MDBX writes.
/// The SF thread writes headers, transactions, senders (if SF), and receipts (if SF, Full mode
/// only). The main thread writes MDBX data (indices, state, trie - Full mode only).
pub fn save_blocks(
    &self,
    blocks: Vec<ExecutedBlock<N::Primitives>>,
    save_mode: SaveBlocksMode,
) -> ProviderResult<()> {
    ...
    thread::scope(|s| {
        // SF writes
        let sf_handle = s.spawn(|| {
            sf_provider.write_blocks_data(&blocks, &tx_nums, sf_ctx)?;
            Ok::<_, ProviderError>(start.elapsed())
        });
        ...
        // MDBX writes
        ...
    })
}
```
- `commit` order is conditional: if unwinding, MDBX commits before static files; otherwise static files finalize first, then MDBX commit.```3454:3494:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/provider.rs
/// Commit database transaction, static files, and pending `RocksDB` batches.
fn commit(self) -> ProviderResult<()> {
    if self.static_file_provider.has_unwind_queued() {
        self.tx.commit()?;
        ...
        self.static_file_provider.commit()?;
    } else {
        // Normal path: finalize() will call sync_all() if not already synced
        self.static_file_provider.finalize()?;
        ...
        self.tx.commit()?;
    }
    Ok(())
}
```
- NippyJar supports Zstd/Lz4 compression and uses `.idx` / `.off` / `.conf` sidecar files; data file uses the base path.```54:61:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/nippy-jar/src/lib.rs
const INDEX_FILE_EXTENSION: &str = "idx";
const OFFSETS_FILE_EXTENSION: &str = "off";
pub const CONFIG_FILE_EXTENSION: &str = "conf";
```
```158:167:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/nippy-jar/src/lib.rs
/// Adds [`compression::Zstd`] compression.
pub fn with_zstd(...)
/// Adds [`compression::Lz4`] compression.
pub fn with_lz4(...)
```
- Static file names are `static_file_{segment}_{start}_{end}` (no `.dat` extension).```106:112:/Users/vicnaum/github/stateless-history-node/reth/crates/static-file/types/src/segment.rs
/// Returns the default file name for the provided segment and range.
pub fn filename(&self, block_range: &SegmentRangeInclusive) -> String {
    format!("static_file_{}_{}_{}", self.as_str(), block_range.start(), block_range.end())
}
```
- `initialize_index` scans files using `iter_static_files`, which loads `NippyJar` headers from those filenames.```21:38:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/static_file/mod.rs
/// Given the `static_files` directory path, it returns a list over the existing `static_files`
pub fn iter_static_files(path: &Path) -> Result<SortedStaticFiles, NippyJarError> {
    ...
    for entry in entries {
        if entry.metadata().is_ok_and(|metadata| metadata.is_file()) &&
            let Some((segment, _)) = StaticFileSegment::parse_filename(...)
        {
            let jar = NippyJar::<SegmentHeader>::load(&entry.path())?;
            ...
        }
    }
}
```
- `StorageSettings` flags are defined in `metadata.rs` (receipts in static files, RocksDB toggles, etc.).```13:34:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/models/metadata.rs
pub struct StorageSettings {
    pub receipts_in_static_files: bool,
    pub transaction_senders_in_static_files: bool,
    pub storages_history_in_rocksdb: bool,
    pub transaction_hash_numbers_in_rocksdb: bool,
    pub account_history_in_rocksdb: bool,
    pub account_changesets_in_static_files: bool,
}
```

## Corrections / Caveats
- Static file data files are named `static_file_{segment}_{start}_{end}` (no `.dat` extension). NippyJar uses `.idx` / `.off` / `.conf` sidecar files. The answer’s “.dat and .idx” claim is not supported by the code.
- `StaticFileProviderBuilder` does not expose compression settings directly; compression is configured per `StaticFileSegment` (default LZ4) via `SegmentConfig`.
- Commit order is conditional: for normal operation static files finalize before MDBX commit, but on unwind MDBX commits before static files.

## Actionable Pointers
- MDBX open/versioning: `reth/crates/storage/db/src/mdbx.rs` and `reth/crates/storage/db/src/version.rs`.
- ProviderFactory + provider RW/RO: `reth/crates/storage/provider/src/providers/database/mod.rs`.
- Save flow + commit ordering: `reth/crates/storage/provider/src/providers/database/provider.rs`.
- Static files: `reth/crates/storage/provider/src/providers/static_file/manager.rs` and `reth/crates/storage/db/src/static_file/mod.rs`.
- NippyJar format: `reth/crates/storage/nippy-jar/src/lib.rs` and `.../writer.rs`.
