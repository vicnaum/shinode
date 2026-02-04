# it

## Purpose
Integration tests for `reth-era-downloader`: validate file list parsing, URL selection, download streaming behavior, checksum enforcement (ERA1), and local-directory streaming.

## Contents (one hop)
### Subdirectories
- (none)

### Files (detailed)

#### `main.rs`
- **Role**: Test harness that bundles all integration tests into a single binary and provides a `StubClient` implementing `HttpClient` using embedded fixtures from `tests/res/`.
- **Key items**: `StubClient`, `ERA1_*`/`ERA_*` fixture constants, module wiring (`checksums`, `download`, `fs`, `list`, `stream`)
- **Interactions**: `StubClient::get()` maps known URLs to fixture bytes (HTML index pages, checksums.txt, and small `.era`/`.era1` sample files).

#### `list.rs`
- **Role**: Tests index-page parsing and ordered filename lookup (`number_to_file_name`) after `fetch_file_list()`.
- **Key items**: `EraClient::fetch_file_list()`, `EraClient::number_to_file_name()`

#### `download.rs`
- **Role**: Tests URL construction and download-to-disk behavior for both `.era1` and `.era` after fetching the file list.
- **Key items**: `EraClient::url()`, `EraClient::download_to_file()`, `EraClient::files_count()`

#### `stream.rs`
- **Role**: Tests `EraStream` scheduling and error behavior (including missing download directory), asserting that downloaded paths match expected filenames.
- **Key items**: `EraStream`, `EraStreamConfig`, `EraStreamConfig::with_max_files()`, `EraStreamConfig::with_max_concurrent_downloads()`

#### `checksums.rs`
- **Role**: Tests that invalid ERA1 checksums are detected and surfaced as errors (using a `FailingClient` with intentionally wrong `checksums.txt`).
- **Key items**: `FailingClient`, checksum mismatch assertions

#### `fs.rs`
- **Role**: Tests `read_dir()` behavior for local directories (valid checksums, invalid checksums, partial failures, and missing `checksums.txt`).
- **Key items**: `read_dir()`, checksum mismatch error strings

## Relationships
- **Uses fixtures from**: `tests/res/` (embedded via `include_bytes!` in `main.rs`).
