# wal

## Purpose
Write-ahead log (WAL) for ExEx notifications: persists notifications to disk (MessagePack) and maintains an in-memory block cache for efficient lookup and finalization to bound growth.

## Contents (one hop)
### Subdirectories
- (none)

### Files
- `mod.rs` - `Wal` and `WalHandle` facade: commit/finalize/iterate notifications and cached lookup by committed block hash.
  - **Key items**: `Wal`, `Wal::commit()`, `Wal::finalize()`, `Wal::iter_notifications()`, `WalHandle`
- `storage.rs` - file-backed storage: each notification is a `{id}.wal` file encoded via `rmp-serde` using `serde_bincode_compat` wrapper types.
  - **Key items**: `Storage`, `files_range()`, `read_notification()`, `write_notification()`, `remove_notifications()`
- `cache.rs` - in-memory cache of notification max blocks and committed block hash->(file id, metadata) mapping to avoid scanning storage on lookups/finalization.
  - **Key items**: `BlockCache`, `insert_notification_blocks_with_file_id()`, `remove_before()`
- `metrics.rs` - WAL metrics (bytes, notification counts, committed block heights).
  - **Key items**: `Metrics`
- `error.rs` - WAL error types and result alias.
  - **Key items**: `WalError`, `WalResult<T>`
