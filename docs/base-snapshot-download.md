# Import Base snapshot history with Shinode

The `snapshot-import` command downloads and verifies selected Base history, then
writes it through Shinode's normal validation, ingestion, WAL, compaction and
sealing pipeline. It uses the Rust binary alone: Python, curl and a separate zstd
executable are not required at runtime.

The initial production reader supports the verified Base V2 producer
`2.5.2-dev (76a8261)` and **pre-Canyon bodies only**. Later-fork headers, unknown
producer versions, pruned inputs and incompatible archive layouts are rejected.
It requires a **new output directory**. Append/resume of node imports, automatic
scratch reclamation and P2P handoff remain subsequent phases.

## Inspect the input

Obtain and retain the [pinned manifest](https://mainnet-v2-snapshots.base.org/1789084806/manifest.json).
Its SHA-256 is recorded in the [compatibility evidence](base-snapshot-import.md).
Build the development checkout, then inspect a bounded range:

```sh
cargo build --release --locked

./target/release/shinode snapshot-import \
  --manifest /path/to/manifest.json \
  --manifest-sha256 d7ddeea72a46b542d3ca97672a64ba73ebc07599f0b19fd8965c00c2b317801c \
  --start-block 4984 --end-block 5527 --dry-run
```

`--dry-run` reads only the local manifest. It creates no directories and makes no
archive requests. JSON output lists the requested range, dependency range, exact
archive URLs, output checksums, compressed/extracted sizes and supported format.
It reports resource requirements even if they exceed the default transfer budget;
actual import refuses that selection until suitable explicit limits are provided.

Only headers, transactions and receipts are selected. State, changesets, senders
and RocksDB indices are excluded. The manifest and its SHA stay fixed throughout
retries; the command never consults a moving `latest` pointer. Shared completed
chunks and snapshot-specific tail URLs are both supported.

The reader currently reconstructs transaction boundaries from genesis. A later
requested range therefore includes the complete header/transaction/receipt prefix
through its end block. Inspection makes that dependency visible. The downloader
can inspect different component chunk widths, but the conversion reader currently
requires aligned block ranges across the three components.

For the example, three archives total **85,140,341 compressed bytes** and
**279,526,316 extracted bytes**. Through the pinned snapshot tip, the same three
components total **1.88 TB compressed / 3.80 TB extracted** across 309 archives.
That whole-history selection exceeds the defaults and also crosses the current
reader's supported fork range; inspection is not a promise that it can be imported.

## Download and import

```sh
./target/release/shinode snapshot-import \
  --chain base \
  --manifest /path/to/manifest.json \
  --manifest-sha256 d7ddeea72a46b542d3ca97672a64ba73ebc07599f0b19fd8965c00c2b317801c \
  --start-block 4984 --end-block 5527 \
  --staging-dir /path/to/base-snapshot-stage \
  --output-dir /path/to/new-base-history
```

Base is the command's default chain. An explicit conflicting chain or chain ID is
rejected. Staging and output must be separate directories and cannot be nested.
Any existing output directory is refused, including an empty one. Existing
Ethereum/Base databases and conflicting overlaps are therefore preserved.

Add `--offline` to use verified staged chunks without archive requests. The same
manifest and range must match staging's saved identity. After a download or
preflight failure, repeat the command to resume staging. If writing to the output
has already started and the process fails, use a new output directory: import
checkpoints and recovery into an existing destination are not implemented yet.

| Limit | Default | Option |
| --- | --- | --- |
| Selected compressed bytes / per-run transferred payload including retries | 512 MiB | `--max-download-bytes` |
| Selected extracted payload | 1 GiB | `--max-extracted-bytes` |
| Free disk reserve | 5 GiB | `--min-free-bytes` |
| Concurrent archive workers | 2, maximum 4 | `--concurrency` |
| Retries per archive | 3, maximum 10 | `--retries` |
| Whole HTTP attempt timeout | 600 seconds | `--timeout-seconds` |
| Import batch size | 64 blocks, maximum 1,024 | `--batch-blocks` |
| Transactions in one reconstructed block | 10,000 | `--max-transactions-per-block` |

The HTTP connection timeout is 30 seconds. The zstd decoder's window is limited to
128 MiB per worker; this is not a total process RSS limit. Tar metadata/padding is
bounded to 1 MiB beyond the manifest's declared payload. Stage writes check the
free-space reserve before writing. Import batches check space before WAL writes
and compaction, budgeting the incoming batch plus a copy of each affected shard's
existing sorted files and bitset; storage failures are surfaced rather than
reported as successful imports.

Archives and verified chunks are retained. Peak input space includes both the
compressed and extracted selection. The output database and compaction working
space are additional. This phase does not yet reclaim inputs after conversion.

## Validation and publication

A private staging directory holds the exact manifest, fixed job identity, a
process lock, archive partials and HTTP validators, temporary extraction files,
verified chunks, and `download-report.json`. A directory belonging to another
manifest/range or an unrelated nonempty directory is refused.

HTTP resume uses `Range` with a strong ETag in `If-Range`, following
[HTTP range semantics](https://www.rfc-editor.org/rfc/rfc9110.html#name-if-range).
A `206` response must match the saved validator and exact remaining byte range.
A full `200` restarts the file safely; without a strong validator, old partial
bytes are discarded. Invalid partial responses are rejected, `416` restarts the
transfer, and transient network/server failures retry with bounded backoff.
Redirects are refused so a changed source can be reviewed and pinned explicitly.

Tar extraction accepts only the declared regular data, `.conf` and `.off` files,
with exact sizes and BLAKE3 checksums. Links, traversal, sparse/extended metadata,
duplicates and unexpected entries are rejected. A chunk becomes available only
after its complete verified extraction directory is renamed into place. An
interrupted extraction is retried from its retained archive; only that job's
unfinished extraction directory is cleared. Restarts rehash verified files.

Before creating output storage, Shinode scans the entire required prefix, checks
the Base genesis, header hashes and parent continuity, reconstructs transaction
boundaries, checks transaction/receipt types, gas, roots, bloom and withdrawals,
and verifies conversion through `BaseChain::process_ingest`. Transaction-number
continuity and full row consumption are checked at archive boundaries. A second
bounded pass writes only the requested blocks; it does not buffer the whole range.

The importer compacts and seals completed shards, reopens storage, checks coverage,
and writes `snapshot-import.json` only after successful completion. Progress reports
archive sizes and limits, transferred bytes, verification progress, imported block
counts, average throughput and stored bytes on stderr. Existing
storage/RPC formats are reused. Snapshot checksums and internally consistent blocks
do not by themselves prove canonicality or L1 finality.

## Reproducible tests

```sh
cargo test --workspace --locked

SHINODE_SNAPSHOT_SAMPLE=/path/to/base-snapshot-stage \
SHINODE_SNAPSHOT_OUTPUT=/path/to/new-snapshot-test-output \
  cargo test --bin shinode --locked import_snapshot_sample -- --ignored --nocapture
```

The opt-in test invokes the production importer for blocks **4,984–5,527** (544
blocks), reopens the database and queries the production RPC module. It records
16 sampled block/log/deposit responses covering the original phase-2 reference
ranges. These responses matched the independently verified phase-2 results.
The normal suite uses generated static-file fixtures across archive boundaries
and local HTTP servers; it requires no public snapshot downloads.

Python experiments were used to establish the format and are kept outside the
repository. They are not part of the production workflow or the intended PR.

## Phase-4 evidence (2026-09-12)

- Workspace tests: **121 passed, six ignored**, including growing-shard space
  budgets and symlink/parent destination resolution. The production snapshot proof
  passed separately; sampled block/log/deposit responses matched the independent
  phase-2 reference evidence.
- The Rust HTTPS smoke reused verified headers/transactions and downloaded only
  the **1,133,974-byte receipts archive**. Staging verified all nine output files
  in 2.76 seconds. Conversion, compaction and reopen imported **544 blocks** in
  1.43 seconds, reporting **517,700 storage bytes** with eight-block test shards.
  These are local debug-build sample measurements, not full-history projections.
- The build without default features passed. New snapshot modules are clean under
  Clippy; the existing workspace binary/test lint failures remain.
- All previously locked package versions and the shared Reth revision were
  retained. Added dependencies provide native HTTPS, tar, BLAKE3 and staging locks.

Raw manifests, snapshots, databases and RPC reports remain local. The implementation
and reusable Rust tests are intended for the PR; temporary plans and Python
prototypes are excluded.
