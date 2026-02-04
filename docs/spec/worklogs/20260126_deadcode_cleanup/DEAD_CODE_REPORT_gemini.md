Here is a report on dead, unused, and legacy-looking code found in the `stateless-history-node` repository, cross-referenced with the provided documentation.

### 1. Explicit Dead Code (`#[allow(dead_code)]`)
These are items explicitly marked by the developer as unused. While some may be reserved for future features (per `ROADMAP.md`), others appear to be abandoned scaffolding or debug tools.

#### Storage & WAL (Write Ahead Log)
The `sharded/wal.rs` module contains significant logic for reading and verifying WAL files that is not wired up to the main application lifecycle. The current recovery logic uses `build_index` (scanning headers) rather than full payload verification.
*   **`node/src/storage/sharded/wal.rs`**:
    *   `read_records`: A full WAL reader implementation.
    *   `WalPayloadCrcReader`: A struct and implementation for validating CRC checksums of WAL payloads.
    *   `read_wal_bundle_record_at`: Logic to read a specific record by offset.
    *   `WalIndexEntry`: Fields `record_offset` and `payload_len` are unused (only `block_number` is used for recovery).
*   **`node/src/storage/sharded/mod.rs`**:
    *   `records_to_map`: Helper to convert WAL records to a map.
    *   `SegmentWriter::append_rows_no_commit` & `SegmentWriter::commit`: The current compaction logic uses `append_rows` which commits immediately.
    *   `DirtyShardInfo`: Fields `sorted` and `complete` are unused in the return struct of `dirty_shards`.

#### P2P & Networking
Abstracts created for testing or future peer selection strategies that are currently bypassed by `PeerPool`.
*   **`node/src/p2p/mod.rs`**:
    *   `RoundRobinPeerSelector` & `PeerSelector` trait: The sync loop uses `PeerPool` and `pick_best_ready_peer_index` directly.
    *   `SelectorPeerId`: Type alias.
    *   `request_receipts_chunked_partial`: Fetch helper unused in production (only used in tests).

#### Sync & Scheduler
*   **`node/src/sync/historical/types.rs`**:
    *   `ProcessingMode`: Enum (`Probe`, `Ingest`) is unused; the mode is determined implicitly by which function (`run_benchmark_probe` vs `run_ingest_pipeline`) is called.
    *   `FetchedBlock`: Struct unused. (See "Legacy Logic" section below).
*   **`node/src/sync/historical/process.rs`**:
    *   `process_probe`: Function unused.
*   **`node/src/sync/historical/db_writer.rs`**:
    *   `DbWriterMessage::Flush`: The flush logic is handled, but this specific enum variant is never sent.
*   **`node/src/sync/historical/scheduler.rs`**:
    *   `PeerHealthConfig`: `partial_threshold_multiplier` and `partial_ban_duration`.
    *   `PeerWorkScheduler`: `completed_count`, `failed_count`.

#### Metrics
*   **`node/src/metrics.rs`**:
    *   `lag_to_head`: Unused helper.
    *   `rate_per_sec`: Unused helper (duplicated in `stats.rs`).

### 2. Unused / Placeholder Code
Code that is technically compiled (or reachable in tests) but serves no function in the current binary.

*   **`node/src/chain/`**: This directory is empty (except for an `AGENTS.md` implied in the file list). It appears to be a placeholder for chain-spec logic that currently lives in `main.rs` or `cli/mod.rs`.
*   **`node/tests/test_strategy.rs`**: Contains `rpc_contract_probe` and `reorg_rollback_integration` tests which are empty and marked `#[ignore]`.
*   **`node/src/rpc/mod.rs`**: `RpcWithdrawal` struct is defined and serialized, but the `eth_getBlockByNumber` implementation hardcodes `withdrawals: None`. (Consistent with PRD "withdrawals not stored").

### 3. Legacy & Repeating Patterns
Patterns that suggest refactoring inconsistencies or older architectural decisions.

#### The `process_probe` Vestige
In `node/src/sync/historical/process.rs`, the function `process_probe` takes a `FetchedBlock` and returns a `ProbeRecord`.
*   **Status:** Dead.
*   **Context:** The probe pipeline in `mod.rs` calls `fetch::fetch_probe_batch`. That function internally converts headers/receipts directly into `ProbeRecord`s (line 58 of `fetch.rs`), bypassing the `FetchedBlock` struct and `process_probe` function entirely. This suggests an earlier design where probe mode shared the same "Fetch -> Process -> Write" channel architecture as Ingest.

#### Duplicate Metrics Logic
*   **`rate_per_sec`**: Defined in `node/src/metrics.rs` (unused) and redefined/used locally in `node/src/sync/historical/stats.rs`.
*   **Percentile Logic**: `stats.rs` implements ad-hoc percentile calculation (`percentile`, `percentile_pair`, `percentile_triplet`). This could likely be moved to `metrics.rs` to centralize math helpers.

#### Stubbed Storage Traits
`node/src/storage/mod.rs` implements several methods that return `Ok(None)` or empty vectors:
*   `block_logs`, `block_logs_range`
*   `block_withdrawals`
*   `log_index_by_address_range`, `log_index_by_topic0_range`

**Assessment:** These are **NOT** dead code in the traditional sense; they are stubbed implementations required to fulfill the internal `Storage` API surface which anticipates future indexing features (per `ROADMAP.md` "Post-sync eth_getLogs index build").

### 4. Summary List of Candidates for Removal

If cleaning up the repo, the following can be safely removed without affecting `v0.2` functionality:

1.  **File:** `node/src/metrics.rs` (Move `range_len` to a util module, delete the rest).
2.  **Struct:** `FetchedBlock` (`sync/historical/types.rs`).
3.  **Function:** `process_probe` (`sync/historical/process.rs`).
4.  **Struct:** `RoundRobinPeerSelector` & `PeerSelector` (`p2p/mod.rs`).
5.  **Struct:** `WalPayloadCrcReader` and `read_wal_bundle_record_at` (`storage/sharded/wal.rs`) — unless a WAL verification CLI tool is planned for v0.3.
6.  **Enum Variant:** `DbWriterMessage::Flush` (`sync/historical/db_writer.rs`).
7.  **Fields:** `PeerHealthConfig` unused fields (`partial_threshold_multiplier`, etc.).