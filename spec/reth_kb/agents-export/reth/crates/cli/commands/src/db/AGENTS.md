# db

## Purpose
Implements `reth db`: CLI database/static-file inspection and maintenance commands (stats/list/get/clear/diff/checksum) plus heavier repair/diagnostics tooling (trie verification/repair) and a small TUI for browsing table entries.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `mod.rs`
- **Role**: Entry point for the `reth db` command group: defines the top-level clap command, wires `EnvironmentArgs`/provider access, and dispatches to subcommands.
- **Key items**: `Command<C>`, `Subcommands`, `list::Command`, `stats::Command`, `get::Command`, `diff::Command`, `clear::Command`
- **Interactions**: constructs/uses `reth_db_common::DbTool` and forwards `tool`/`data_dir`/`task_executor` into subcommand executors.
- **Knobs / invariants**: some subcommands warn/assume the node is not running (long read transactions; safe use of RW operations).

#### `stats.rs`
- **Role**: Implements `reth db stats`: prints sizes/entry counts for MDBX tables and static-file segments, and can optionally checksum all tables.
- **Key items**: `stats::Command`, `db_stats_table()`, `static_files_stats_table()`, `checksum_report()`, `ChecksumViewer`
- **Interactions**: reads MDBX stats via `db_stat`, iterates static files via `iter_static_files`, and reuses `checksum::ChecksumViewer` for checksumming.
- **Knobs / invariants**: `--checksum` is slow (full DB traversal); `--skip-consistency-checks`, `--detailed-sizes`, `--detailed-segments`.

#### `list.rs`
- **Role**: Implements `reth db list`: paginated table browsing with filters, supporting JSON output or an interactive terminal UI.
- **Key items**: `list::Command`, `list_filter()`, `ListTableViewer`, `DbListTUI`
- **Interactions**: uses `reth_db_common::DbTool::list()` and `tui::DbListTUI` for interactive paging; disables long-read transaction safety for long sessions.
- **Knobs / invariants**: `--skip`, `--len`, `--reverse`, `--search`, `--min-*-size`, `--count`, `--json`, `--raw`.

#### `tui.rs`
- **Role**: Terminal UI used by `db list` to navigate table rows and inspect key/value payloads.
- **Key items**: `DbListTUI<F, T>`, `ViewMode`, `Entries<T>`, `event_loop()`, `handle_event()`, `CMDS`
- **Interactions**: calls a provided page fetcher \(closure\) to load pages; integrates `crossterm` input with `ratatui` rendering.
- **Knobs / invariants**: assumes raw-mode terminal operation; keybindings for paging/selection and "go to page".

#### `get.rs`
- **Role**: Implements `reth db get`: retrieves a single entry (or a range) from an MDBX table, or retrieves a record from a static-file segment (with optional raw bytes output).
- **Key items**: `get::Command`, `Subcommand`, `GetValueViewer`, `table_key()`, `maybe_json_value_parser`
- **Interactions**: uses `Tables::view()` to decode MDBX values; special-cases `StaticFileSegment::AccountChangeSets` via provider helpers.
- **Knobs / invariants**: JSON parsing for keys/subkeys; `--raw` prints bytes for MDBX/static-file reads.

#### `checksum.rs`
- **Role**: Implements `reth db checksum`: computes a stable hash over table key/value bytes for auditing or cross-node comparisons (optionally ranged/limited).
- **Key items**: `checksum::Command`, `ChecksumViewer`, `ChecksumViewer::new()`
- **Interactions**: uses a raw cursor walk and feeds bytes into a seeded hasher; reuses key parsing utilities from `get.rs`.
- **Knobs / invariants**: expects the node to be stopped (long-lived reads); supports `--start-key`, `--end-key`, `--limit`.

#### `diff.rs`
- **Role**: Implements `reth db diff`: compares a primary DB with a secondary DB (read-only) and writes discrepancy reports per table.
- **Key items**: `diff::Command`, `find_diffs()`, `find_diffs_advanced()`, `tables_to_generic!`
- **Interactions**: opens a second DB via `open_db_read_only`, walks tables with cursors, and emits text reports into an output directory.
- **Knobs / invariants**: expects the node to be stopped; supports diffing all tables or a selected table.

#### `clear.rs`
- **Role**: Implements `reth db clear`: deletes all entries for a selected MDBX table, or deletes static-file jars for a given segment.
- **Key items**: `clear::Command`, `Subcommands`, `ClearViewer`
- **Interactions**: clears MDBX tables via a RW transaction and commit; deletes static-file jars by iterating `iter_static_files`.
- **Knobs / invariants**: destructive; static-file deletion operates at the jar level by segment.

#### `static_file_header.rs`
- **Role**: Implements `reth db static-file-header`: prints the user header for a static file segment (by block/segment or by path) for debugging static-file layout/ranges.
- **Key items**: `static_file_header::Command`, `Source`, `StaticFileProviderFactory::get_segment_provider*`
- **Interactions**: performs an optional consistency check before selecting a segment provider and printing header metadata.
- **Knobs / invariants**: `Source::Block` vs `Source::Path` selection.

#### `settings.rs`
- **Role**: Implements `reth db settings`: reads or updates persisted `StorageSettings` (what lives in static files vs MDBX vs rocksdb, etc.).
- **Key items**: `settings::Command`, `Subcommands`, `SetCommand`, `StorageSettings`
- **Interactions**: reads settings via `MetadataProvider` and writes via `MetadataWriter`/`write_storage_settings()`.
- **Knobs / invariants**: distinguishes RO vs RW access needs (`Command::access_rights()`); setters are per-field toggles.

#### `repair_trie.rs`
- **Role**: Implements `reth db repair-trie`: verifies trie consistency against hashed state and (optionally) repairs DB trie tables when inconsistencies are found.
- **Key items**: `repair_trie::Command`, `Verifier`, `Output`, `verify_checkpoints()`, `verify_and_repair()`
- **Interactions**: checks stage checkpoints to ensure Merkle sync is complete; reads/writes `AccountsTrie`/`StoragesTrie` via cursors; can expose Prometheus metrics.
- **Knobs / invariants**: `--dry-run` avoids mutations; requires a consistent stage state (no in-progress MerkleExecute).

#### `account_storage.rs`
- **Role**: Implements `reth db account-storage`: estimates storage footprint for an account by scanning `PlainStorageState` dupsort entries.
- **Key items**: `account_storage::Command`, `PlainStorageState`, `keccak256`, `LOG_INTERVAL`
- **Interactions**: iterates dupsort values and uses `reth_codecs::Compact` to estimate encoded size; prints plain vs hashed size estimates.
- **Knobs / invariants**: logs progress periodically; operates as a full scan over the address's storage entries.

## Key APIs (no snippets)
- **Top-level**: `db::Command<C>`, `db::Subcommands`
- **Subcommands**: `stats::Command`, `list::Command`, `get::Command`, `diff::Command`, `checksum::Command`, `clear::Command`, `repair_trie::Command`, `static_file_header::Command`, `settings::Command`, `account_storage::Command`
- **TUI**: `DbListTUI<F, T>`, `ViewMode`

## Relationships
- **Used by**: `reth/crates/cli/commands/src/lib.rs` (registers `db` under the main CLI command set).
- **Depends on**: `reth-db*` + `reth-db-common` (tooling/cursors), `reth-provider` (provider factories/static files/metadata), `reth-trie*` (verification/repair), and `ratatui`/`crossterm` (interactive TUI).

## End-to-end flow (high level)
- Parse `reth db ...` CLI args into `db::Command<C>` + `Subcommands`.
- Initialize environment/provider access according to command needs (RO vs RW).
- Build `DbTool` and dispatch to the selected subcommand.
- Subcommand runs a DB/static-file read/write routine (cursor walk, table stats, static-file provider lookup, etc.).
- Print results (JSON/text/tables) or run `DbListTUI` for interactive navigation.
