# Q010 - Storage Migrations and Versioning

Status: verified

## Question
How are storage migrations and versioning handled beyond `database.version`?

## Context Packs
- `spec/reth_repo_context/context-storage.xml`

## Gemini Prompt

```text
You are analyzing the Rust codebase for Reth. I attached the following Repomix XML pack:
- spec/reth_repo_context/context-storage.xml

Question:
How are storage migrations and versioning handled beyond `database.version`?
I want to understand how client versions are recorded, how incompatible changes are detected, and how any migration hooks are wired.

Requirements:
1. Cite file paths and function/type names for every major claim.
2. Identify where client versions are recorded and how they are queried.
3. Note any explicit migration or compatibility checks beyond `check_db_version_file`.
4. If a claim is outside the pack, say so explicitly.

Please structure the answer as:
1. Summary (5-8 bullets)
2. Entry points (files + types + purpose)
3. Versioning/migration flow (step-by-step)
4. Key abstractions/types (with file paths)
5. “Where to look next”
6. Suggested follow-up questions
```

## Gemini Answer
Based on the `context-storage.xml` pack, here is an analysis of how Reth handles versioning, client tracking, and migrations.

### 1. Summary
*   **Strict Schema Versioning**: The primary gatekeeper is a hard-coded constant (`DB_VERSION = 2`). If the file `database.version` on disk does not match this constant, startup fails immediately. There is no automatic "upcasting" or migration logic visible in this pack for the global schema version.
*   **Additive Schema Updates**: Table creation is idempotent. The `init_db` function calls `create_db` for every table defined in the `tables!` macro. If the code adds a new table, it is created automatically on the next startup; existing tables are opened.
*   **Client Version History**: Reth maintains a dedicated MDBX table (`VersionHistory`) that logs every unique client software version (e.g., "reth/v1.0.0") that has opened the database in Read-Write mode, keyed by timestamp.
*   **Configuration persistence**: "Soft migrations" or behavior changes are handled via `StorageSettings` stored in the `Metadata` table. This allows the node to persist decisions (like "store receipts in static files") across restarts without changing the global schema version.
*   **No Explicit Migration Framework**: There is no SQL-style migration runner (e.g., "apply migration 001, then 002") visible in this pack. Version mismatches result in errors, implying migration is likely manual or handled by external tools/commands not included here.

---

### 2. Entry Points

| File Path | Type / Function | Purpose |
| :--- | :--- | :--- |
| `crates/storage/db/src/mdbx.rs` | `record_client_version` | The core logic that checks the last recorded client version and appends a new one if it has changed. |
| `crates/storage/db-api/src/tables/mod.rs` | `VersionHistory` | The MDBX table definition storing `(u64, ClientVersion)`. Key is a UNIX timestamp. |
| `crates/storage/db-api/src/models/metadata.rs` | `ClientVersion` | The struct serializing the git SHA, version string, and build timestamp. |
| `crates/storage/db/src/version.rs` | `check_db_version_file` | The "hard gate" that returns `DatabaseVersionError::VersionMismatch` if the disk version != code version. |
| `crates/storage/provider/src/providers/database/mod.rs` | `ProviderFactory::new` | Loads `StorageSettings` from the `Metadata` table to configure how data is written (e.g., Legacy vs. Edge behavior). |

---

### 3. Versioning & Migration Flow

#### A. The Hard Gate (Schema Version)
1.  **Startup**: When `init_db` is called, it invokes `check_db_version_file` (`crates/storage/db/src/version.rs`).
2.  **Check**: It reads the plain-text file `database.version` in the data directory.
3.  **Validate**: It compares the number in the file to `pub const DB_VERSION: u64 = 2`.
4.  **Result**:
    *   If equal: Proceed.
    *   If mismatch: Return `VersionMismatch { version: u64 }`. **There is no code here to resolve this mismatch automatically.**

#### B. The Audit Log (Client Version)
1.  **Open RW**: When `DatabaseEnv::open` or `init_db` creates a Read-Write transaction, it calls `record_client_version` (`crates/storage/db/src/mdbx.rs`).
2.  **Query Last**: It opens a cursor on the `VersionHistory` table and fetches the `last()` entry.
3.  **Compare**: It compares the `ClientVersion` struct passed in `DatabaseArguments` with the one on disk.
4.  **Append**: If they differ, it gets the current system time (UNIX timestamp) and inserts a new row: `tx.upsert(timestamp, &version)`.

#### C. Feature Flags (Storage Settings)
1.  **Initialization**: `ProviderFactory::new` creates a temporary provider to read the `Metadata` table (`crates/storage/provider/src/providers/database/mod.rs`).
2.  **Read**: It looks for the key `storage_settings`.
3.  **Decide**:
    *   If found: It deserializes `StorageSettings` (e.g., `receipts_in_static_files: true`).
    *   If missing: It defaults to `StorageSettings::legacy()`.
4.  **Enforce**: These settings are cached in `ProviderFactory` and dictate whether subsequent writes go to MDBX or Static Files (via `EitherWriter`).

---

### 4. Key Abstractions & Types

| Type | File Path | Description |
| :--- | :--- | :--- |
| `ClientVersion` | `crates/storage/db-api/src/models/metadata.rs` | Contains `version` (String), `commit_sha` (String), `build_timestamp` (String). Implements `Compact` for efficient storage. |
| `VersionHistory` | `crates/storage/db-api/src/tables/mod.rs` | `Table<Key=u64, Value=ClientVersion>`. The key is the timestamp of when the version was first seen. |
| `StorageSettings` | `crates/storage/db-api/src/models/metadata.rs` | Controls granular data layout features (e.g., `account_history_in_rocksdb`). This acts as a soft versioning system for feature flags. |
| `Metadata` | `crates/storage/db-api/src/tables/mod.rs` | A general-purpose `Table<Key=String, Value=Vec<u8>>` used to store `StorageSettings` and potentially other migration flags. |

---

### 5. Where to Look Next
*   **`crates/storage/db/src/mdbx.rs`**: Specifically `record_client_version` to see the exact logic for determining if a new history entry is needed.
*   **`crates/storage/db-api/src/models/metadata.rs`**: To see the specific flags available in `StorageSettings` (Legacy vs Edge) which act as the primary way Reth handles layout evolution without breaking `DB_VERSION`.
*   **Outside this pack**: `reth-node` or `reth-cli` likely contains the logic that handles the `DatabaseVersionError`—potentially prompting the user to run a separate migration command (like `reth db migrate`), as no such logic exists in the storage crates themselves.

---

### 6. Suggested Follow-up Questions
*   "How are the `StorageSettings` initially written to the `Metadata` table during a fresh node initialization?"
*   "What happens if the `VersionHistory` table contains a client version newer than the currently running code?" (The code suggests nothing happens; it just appends the current version, but verifying this behavior is useful).
*   "Since `DB_VERSION` is hardcoded to 2, where is the code that handled the migration from version 1, and is it still in the codebase?"

## Verification
- `DB_VERSION` and `check_db_version_file` enforce the `database.version` check.```9:52:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/version.rs
pub const DB_VERSION: u64 = 2;
pub fn check_db_version_file<P: AsRef<Path>>(db_path: P) -> Result<(), DatabaseVersionError> {
    let version = get_db_version(db_path)?;
    if version != DB_VERSION {
        return Err(DatabaseVersionError::VersionMismatch { version })
    }
    Ok(())
}
```
- `create_db` runs the version check (or creates the file) before opening RW MDBX.```12:28:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/mdbx.rs
if is_database_empty(rpath) { ... create_db_version_file(rpath)?; } else {
    match check_db_version_file(rpath) { ... Err(err) => return Err(err.into()) }
}
Ok(DatabaseEnv::open(rpath, DatabaseEnvKind::RW, args)?)
```
- `init_db` creates tables for the default `Tables` set and records client version.```31:47:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/mdbx.rs
pub fn init_db<P: AsRef<Path>>(path: P, args: DatabaseArguments) -> eyre::Result<DatabaseEnv> {
    init_db_for::<P, Tables>(path, args)
}
...
let mut db = create_db(path, args)?;
db.create_and_track_tables_for::<TS>()?;
db.record_client_version(client_version)?;
```
- `VersionHistory` and `Metadata` tables exist with the claimed key/value types.```535:552:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-api/src/tables/mod.rs
table VersionHistory { type Key = u64; type Value = ClientVersion; }
table Metadata { type Key = String; type Value = Vec<u8>; }
```
- `record_client_version` stores versions keyed by UNIX timestamp seconds if changed.```563:577:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db/src/implementation/mdbx/mod.rs
let last_version = version_cursor.last()?.map(|(_, v)| v);
if Some(&version) != last_version.as_ref() {
    version_cursor.upsert(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(), &version)?;
    tx.commit()?;
}
```
- `ClientVersion` is defined in `db-models` (and re-exported), not `models/metadata.rs`.```10:16:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/db-models/src/client_version.rs
pub struct ClientVersion {
    pub version: String,
    pub git_sha: String,
    pub build_timestamp: String,
}
```
- Storage settings are stored as JSON in `Metadata` under the `storage_settings` key.```8:24:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/storage-api/src/metadata.rs
pub mod keys { pub const STORAGE_SETTINGS: &str = "storage_settings"; }
fn storage_settings(&self) -> ProviderResult<Option<StorageSettings>> {
    self.get_metadata(keys::STORAGE_SETTINGS)?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(ProviderError::other))
        .transpose()
}
```
- `ProviderFactory::new` loads cached storage settings via `storage_settings()` (fallback legacy).```97:113:/Users/vicnaum/github/stateless-history-node/reth/crates/storage/provider/src/providers/database/mod.rs
let legacy_settings = StorageSettings::legacy();
let storage_settings = DatabaseProvider::<_, N>::new(...).storage_settings()?.unwrap_or(legacy_settings);
```

## Corrections / Caveats
- `record_client_version` is defined on `DatabaseEnv` in `implementation/mdbx/mod.rs`, not in `mdbx.rs` (though `open_db`/`init_db` call it).
- `ClientVersion` is defined in `db-models` and re-exported via `db-api` models, not in `models/metadata.rs`.
- No explicit migration runner is shown in this pack; that claim remains an inference about absence.

## Actionable Pointers
- Global DB version: `reth/crates/storage/db/src/version.rs`, `reth/crates/storage/db/src/mdbx.rs`.
- Client version history: `reth/crates/storage/db/src/implementation/mdbx/mod.rs`, `reth/crates/storage/db-api/src/tables/mod.rs`.
- Metadata + storage settings: `reth/crates/storage/storage-api/src/metadata.rs`, `reth/crates/storage/db-api/src/tables/mod.rs`.
- Provider cache load: `reth/crates/storage/provider/src/providers/database/mod.rs`.
