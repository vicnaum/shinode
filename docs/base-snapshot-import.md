# Base snapshot conversion: inspection and bounded proof

Verified on 2026-09-12. A small historical Base range can be converted from V2
snapshot static files into Shinode without downloading execution state, MDBX, or
RocksDB. This document records the initial bounded proof. Phase 4 now exposes a
[Rust import command](base-snapshot-download.md) for the verified producer and
pre-Canyon format. Later forks remain unsupported. The existing Reth pin is unchanged.

## Pinned source and integrity

| Property | Value |
| --- | --- |
| Manifest | [1789084806/manifest.json](https://mainnet-v2-snapshots.base.org/1789084806/manifest.json) |
| Manifest SHA-256 | `d7ddeea72a46b542d3ca97672a64ba73ebc07599f0b19fd8965c00c2b317801c` |
| Chain / storage version | Base mainnet, 8453 / V2 |
| Manifest height | 51,147,727 |
| Producer reported by manifest | `2.5.2-dev (76a8261)` |
| Resolved Reth producer revision | `76a826109009c936c2ea98766e0b6b50d2803d26` |
| Shinode reader revision | `a74cb9cbc38a5590b2596c67e64a3c7642024aca` |
| Genesis | `0xf712aa9241cc24369b143cf6dce85f0902a9731e70d66818a3a5845b296c73dd` |
| Independent hash at manifest height | `0xc8ecb2bb909e54fde8b65a76950f119596fe04f828517454980eb96144661bfb` |

The manifest has no genesis or boundary block hash. Those anchors were read from
`https://mainnet.base.org`; the decoded genesis also matches Shinode's chain
configuration. The last snapshot header archive was not downloaded, so the hash
at manifest height is a reference anchor, not a verified snapshot-to-P2P boundary.

The pinned manifest supplies output-file sizes and BLAKE3 checksums. All nine
selected files passed those checks. It does not supply archive checksums: the
download helper records archive SHA-256 values locally for reproducibility.
Completed-chunk URLs are shared across snapshots and can change; tail archives
use snapshot-specific directories. The helper rejects
files that no longer match this manifest instead of switching to newer metadata.
HTTPS provenance, checksums, header commitments and independent sample comparison
provide different checks; they do not prove whole-chain canonicality or L1 finality.

Layout references: the producer's
[static-file segments](https://github.com/paradigmxyz/reth/blob/76a826109009c936c2ea98766e0b6b50d2803d26/crates/static-file/types/src/segment.rs),
Shinode's pinned
[compact codecs](https://github.com/paradigmxyz/reth/tree/a74cb9cbc38a5590b2596c67e64a3c7642024aca/crates/storage/codecs),
and the inspected Base
[receipt representation](https://github.com/base/base/blob/d5eed12dea87bff8688fbf0e1b789aa3f93f78a2/crates/common/consensus/src/receipts/receipt.rs).
The Base source revision is a format reference; the manifest does not identify
the exact Base binary revision that produced the snapshot.

## Input inventory

Totals below are bytes from the pinned manifest, not downloaded totals.

| Component | Compressed bytes | Extracted bytes | Needed by this proof |
| --- | ---: | ---: | --- |
| Headers | 19,659,082,782 | 30,449,120,251 | First chunk |
| Transactions | 1,214,878,587,351 | 2,322,456,952,030 | First chunk |
| Receipts | 646,272,838,863 | 1,445,101,840,428 | First chunk |
| Transaction senders | 44,573,625,863 | 209,181,160,220 | No |
| Account changesets | 134,409,679,737 | 343,636,979,861 | No |
| Storage changesets | 709,117,936,157 | 2,362,886,930,432 | No |
| State | 723,291,640,864 | 1,090,922,721,293 | No |
| RocksDB indices | 572,429,938,567 | 685,472,229,603 | No |

Headers, transactions and receipts alone total **1,880,810,508,996 compressed
bytes (1.88 TB)** and **3,798,007,912,709 extracted bytes (3.80 TB)**. Skipping state
still leaves a substantial full-history job. Some later transaction chunks exceed
32 GB compressed; chunking alone does not establish a small scratch requirement.

Use archival history components. The current
[Base snapshot documentation](https://docs.base.org/specifications/node-operators/snapshots)
describes V2 full/minimal presets as pruned subsets; the full preset retains
10,064 blocks, roughly 5–6 hours on Base. A peer's actual retention depends on
operator settings, so neither 30 days nor the full-preset window is a P2P guarantee.

## Reader and reconstruction

Each selected archive covers blocks 0–499,999 and contains a NippyJar data file,
`.conf`, and `.off`. Headers have three columns: compact header, total difficulty,
and block hash. Transactions and receipts each have one column indexed by
transaction number. The sample has 500,000 headers and 500,033 transactions and
receipts; both transaction-number ranges are 0–500,032.

These inputs contain no per-block BodyIndices table. The proof scans from genesis,
consuming transaction/receipt pairs until cumulative gas and the transaction root
match the current header. Gas equality alone is insufficient: another transaction
can consume zero additional gas. The common Base validator then checks transaction
and receipt roots, bloom, ommers and withdrawals, while the reader checks stored
header hashes and parent continuity. Offline tests cover ambiguous gas boundaries,
truncated input, mismatched receipt gas and unsupported withdrawals.

No body state is needed for these pre-Canyon blocks. Empty ommers are checked
against the header; withdrawals must be absent. Ordinary transaction metadata
uses the existing ingestion path, and deposit senders remain explicit in decoded
deposit transactions. No sender archive or execution-client index is used.
Exact block RLP size is computed from reconstructed bodies and compared with RPC.

The pinned Compact decoders successfully read this prefix of the newer snapshot.
On compressed branches their returned remainder is the original compressed input;
it is not an unused suffix. Jar offsets bound the input, and payload commitments
check the decoded values. This result does not establish general V2 compatibility:
post-Canyon withdrawals/body reconstruction and newer transaction encodings need
separate samples. The helper refuses headers with a withdrawals root.

Starting at arbitrary later chunks also remains unresolved. Production import
needs either sequential boundary reconstruction with reusable checkpoints or a
verified block-to-transaction index. Do not select only the requested later archive
and assume its first transaction belongs to its first header.

## Measured conversion and independent comparison

The helper downloaded only these archives, under limits of **512 MiB downloaded**,
**1 GiB extracted**, and an additional **5 GiB free-space reserve**:

| First chunk | Compressed bytes | Extracted bytes |
| --- | ---: | ---: |
| Headers | 58,105,905 | 144,436,251 |
| Transactions | 25,900,462 | 120,506,071 |
| Receipts | 1,133,974 | 14,583,994 |
| Total | **85,140,341** | **279,526,316** |

Download, extraction and checksum verification took 13.34 seconds. Because all
archives are retained, input scratch contains 364,666,657 bytes of file payload
at completion, plus small reports and filesystem overhead. Dependency/build cache
space is additional. No execution state was requested or deleted.

The reader verified **5,528 blocks (0–5,527)** and **5,538 transactions**, then
imported two complete eight-block shards: **4,984–4,991** and **5,520–5,527**.
The selected blocks contain **18 transactions: 16 deposits and two ordinary
transactions**, with **one event log**. These are deliberately quiet early-history
samples, not representative of current Base throughput.

Payloads passed `IngestChain::process_ingest`, WAL writing, compaction and sealing.
After dropping and reopening storage, receipt fields and transaction metadata
matched the original bundles. Queries used the production JSON-RPC module in
process; the experiment did not start a TCP RPC server.

All 16 blocks matched independent `https://mainnet.base.org` responses for block
hash, parent hash, state/transaction/receipt roots, bloom, timestamp, gas values,
exact block size, every transaction hash, and every standard log field. All 16
deposit nonces matched; receipt version was absent in this pre-Canyon sample.
The sole nonempty-log block was 5,521; ordinary transactions first appeared at
4,987 and 5,521. First and last imported hashes:

- 4,984: `0xeba9151d518758419acb82fcf1b040da60410c0c9e15dba0bc1f71907eba48bf`
- 5,527: `0x2246525ba06cf8771e47607736d502e446cf5cffe00a2646ceb12de043d4205f`

A debug test-binary run, excluding compilation, measured 0.93 seconds wall time
and **46,219,264 bytes peak RSS (44.1 MiB)** with macOS `/usr/bin/time -l`.
The prefix decode/validation took 0.707 seconds; selected payload conversion took
0.004 seconds and write/compact/reopen took 0.035 seconds. Final storage occupied
**16,123 bytes**, excluding raw RPC evidence. A subsequent gas-candidate optimization
produced identical RPC evidence. Timings are local sample observations, not a
production benchmark or a projection of complete-history output size.

The known source sizes establish the full-history input budget. A useful estimate
of converted TB and total runtime still requires busy ranges, later forks and
cross-archive boundary measurements before bulk import. The existing sealed-shard
cache metadata decode issue can fall back to shard files; this proof does not fix
that pre-existing cache issue.

## Reproduce

Use the [Rust snapshot-import command](base-snapshot-download.md) to inspect,
stage and convert the pinned range. It has no Python dependency. The opt-in Rust
RPC proof now exercises that production importer for the continuous 544-block
range 4,984–5,527 and samples the original 16 reference blocks after reopening.

The measurements above describe the original phase-1/2 experiment, which imported
only two selected eight-block shards. Python prototype scripts and raw manifests,
archives, RPC responses and timing logs remain local, outside the intended PR.
The production guide records the supported format, resource limits and remaining
later-fork and P2P handoff work.

## Validation

- `cargo test --workspace --locked --offline`: 107 passed, six ignored; the
  snapshot proof additionally passed when explicitly enabled with verified inputs.
  RPC socket tests require permission to bind local ports.
- `cargo check --workspace --no-default-features --locked --offline`: passed.
- New Rust file formatting, prototype Python syntax and `git diff --check`: passed.
- Wrong manifest and empty comparison input were rejected before creating output
  directories or making network requests.
- Workspace all-target Clippy still reports the existing 222 binary/test errors;
  there are no diagnostics in the new snapshot module.
