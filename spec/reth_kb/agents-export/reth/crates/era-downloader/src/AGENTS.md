# src

## Purpose
Implements `reth-era-downloader`: an async interface for discovering and fetching `.era`/`.era1` history files from remote hosts (or local directories), with streaming/concurrency controls and optional checksum verification (for `.era1`).

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `lib.rs`
- **Role**: Crate entrypoint: wires modules and exposes the public API surface for clients/streams.
- **Key items**: `EraClient`, `EraStream`, `EraStreamConfig`, `EraMeta`, `read_dir()`, `BLOCKS_PER_FILE`
- **Interactions**: re-exports `HttpClient` trait to allow custom HTTP backends (used heavily in tests with a stub client).
- **Knobs / invariants**: `BLOCKS_PER_FILE` (8192) is used to map block numbers to era1 file indices.

#### `client.rs`
- **Role**: Remote download client: fetches an index page and (for `.era1`) `checksums.txt`, extracts ordered filenames, downloads files to disk, and verifies checksums when applicable.
- **Key items**: `HttpClient`, `EraClient<Http>`, `download_to_file()`, `fetch_file_list()`, `number_to_file_name()`, `verify_checksum()`, `recover_index()`, `delete_outside_range()`
- **Interactions**: uses `reth_era::common::file_ops::EraFileType` to distinguish `.era` vs `.era1`; streams bytes via `HttpClient` and writes to `tokio::fs::File`.
- **Knobs / invariants**: checksum verification is enforced for `.era1` (SHA-256); `.era` hosts may not provide checksums so verification is skipped.

#### `stream.rs`
- **Role**: Asynchronous stream abstraction that schedules and runs downloads concurrently while enforcing a cap on concurrent downloads and on-disk file count.
- **Key items**: `EraStream<Http>`, `EraStreamConfig`, `EraMeta`, `EraRemoteMeta`, `EraStreamConfig::start_from()`
- **Interactions**: coordinates with `EraClient` for URL discovery and download; returns temporary local files whose lifecycle is controlled via `EraMeta::mark_as_processed()`.
- **Knobs / invariants**: `max_files` limits how many downloaded files may exist simultaneously; `max_concurrent_downloads` controls parallelism; internal state machine advances through "fetch list -> delete outside range -> recover index -> schedule URLs".

#### `fs.rs`
- **Role**: Local-directory adapter: reads `.era1` files from disk in index order starting at a computed start point, validating against `checksums.txt`.
- **Key items**: `read_dir()`, `EraLocalMeta`, `EraMeta::mark_as_processed()` (no-op)
- **Interactions**: uses `sha2` to compute file checksums and aligns them with `checksums.txt` lines by index.
- **Knobs / invariants**: requires `checksums.txt` to exist and have at least as many entries as files in range; start position is derived from `start_from / BLOCKS_PER_FILE`.

## Key APIs (no snippets)
- **HTTP abstraction**: `HttpClient`
- **Remote client**: `EraClient`
- **Streaming download**: `EraStream`, `EraStreamConfig`, `EraMeta`
- **Local ingestion**: `read_dir()`

## Relationships
- **Used by**: ERA import tooling (`reth-cli-commands` `import-era`, and other utilities) that want to stream history files into storage/pipeline ingestion.
