# Share and restore Base history

The [50,000-block integration validation](base-integration-validation.md) records
release-build import/restore costs and independent full-range log comparisons.

`history-export` packages complete native Shinode shards. `history-restore`
verifies and imports those packages using the Rust binary alone. Recipients need
only the package, its trusted manifest SHA-256, and an independently verified
end-block hash. They do not need the execution snapshot or the snapshot converter.

The first package format supports Base, storage schema 2, and the `op_v1` codec.
It distributes already imported history; it does not extend the execution
snapshot reader's supported producer versions or fork range.

## Export

Stop the writer using the source database. If older data has not been durably
sealed, run `db compact` first. Both commands infer the saved chain and shard size:

```sh
./target/release/shinode --data-dir /path/to/base-history db compact

./target/release/shinode --data-dir /path/to/base-history history-export \
  --start-block 4984 --end-block 5527 \
  --output-dir /path/to/new-history-package
```

This example requires the sample database's shard size of 8. The first block must
start a shard and the last block must end one. Every selected shard must be
complete, sorted, sealed, free of pending WAL, and within verified contiguous
coverage. Partial tail shards are excluded by selecting an earlier aligned end.

The source must already have a trusted snapshot boundary. When exporting through
that exact boundary, its saved hash is reused. For another end height, supply
`--checkpoint-hash <independently-verified-end-block-hash>` explicitly.

Export creates a **new** directory containing `manifest.json` and one
`shard-<start>.tar.zst` per shard. It prints the manifest SHA-256 and measured native
and compressed byte totals. The complete directory is published only after the
archives and manifest have been synced. Source and package directories must be
separate and cannot be nested.

The versioned manifest records chain/genesis, storage schema/codec/options, shard
size, exact ranges, boundary parent hashes, sealed-shard and stored-bundle hashes,
each file's size/SHA-256, and each archive's size/SHA-256. Provenance records the
source's trusted boundary and its original manifest hash. No machine paths are
included. Archives contain only `shard.json`, `present.bitset`, and the five native
sorted segments with their `.conf`/`.off` files. Execution state, WAL, peer caches,
logs, original snapshot inputs, and scratch files are excluded.

Distribute the package files using your own transport and supply the manifest
hash through a trusted channel. These commands do not upload or host anything.

## Restore

Keep all archives beside their manifest. Stop any writer using the destination:

```sh
./target/release/shinode history-restore \
  --manifest /path/to/history-package/manifest.json \
  --manifest-sha256 <publisher-manifest-sha256> \
  --checkpoint-hash <independently-verified-end-block-hash> \
  --output-dir /path/to/recipient-history
```

Restore defaults to Base and reads the shard size from the manifest. Explicit
conflicting chain selections and incompatible storage options are rejected.
The destination can be new, empty, or a matching Base database. Package and
destination directories must be separate and cannot be nested.

Before creating or adding blocks to the destination, restore checks all archives,
extracts only declared regular files into private temporary storage, checks native
seals, exact block coverage, header continuity and stored-bundle digests, then
compares every existing overlap and the combined parent chain. Missing, duplicate,
unexpected, linked, truncated, or checksum-mismatched entries are rejected.
Conflicting existing blocks and gaps before the restored range stop the job.

Matching blocks are reused. Missing blocks go through WAL and compaction;
existing shard files are not overwritten by archive extraction. After sealing and
reopening, every restored bundle is compared again, then the verified boundary
and manifest provenance are saved. An older matching package preserves an already
newer trusted boundary. A package cannot fill history below an existing trusted
range's recorded lower bound; use a fresh destination for that range.

Repeat the restore command after interruption. Recovery compacts pending WAL,
rechecks the package and all overlaps, and writes only missing blocks. An
interrupted first restore blocks normal P2P sync until a verified boundary is
published. Archives must remain available for a retry; extraction is repeated.
Temporary extraction is removed on ordinary completion or error. A killed process
can leave a private `.shinode-package-*` sibling directory; verify that its process
has stopped before manually removing that specific directory.

A publisher's trusted manifest pin authenticates the distributed **stored fields**.
The independent end hash anchors the parent-linked headers, but native Shinode
history does not retain full transaction envelopes needed to repeat the original
transaction-root validation. Hashes and seals detect changes; they do not establish
publisher trust, canonicality, OP safe/finalized status, or L1 finality.

## Disk limits and continuation

Export defaults to at most 1 GiB of selected native payload and a 5 GiB free-space
reserve. Restore defaults to 512 MiB of compressed archives, 1 GiB of extracted
payload, a 5 GiB reserve, and batches of 64 blocks. Adjust the corresponding
`--max-archive-bytes` (restore), `--max-extracted-bytes`, `--min-free-bytes`, and
`--batch-blocks` (restore, 1–1,024) options for a larger verified selection.

Restore needs the package files, a full temporary extracted copy, the resulting
database, and compaction working space. Per-batch checks include a copy of each
affected shard's existing sorted files. The decoder limits its zstd window to
128 MiB and tar metadata/padding to 1 MiB per shard beyond declared payload.
Neither limit is a total process RAM limit. Export uses zstd level 3; native
transaction metadata and receipts are already compressed, so measure actual
package savings instead of assuming a fixed ratio.

After successful restore, start normal sync with the saved lower bound:

```sh
nice -n 10 ./target/release/shinode \
  --chain base --data-dir /path/to/recipient-history \
  --p2p-port 30309 --rpc-bind 127.0.0.1:8546
```

The ordinary startup adopts the database's shard size. For old history, supply a
peer serving the successor and the intervening range with `--trusted-peer`.
Restoration does not fill the gap to today's public retention window by itself.
The [P2P handoff checks](base-snapshot-download.md#continue-through-p2p) and trusted
boundary rollback fence apply equally to restored history.

## Verified small sample

The local debug-build smoke test exported Base blocks **4,984–5,527** into 68
shards of eight blocks each, then restored them into a second clean directory
without opening any execution snapshot inputs. All 544 stored bundles matched;
16 sampled block/log/deposit responses matched the prior verified RPC reference.
Generated fixtures separately exercise restore, RPC, production ingestion through
controlled typed peers, and restart. This is not evidence of public peers serving
that ancient range.

| Measurement | Result |
| --- | ---: |
| Selected native shard files | 521,479 bytes |
| Compressed shard archives | 173,423 bytes |
| Distribution manifest | 238,906 bytes |
| Total package | 412,329 bytes |
| Export time | 2.7 seconds |
| Restore time | 29.3 seconds |

The total package was about **21% smaller**, including the manifest. These tiny
shards make per-file manifest metadata unusually large. This bounded debug run
is not a throughput, peak-RAM, or full-history capacity benchmark.

Run the generated acceptance tests without external inputs:

```sh
cargo test --bin shinode --locked snapshot::package
```

The opt-in sample comparison expects the verified native sample (including its
`proof.json` RPC reference) and a new output directory:

```sh
SHINODE_HISTORY_SOURCE=/path/to/verified-native-sample \
SHINODE_HISTORY_OUTPUT=/path/to/new-package-proof \
  cargo test --bin shinode --locked \
  snapshot::package::tests::packaged_sample_matches_rpc_reference -- --ignored
```
