# Base integration validation

Verified on **2026-09-12**, on macOS arm64 with Rust 1.93.1. Snapshot import,
history packaging, restoration, and P2P continuation belong to the existing Base
integration in [PR #7](https://github.com/vicnaum/shinode/pull/7).

The release binary imported **50,000 blocks (0–49,999)** into 50 complete shards,
exported them, and restored the package into a second clean database. Both
reopened databases matched an independently fetched `https://mainnet.base.org`
reference for 26 sampled blocks, 25 sampled deposit receipts, and **every log
across the entire range: 13 logs**. The recipient used only the native package.

This is a quiet early-history range, not a current-throughput benchmark or a
whole-history capacity estimate. The real run stays within the first source
archive and before Canyon. Generated tests cover source archive boundaries;
later-fork execution snapshot bodies are deliberately rejected.

## Source, scope and trust

| Property | Value |
| --- | --- |
| Network | Base mainnet, chain ID 8453 |
| Pinned V2 manifest | [1789084806/manifest.json](https://mainnet-v2-snapshots.base.org/1789084806/manifest.json) |
| Manifest SHA-256 | `d7ddeea72a46b542d3ca97672a64ba73ebc07599f0b19fd8965c00c2b317801c` |
| Reported producer | `2.5.2-dev (76a8261)` |
| Shinode's unchanged Reth pin | `a74cb9cbc38a5590b2596c67e64a3c7642024aca` |
| End block | 49,999 (`0xc34f`) |
| Independently fetched end hash | `0x3b46cee7d00681f5c11ce3138e1aa3e5ad14b34b5d4ef76170428d3c65506f68` |
| Native format | Storage schema 2, `op_v1`, 1,000 blocks per shard |
| Import / restore batch | 1,000 blocks |

The source history selection consists of the first header, transaction, and
receipt archives. Its compressed payload is **85,140,341 bytes**, and its nine
extracted files total **279,526,316 bytes**. The measured import reused these
already verified files offline in a new range-specific staging directory; it
made no archive requests. Download/extraction time and compressed archive space
are therefore excluded from the measured run. A fresh online run needs those
resources in addition to the measurements below.

No execution state, sender archive, RocksDB index or execution-client database was
used. All selected source checksums and all imported payload commitments were
validated. Native restore checked archive/file hashes, seals, full coverage,
parent continuity and every stored bundle before publishing its trusted boundary.
The independent comparison used headers, transaction hashes, exact block sizes,
receipt types, deposit nonce/version fields, and complete standard log fields. Each
reference receipt type must match its stored type before deposit fields are checked;
a regression rejects a deposit incorrectly stored as an ordinary receipt. Samples include
genesis, the end block, the known ordinary-transaction blocks 4,987 and 5,521,
and both sides of ten native shard boundaries.

The supplied checkpoint anchors header history; the publisher manifest pin
also authenticates native stored fields. These checks do not establish OP
safe/finalized status or L1 finality. Native transaction metadata cannot reproduce
the original full transaction-envelope root validation. See the
[format and trust limits](base-history-packages.md).

## Measurements

Each operation ran the release binary without compilation or RPC-reference-fetch
time. `/usr/bin/time -l` measured process peak RSS. File sizes and allocated file
blocks under the evidence directory were sampled every 200 ms, including retained
inputs, source output, package, recipient, and active extraction/compaction files.
Sampling can miss shorter peaks; allocated-file totals exclude directory metadata
and are not a filesystem-wide capacity bound. Build caches and earlier experiments
outside that directory are excluded.

| Operation | Wall time | Peak RSS | Sampled peak file bytes | Sampled peak allocated file bytes |
| --- | ---: | ---: | ---: | ---: |
| Offline import | 35.8 s | 77,365,248 | 325,291,571 | 327,077,888 |
| Export | 15.8 s | 39,927,808 | 333,734,058 | 335,671,296 |
| Clean restore | 91.1 s | 39,534,592 | 420,205,005 | 425,574,400 |

End-to-end import throughput was about **1,397 blocks/s**, and restore about
**549 blocks/s** for this range. All operations retained the configured **5 GiB**
free-space reserve; the lowest observed filesystem availability exceeded 44 GB.
These measurements include verification and durable publication, not just decoding.

| Output | Bytes |
| --- | ---: |
| Selected native shard files | 43,145,853 |
| Compressed shard archives | 8,263,617 |
| Distribution manifest | 177,167 |
| Total distributed package | **8,440,784** |

The package is **80.4% smaller** than the selected native files. Much of this quiet
range is repetitive headers and deposits; native receipt/transaction segments are
already compressed. The earlier eight-block-shard sample saved only 21% including
its manifest. Neither ratio should be projected onto busy or later-fork history.

A completed import also resumed successfully after its extracted input files were
moved out of staging. Staging recreated empty directories; no input files were
needed or produced. The source files were then returned to their original location.
Generated tests separately exercise interruption before completion and recovery
from pending WAL and compaction checkpoints.

## Reproduce the workflow

Build the checked-out integration and obtain the exact pinned manifest. Use a new
staging/output pair for this range:

```sh
cargo build --release --bin shinode --locked

./target/release/shinode --shard-size 1000 snapshot-import \
  --manifest /path/to/manifest.json \
  --manifest-sha256 d7ddeea72a46b542d3ca97672a64ba73ebc07599f0b19fd8965c00c2b317801c \
  --start-block 0 --end-block 49999 \
  --checkpoint-hash 0x3b46cee7d00681f5c11ce3138e1aa3e5ad14b34b5d4ef76170428d3c65506f68 \
  --staging-dir /path/to/stage-50000 \
  --output-dir /path/to/base-50000 \
  --batch-blocks 1000 --keep-scratch

./target/release/shinode --data-dir /path/to/base-50000 history-export \
  --start-block 0 --end-block 49999 --output-dir /path/to/new-package

./target/release/shinode history-restore \
  --manifest /path/to/new-package/manifest.json \
  --manifest-sha256 <sha256-printed-by-export> \
  --checkpoint-hash 0x3b46cee7d00681f5c11ce3138e1aa3e5ad14b34b5d4ef76170428d3c65506f68 \
  --output-dir /path/to/new-recipient --batch-blocks 1000
```

Add `--dry-run` to snapshot inspection, or `--offline` when the matching range's
verified staging is already available. Repeat the import command to resume. See
[import/resume/cleanup](base-snapshot-download.md) and
[export/restore/P2P startup](base-history-packages.md) for limits and operational
requirements. No Python script is a runtime or CI dependency.

The opt-in Rust reference test opens an existing database and compares it with
saved independent RPC responses. Its reference JSON has `range: [start, end]`,
`reference_rpc`, ordered `blocks` entries containing `number`, `block` (the complete
`eth_getBlockByNumber(number, false)` response result), and `receipts` (transaction
receipt results in block order), plus `logs` containing all `eth_getLogs` results
for the entire range in canonical order. Both endpoint blocks must be sampled.
Fetch long log ranges in bounded non-overlapping chunks and concatenate them.
The test permits at most 100,000 blocks and an 8 MiB reference file.

```sh
SHINODE_HISTORY_DATA=/path/to/base-50000 \
SHINODE_HISTORY_REFERENCE=/path/to/reference.json \
SHINODE_HISTORY_PROOF=/path/to/comparison.json \
  cargo test --bin shinode --locked \
  rpc::history_reference_tests::history_matches_independent_reference -- --ignored
```

Repeat with the restored directory. Reference collection, raw snapshots, timing
logs and evidence files stay outside Git; the committed Rust test makes the
comparison reusable.

## Regression coverage and remaining limits

| Concern | Evidence |
| --- | --- |
| Empty blocks and deposits | Real genesis, 25 sampled deposits, typed RPC/storage fixtures |
| Fork rules | Generated Canyon/Isthmus and later-timestamp Compact archive reconstruction; real-source scale evidence remains pre-Canyon |
| Source archive boundaries | Generated multi-archive reconstruction and complete-row checks |
| Corrupt, truncated or pruned input | Manifest, downloader, tar, reconstruction and package rejection tests |
| Resource limits and interrupted writes | Disk/transfer limits, growing-shard compaction budgets, injected WAL/checkpoint failures and restart tests |
| Overlaps and resume metadata | Matching/gap-filling imports; anchored and unanchored rejection preserves the saved start; valid earlier restore updates it after validation |
| Recipient continuation | Restore → production typed-peer ingestion → RPC → restart, with reorg fencing and worker-drain cancellation |
| Actual public P2P | Earlier Base/Ethereum live runs in [base-protocol.md](base-protocol.md); this ancient range's public-peer bridge is **not** established |

Final checks passed: **159 workspace tests**, eight intentionally ignored;
release build; workspace no-default-features check; new-file formatting and diff
whitespace checks. The new reference test passed explicitly for both imported
and restored 50,000-block databases. Workspace all-target Clippy retains exactly
**220 errors and 15 warnings**, with no new diagnostics. Whole-workspace formatting
still has existing differences; this change does not reformat unrelated code.

The compaction seal is now persisted before returning, allowing native export to
verify it after reopen. The previously observed sealed-cache decode/fallback issue
remains distinct and unresolved. Real-source later-fork conversion, real cross-source-archive bulk imports,
busy-range capacity, cross-platform runtime measurements,
and sustained historical P2P availability remain unverified. Public-peer retention
is operator-dependent; the documented V2 full preset's 10,064-block window is not
a universal retention promise. No hosting or upload service is introduced.

## Initial archive dashboard evidence

The archive TUI was exercised in a real pseudo-terminal on the same host and
50,000-block range. A fresh archive download was cancelled after partial bytes
arrived, then resumed through a complete import. Export and clean restore also
completed. A second restore was cancelled after the first 1,000-block shard was
written: it retained the unfinished-restore marker without publishing a trusted
boundary, then completed on retry. Terminal settings and alternate-screen exit
were checked on success, q, Ctrl-C, and an input error. `--no-tui` stayed in plain
output even with a terminal, and dry-run retained clean JSON with empty stderr.

Imported, clean-restored and resumed-restored output passed the independent reference comparison.
The workspace suite passes **163 tests**, eight ignored, including archive UI
rendering at small sizes, concurrent counters, cancellation/worker cleanup, and
cancelled-download resume. These are behavior checks, not replacement throughput
measurements for the table above. Raw terminal captures and helper scripts remain
local under `/tmp/shinode-archive-tui-smoke` and are not runtime dependencies.
The release build and no-default-features check pass; Clippy retains its exact
220-error / 15-warning baseline. A completed-import TUI run also verified that
existing output size is shown while transfer and extraction counters stay at zero.

## Unified archive/history workflow

The archive UI now extends the existing Shinode renderer and retains the same
terminal controller into historical sync and follow. New jobs reconstruct from
the containing archive rather than downloading a genesis prefix. Automatic mode
probes actual typed peer bodies/receipts, checks the selected successor's parent
against an independent RPC, and journals the boundary for stable resumes.

Regression coverage includes pruned and headers-only peers, independent RPC
agreement/disagreement, saved-boundary identity and budgets, non-genesis imports
with missing earlier archives, wrong checkpoint rejection, later-fork Compact
archives, and shared rendering across all phases. Production pipeline tests still
cover archive → P2P ingestion, cross-boundary RPC, restart and reorg handling.
The workspace suite passed 173 tests (eight ignored); an additional archive ETA
regression passed with the five-test UI suite. Release and no-default-features
builds pass. Clippy matches the existing 220 errors / 15 warnings.

A real 50,000-block completed import ran in the shared TUI with exactly one
alternate-screen entry/exit and restored terminal settings. Its history handoff
kept that controller through startup. After tightening the startup quit path,
pressing q at handoff exited with code 0, one alternate-screen entry/exit and
restored terminal settings. One live attempt with this ancient range correctly
reported a history gap because its connected peers could not serve the successor.
Raw evidence remains local.
A full recent-chain archive → public history → live follow run is not claimed.

The separate archive/compact renderers and resize gate were removed after user
review. Archive progress now uses the original `render_sync_ui` layout, map, chart,
four panels and logs. Original colors and startup sound remain. Base archive
workflows alone receive ARCHIVE/HISTORY labels; Ethereum keeps STARTUP/SYNC.
Earlier compact-dashboard evidence describes the superseded implementation.

Original-style verification passed eight UI tests and a real cached 50,000-block
run at 120×30 (exit 0, one screen entry/exit, restored terminal). Fifteen original
layout/panel renderer bodies were compared against the committed version and are
unchanged except for passing state to the Base phase indicator. Release build
passes; Clippy has no added or removed diagnostics against its existing baseline.

User-provided terminal screenshots subsequently showed a 50,000-block archive
import handing off to history and adding 1,381 blocks (51,381 stored total).
A separate recent-range run starting at 51,227,600 reached FOLLOW at 51,228,026
with 427 stored blocks and RPC active. These observations demonstrate the two
transitions in separate runs, not a continuous archive-to-live bootstrap or
guaranteed availability of ancient history from public peers.

The final pre-push `cargo test --workspace --locked` run passed 175 tests with
eight ignored on 2026-09-13. No live-network tests were enabled in that run.
